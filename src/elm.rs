use crate::inflected_string::InflectedString;
use eyre::{bail, eyre, Result, WrapErr};
use jtd::Schema;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Maybe(Box<Type>),
    Unit,
    DictWithStringKeys(Box<Type>),
    List(Box<Type>),
    Ref(InflectedString),
    Record(BTreeMap<InflectedString, Type>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decl {
    CustomTypeEnum {
        name: InflectedString,
        discriminator: Option<String>,
        constructor_prefix: InflectedString,
        cases: BTreeMap<InflectedString, Option<Type>>,
    },
    TypeAlias {
        name: InflectedString,
        type_: Type,
    },
}

impl Decl {
    fn to_source(&self) -> Result<String> {
        let mut out = String::new();

        match self {
            Decl::CustomTypeEnum {
                name,
                constructor_prefix,
                cases,
                ..
            } => {
                out.push_str("type ");
                out.push_str(&name.to_pascal_case()?);
                out.push('\n');

                for (i, (case_name, case_type_opt)) in cases.iter().enumerate() {
                    if i == 0 {
                        out.push_str("    = ");
                    } else {
                        out.push_str("    | ");
                    }

                    out.push_str(&constructor_prefix.to_pascal_case()?);
                    out.push_str(&case_name.to_pascal_case()?);

                    if let Some(case_type) = case_type_opt {
                        out.push(' ');
                        out.push_str(&case_type.to_source()?.replace('\n', "\n    "));
                    }

                    out.push('\n')
                }
            }
            Decl::TypeAlias { name, type_ } => {
                out.push_str("type alias ");
                out.push_str(&name.to_pascal_case()?);
                out.push_str(" =\n    ");
                out.push_str(&type_.to_source()?.replace('\n', "\n    "));
            }
        }

        Ok(out)
    }

    fn name(&self) -> &InflectedString {
        match self {
            Decl::CustomTypeEnum { name, .. } => name,
            Decl::TypeAlias { name, .. } => name,
        }
    }

    fn decoder_name(&self) -> Result<String> {
        Ok(format!("{}Decoder", self.name().to_camel_case()?))
    }

    fn to_decoder_source(&self) -> Result<String> {
        let mut out = String::new();

        let name = self.name();
        let decoder_name = self.decoder_name()?;
        let type_name = name.to_pascal_case()?;
        out.push_str(&decoder_name);
        out.push_str(" : Decoder ");
        out.push_str(&type_name);
        out.push('\n');
        out.push_str(&decoder_name);
        out.push_str(" =\n");

        match &self {
            Decl::CustomTypeEnum {
                constructor_prefix,
                discriminator,
                cases,
                ..
            } => {
                out.push_str("    Decode.andThen\n        (\\tag ->\n            case tag of\n");

                for (case, case_type_opt) in cases {
                    out.push_str("                \"");
                    out.push_str(case.orig());
                    out.push_str("\" ->\n                    ");

                    match case_type_opt {
                        Some(type_) => {
                            let sub_decoder = type_.to_decoder_source(&type_name)?;

                            out.push_str("Decode.map ");
                            out.push_str(&constructor_prefix.to_pascal_case()?);
                            out.push_str(&case.to_pascal_case()?);
                            if sub_decoder.contains('\n') {
                                out.push_str("\n                        ");
                                out.push_str(
                                    &sub_decoder.replace('\n', "\n                        "),
                                );
                            } else {
                                out.push(' ');
                                out.push_str(&sub_decoder);
                            }
                        }
                        None => {
                            out.push_str("Decode.succeed ");
                            out.push_str(&constructor_prefix.to_pascal_case()?);
                            out.push_str(&case.to_pascal_case()?);
                        }
                    }
                    out.push_str("\n\n");
                }

                out.push_str("        )\n        ");
                match discriminator {
                    None => out.push_str("Decode.string"),
                    Some(name) => {
                        out.push_str("(Decode.field \"");
                        out.push_str(name);
                        out.push_str("\" Decode.string)");
                    }
                }
            }
            Decl::TypeAlias { type_, .. } => {
                out.push_str("    ");
                out.push_str(&type_.to_decoder_source(&type_name)?.replace('\n', "\n    "));
            }
        }

        Ok(out)
    }
}

impl Type {
    pub fn from_schema(
        schema: Schema,
        name_suggestion: Option<String>,
        globals: &BTreeMap<String, Schema>,
    ) -> Result<(Self, Vec<Decl>)> {
        let mut is_nullable = false;
        let mut decls = Vec::new();

        let base = match schema {
            Schema::Empty { .. } => Self::Unit,
            Schema::Ref {
                definitions,
                nullable,
                ref_,
                ..
            } => match definitions.get(&ref_).or_else(|| globals.get(&ref_)) {
                Some(schema) => {
                    is_nullable = nullable;

                    let (def_type, def_decls) = Self::from_schema(
                        schema.clone(),
                        name_suggestion.or_else(|| Some(ref_.to_string())),
                        globals,
                    )
                    .wrap_err_with(|| format!("could not convert the value of ref `{ref_}`"))?;

                    decls.extend(def_decls);
                    def_type
                }
                None => bail!("could not find a definition for `{ref_}`"),
            },
            Schema::Type {
                nullable, type_, ..
            } => {
                is_nullable = nullable;

                match type_ {
                    jtd::Type::Boolean => Self::Bool,
                    jtd::Type::Int8
                    | jtd::Type::Uint8
                    | jtd::Type::Int16
                    | jtd::Type::Uint16
                    | jtd::Type::Int32
                    | jtd::Type::Uint32 => Self::Int,
                    jtd::Type::Float32 | jtd::Type::Float64 => Self::Float,
                    jtd::Type::String | jtd::Type::Timestamp => Self::String,
                }
            }
            Schema::Enum {
                metadata,
                nullable,
                enum_,
                ..
            } => match metadata
                .get("name")
                .and_then(|n| n.as_str())
                .or(name_suggestion.as_deref())
            {
                Some(name) => {
                    is_nullable = nullable;

                    let mut cases = BTreeMap::new();
                    for value in enum_ {
                        cases.insert(value.into(), None);
                    }

                    decls.push(Decl::CustomTypeEnum {
                        name: name.into(),
                        discriminator: None,
                        constructor_prefix: metadata
                            .get("constructorPrefix")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .into(),
                        cases,
                    });

                    Self::Ref(name.into())
                }
                None => bail!("string names are required for enums"),
            },
            Schema::Elements {
                nullable, elements, ..
            } => {
                is_nullable = nullable;

                let (type_, sub_decls) = Self::from_schema(
                    *elements,
                    name_suggestion.map(|n| format!("{n}Elements")),
                    globals,
                )
                .wrap_err("could not convert elements of a list")?;

                decls.extend(sub_decls);

                Self::List(Box::new(type_))
            }
            Schema::Properties {
                metadata,
                nullable,
                properties,
                ..
            } => match metadata
                .get("name")
                .and_then(|n| n.as_str())
                .or(name_suggestion.as_deref())
            {
                Some(name) => {
                    is_nullable = nullable;

                    let mut fields = BTreeMap::new();
                    for (field_name, field_schema) in properties {
                        let (field_type, field_decls) =
                            Self::from_schema(field_schema, Some(field_name.clone()), globals)
                                .wrap_err_with(|| {
                                    format!("could not convert the type of `{field_name}`")
                                })?;

                        decls.extend(field_decls);
                        fields.insert(field_name.into(), field_type);
                    }

                    decls.push(Decl::TypeAlias {
                        name: name.into(),
                        type_: Self::Record(fields),
                    });

                    Self::Ref(name.into())
                }
                None => bail!("string names are required for properties"),
            },
            Schema::Values {
                nullable, values, ..
            } => {
                is_nullable = nullable;

                let (type_, sub_decls) = Self::from_schema(
                    *values,
                    name_suggestion.map(|n| format!("{n}Values")),
                    globals,
                )
                .wrap_err("could not convert elements of a list")?;

                decls.extend(sub_decls);

                Self::DictWithStringKeys(Box::new(type_))
            }
            Schema::Discriminator {
                metadata,
                nullable,
                mapping,
                discriminator,
                ..
            } => match metadata
                .get("name")
                .and_then(|n| n.as_str())
                .or(name_suggestion.as_deref())
            {
                Some(name) => {
                    is_nullable = nullable;

                    let mut cases = BTreeMap::new();
                    for (tag, tag_schema) in mapping {
                        let (value_type, value_decls) =
                            Self::from_schema(tag_schema, Some(tag.to_string()), globals)
                                .wrap_err_with(|| {
                                    format!("could not convert mapping for `{tag}`")
                                })?;

                        decls.extend(value_decls);
                        cases.insert(tag.into(), Some(value_type));
                    }
                    decls.push(Decl::CustomTypeEnum {
                        name: name.into(),
                        discriminator: Some(discriminator),
                        constructor_prefix: metadata
                            .get("constructorPrefix")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .into(),
                        cases,
                    });

                    Self::Ref(name.into())
                }
                None => bail!("string names are required for discriminators"),
            },
        };

        Ok((
            if is_nullable {
                Self::Maybe(Box::new(base))
            } else {
                base
            },
            decls,
        ))
    }

    fn to_source(&self) -> Result<String> {
        Ok(match self {
            Type::Bool => String::from("Bool"),
            Type::Int => String::from("Int"),
            Type::Float => String::from("Float"),
            Type::String => String::from("String"),
            Type::Maybe(inner) => {
                let mut out = String::from("Maybe ");

                let inner_source = inner.to_source()?;

                if inner_source.contains(' ') {
                    out.push('(');
                    out.push_str(&inner_source);
                    out.push(')');
                } else {
                    out.push_str(&inner_source);
                }

                out
            }
            Type::Unit => "()".to_string(),
            Type::DictWithStringKeys(inner) => {
                let mut out = String::from("Dict String ");

                let inner_source = inner.to_source()?;

                if inner_source.contains(' ') {
                    out.push('(');
                    out.push_str(&inner_source);
                    out.push(')');
                } else {
                    out.push_str(&inner_source);
                }

                out
            }
            Type::List(inner) => {
                let mut out = String::from("List ");

                let inner_source = inner.to_source()?;

                if inner_source.contains(' ') {
                    out.push('(');
                    out.push_str(&inner_source);
                    out.push(')');
                } else {
                    out.push_str(&inner_source);
                }

                out
            }
            Type::Ref(ref_) => ref_.to_pascal_case()?,
            Type::Record(fields) => {
                let mut out = String::new();

                if fields.is_empty() {
                    out.push_str("{}");
                } else {
                    for (i, (name, value)) in fields.iter().enumerate() {
                        if i == 0 {
                            out.push_str("{ ");
                        } else {
                            out.push_str("\n, ");
                        }

                        out.push_str(&name.to_camel_case()?);
                        out.push_str(" : ");
                        out.push_str(&value.to_source()?.replace('\n', "\n    "));
                    }

                    out.push_str("\n}");
                }
                out
            }
        })
    }

    fn to_decoder_source(&self, dest_type: &str) -> Result<String> {
        let mut out = String::new();

        match self {
            Type::Int => out.push_str("Decode.int"),
            Type::Float => out.push_str("Decode.float"),
            Type::Bool => out.push_str("Decode.bool"),
            Type::String => out.push_str("Decode.string"),
            Type::Maybe(type_) => {
                let sub_decoder = type_.to_decoder_source(dest_type)?;
                out.push_str("Decode.nullable ");

                if sub_decoder.contains(' ') {
                    out.push('(');
                    out.push_str(&sub_decoder);
                    out.push(')');
                } else {
                    out.push_str(&sub_decoder);
                }
            }
            Type::Unit => out.push_str("Decode.null ()"),
            Type::DictWithStringKeys(type_) => {
                let sub_decoder = type_.to_decoder_source(dest_type)?;
                out.push_str("Decode.dict ");

                if sub_decoder.contains(' ') {
                    out.push('(');
                    out.push_str(&sub_decoder);
                    out.push(')');
                } else {
                    out.push_str(&sub_decoder);
                }
            }
            Type::List(type_) => {
                let sub_decoder = type_.to_decoder_source(dest_type)?;
                out.push_str("Decode.list ");

                if sub_decoder.contains(' ') {
                    out.push('(');
                    out.push_str(&sub_decoder);
                    out.push(')');
                } else {
                    out.push_str(&sub_decoder);
                }
            }
            Type::Ref(name) => {
                out.push_str(&name.to_camel_case()?);
                out.push_str("Decoder");
            }
            Type::Record(fields) => {
                out.push_str("Decode.map");
                if fields.len() > 1 {
                    out.push_str(&fields.len().to_string()); // TODO: fix for >9
                }
                out.push(' ');
                out.push_str(dest_type);

                for (name, field_type) in fields {
                    let sub_decoder = field_type.to_decoder_source(dest_type)?;

                    out.push_str("\n    ");
                    out.push_str("(Decode.field \"");
                    out.push_str(name.orig());
                    out.push_str("\" ");

                    if sub_decoder.contains(' ') {
                        out.push('(');
                        out.push_str(&sub_decoder);
                        out.push(')');
                    } else {
                        out.push_str(&sub_decoder);
                    }

                    out.push(')');
                }
            }
        }

        Ok(out)
    }
}

#[derive(Debug)]
pub struct Module {
    pub name: Vec<String>,
    decls: Vec<Decl>,
}

impl Module {
    pub fn new(name: Vec<String>) -> Self {
        Self {
            name,
            decls: Vec::new(),
        }
    }

    pub fn insert_from_schema(
        &mut self,
        schema: Schema,
        name_suggestion: Option<String>,
        globals: &BTreeMap<String, Schema>,
    ) -> Result<()> {
        let (type_, decls) = Type::from_schema(schema, name_suggestion.clone(), globals)?;

        self.decls.extend(decls);

        match type_ {
            Type::Ref(_) => (),
            otherwise => self.decls.push(Decl::TypeAlias {
                name: name_suggestion
                    .ok_or(eyre!("need a name suggestion to create a top-level definition from an unnamed type"))
                    ?.into(),
                type_: otherwise,
            }),
        };

        Ok(())
    }

    pub fn to_source(&self) -> Result<String> {
        if self.decls.is_empty() {
            eyre::bail!(
                "Module {} didn't contain any definitions",
                self.name.join(".")
            )
        }

        let mut out = String::new();

        out.push_str("module ");
        out.push_str(&self.name.join("."));
        out.push_str(" exposing (..)\n\n{-| Warning: this file is automatically generated. Don't edit by hand!\n-}\n"); // TODO: expose exactly the things we need?

        for decl in &self.decls {
            out.push_str("\n\n");
            out.push_str(&decl.to_source()?);
            out.push_str("\n\n\n");
            out.push_str(&decl.to_decoder_source()?);
            out.push('\n');
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod type_ {
        use serde_json::{json, Value};

        use super::*;

        fn from_json(value: Value) -> jtd::Schema {
            let json = serde_json::from_value(value).unwrap();
            Schema::from_serde_schema(json).unwrap()
        }

        fn from_schema(value: Value) -> (Type, Vec<Decl>) {
            Type::from_schema(from_json(value), None, &BTreeMap::new())
                .expect("valid schema from JSON value")
        }

        #[test]
        fn interprets_int8() {
            let (type_, _) = from_schema(json!({"type": "int8"}));

            assert_eq!(type_, Type::Int);
        }

        #[test]
        fn interprets_int16() {
            let (type_, _) = from_schema(json!({"type": "int16"}));

            assert_eq!(type_, Type::Int);
        }

        #[test]
        fn interprets_int32() {
            let (type_, _) = from_schema(json!({"type": "int32"}));

            assert_eq!(type_, Type::Int);
        }

        #[test]
        fn interprets_uint8() {
            let (type_, _) = from_schema(json!({"type": "uint8"}));

            assert_eq!(type_, Type::Int);
        }

        #[test]
        fn interprets_uint16() {
            let (type_, _) = from_schema(json!({"type": "uint16"}));

            assert_eq!(type_, Type::Int);
        }

        #[test]
        fn interprets_uint32() {
            let (type_, _) = from_schema(json!({"type": "uint32"}));

            assert_eq!(type_, Type::Int);
        }

        #[test]
        fn interprets_float32() {
            let (type_, _) = from_schema(json!({"type": "float32"}));

            assert_eq!(type_, Type::Float);
        }

        #[test]
        fn interprets_float64() {
            let (type_, _) = from_schema(json!({"type": "float64"}));

            assert_eq!(type_, Type::Float);
        }

        #[test]
        fn interprets_string() {
            let (type_, _) = from_schema(json!({"type": "string"}));

            assert_eq!(type_, Type::String);
        }

        #[test]
        fn interprets_timestamp() {
            let (type_, _) = from_schema(json!({"type": "timestamp"}));

            assert_eq!(type_, Type::String);
        }

        #[test]
        fn interprets_boolean() {
            let (type_, _) = from_schema(json!({"type": "boolean"}));

            assert_eq!(type_, Type::Bool);
        }

        #[test]
        fn interprets_nullable_type() {
            let (type_, _) = from_schema(json!({"type": "string", "nullable": true}));

            assert_eq!(type_, Type::Maybe(Box::new(Type::String)));
        }

        #[test]
        fn interprets_empty() {
            let (type_, _) = from_schema(json!({}));

            assert_eq!(type_, Type::Unit);
        }

        #[test]
        fn interprets_values() {
            let (type_, _) = from_schema(json!({
                "values": {
                    "type": "string",
                },
            }));

            assert_eq!(type_, Type::DictWithStringKeys(Box::new(Type::String)));
        }

        #[test]
        fn interprets_nullable_values() {
            let (type_, _) = from_schema(json!({
                "values": {
                    "type": "string",
                },
                "nullable": true,
            }));

            assert_eq!(
                type_,
                Type::Maybe(Box::new(Type::DictWithStringKeys(Box::new(Type::String))))
            );
        }

        #[test]
        fn interprets_elements() {
            let (type_, _) = from_schema(json!({
                "elements": {
                    "type": "string",
                },
            }));

            assert_eq!(type_, Type::List(Box::new(Type::String)));
        }

        #[test]
        fn interprets_nullable_elements() {
            let (type_, _) = from_schema(json!({
                "elements": {
                    "type": "string",
                },
                "nullable": true,
            }));

            assert_eq!(
                type_,
                Type::Maybe(Box::new(Type::List(Box::new(Type::String))))
            );
        }

        #[test]
        fn interprets_enum_no_name() {
            let err = Type::from_schema(
                from_json(json!({
                    "enum": ["a", "b"],
                })),
                None,
                &BTreeMap::new(),
            )
            .unwrap_err();

            assert_eq!(
                err.to_string(),
                "string names are required for enums".to_string()
            );
        }

        #[test]
        fn interprets_enum() {
            let (type_, decls) = from_schema(json!({
                "metadata": {
                    "name": "Foo",
                },
                "enum": ["a", "b"],
            }));

            assert_eq!(type_, Type::Ref("Foo".into()));

            assert_eq!(
                decls,
                Vec::from([Decl::CustomTypeEnum {
                    name: "Foo".into(),
                    discriminator: None,
                    constructor_prefix: "".into(),
                    cases: BTreeMap::from([("a".into(), None), ("b".into(), None)]),
                }])
            );
        }

        #[test]
        fn interprets_properties_no_name() {
            let err = Type::from_schema(
                from_json(json!({
                    "properties": {
                        "a": {
                            "type": "string",
                        },
                        "b": {
                            "type": "int32",
                        },
                    },
                })),
                None,
                &BTreeMap::new(),
            )
            .unwrap_err();

            assert_eq!(
                err.to_string(),
                "string names are required for properties".to_string()
            );
        }

        #[test]
        fn interprets_properties() {
            let (type_, decls) = from_schema(json!({
                "metadata": {
                    "name": "Foo",
                },
                "properties": {
                    "a": {},
                    "b": {},
                },
            }));

            assert_eq!(type_, Type::Ref("Foo".into()));

            assert_eq!(
                decls,
                Vec::from([Decl::TypeAlias {
                    name: "Foo".into(),
                    type_: Type::Record(BTreeMap::from([
                        ("a".into(), Type::Unit),
                        ("b".into(), Type::Unit),
                    ])),
                }])
            );
        }

        #[test]
        fn interprets_mapping() {
            let (type_, decls) = from_schema(json!({
                "metadata": {
                    "name": "Foo",
                },
                "discriminator": "tag",
                "mapping": {
                    "a": {
                        "properties": {
                            "value": {
                                "type": "string"
                            }
                        }
                    },
                    "b": {
                        "properties": {
                            "value": {
                                "type": "float32"
                            }
                        }
                    },
                },
            }));

            assert_eq!(type_, Type::Ref("Foo".into()));

            assert_eq!(
                decls,
                Vec::from([
                    Decl::TypeAlias {
                        name: "a".into(),
                        type_: Type::Record(BTreeMap::from([("value".into(), Type::String)]))
                    },
                    Decl::TypeAlias {
                        name: "b".into(),
                        type_: Type::Record(BTreeMap::from([("value".into(), Type::Float)]))
                    },
                    Decl::CustomTypeEnum {
                        name: "Foo".into(),
                        discriminator: Some("tag".to_string()),
                        constructor_prefix: "".into(),
                        cases: BTreeMap::from([
                            ("a".into(), Some(Type::Ref("a".into()))),
                            ("b".into(), Some(Type::Ref("b".into()))),
                        ])
                    },
                ])
            );
        }

        #[test]
        fn interprets_ref_local() {
            let (type_, decls) = from_schema(json!({
                "definitions": {
                    "foo": {
                        "type": "string"
                    }
                },
                "ref": "foo",
            }));

            assert_eq!(type_, Type::String);
            assert_eq!(decls, Vec::new());
        }

        #[test]
        fn interprets_ref_global() {
            let (type_, decls) = Type::from_schema(
                from_json(json!({
                    "ref": "foo",
                })),
                None,
                &BTreeMap::from([("foo".into(), from_json(json!({"type": "string"})))]),
            )
            .unwrap();

            assert_eq!(type_, Type::String);
            assert_eq!(decls, Vec::new());
        }

        #[test]
        fn interprets_ref_suggesting_name() {
            let (type_, decls) = Type::from_schema(
                from_json(json!({
                    "ref": "foo",
                })),
                None,
                &BTreeMap::from([("foo".into(), from_json(json!({"properties": {}})))]),
            )
            .unwrap();

            assert_eq!(type_, Type::Ref("foo".into()));
            assert_eq!(
                decls,
                Vec::from([Decl::TypeAlias {
                    name: "foo".into(),
                    type_: Type::Record(BTreeMap::new())
                }])
            );
        }
    }

    mod module {
        use super::*;
        use serde_json::{json, Value};

        fn from_json(value: Value) -> jtd::Schema {
            let json = serde_json::from_value(value).unwrap();
            Schema::from_serde_schema(json).unwrap()
        }

        fn from_schema(value: Value, name_suggestion: Option<String>) -> Module {
            let mut module = Module::new(Vec::from(["Main".into()]));

            module
                .insert_from_schema(from_json(value), name_suggestion, &BTreeMap::new())
                .expect("valid schema from JSON value");

            module
        }

        #[test]
        fn from_schema_ref() {
            let mod_ = from_schema(
                json!({
                    "properties": {
                        "a": {
                            "type": "string"
                        }
                    }
                }),
                Some("Flags".into()),
            );

            assert_eq!(
                mod_.decls,
                Vec::from([Decl::TypeAlias {
                    name: "Flags".into(),
                    type_: Type::Record(BTreeMap::from([("a".into(), Type::String)]))
                }])
            );
        }

        #[test]
        fn from_schema_non_ref() {
            let mod_ = from_schema(json!({"type": "string"}), Some("Flags".into()));

            assert_eq!(
                mod_.decls,
                Vec::from([Decl::TypeAlias {
                    name: "Flags".into(),
                    type_: Type::String
                }])
            );
        }

        #[test]
        fn error_on_no_defs_to_source() {
            let m = Module {
                name: Vec::from(["A".to_string(), "B".to_string()]),
                decls: Vec::new(),
            };

            let err = m.to_source().unwrap_err();

            assert_eq!(
                err.to_string(),
                "Module A.B didn't contain any definitions".to_string()
            )
        }
    }
}
