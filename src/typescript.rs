use crate::inflected_string::InflectedString;
use color_eyre::Result;
use eyre::{bail, WrapErr};
use jtd::{Schema, Type};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub enum TSType {
    Object {
        properties: BTreeMap<InflectedString, (TSType, FieldPresence)>,
        nullable: bool,
    },
    NeverObject,
    Record {
        values: Box<TSType>,
        nullable: bool,
    },
    Scalar {
        value: &'static str,
        nullable: bool,
    },
    StringScalar(String),
    TypeRef(String),
    Union {
        members: Vec<TSType>,
        nullable: bool,
    },
    List {
        elements: Box<TSType>,
        nullable: bool,
    },
    Function {
        args: BTreeMap<InflectedString, TSType>,
        returning: Box<TSType>,
    },

    // For the following members, we're making no effort to constrain what's valid where. That's up
    // to our tests!
    TypeDecl {
        name: InflectedString,
        definition: Box<TSType>,
    },
    ModuleDecl {
        name: InflectedString,
        members: Vec<TSType>,
    },
    NamespaceDecl {
        name: InflectedString,
        members: Vec<TSType>,
    },
    NamedFunctionDecl {
        name: InflectedString,
        function: Box<TSType>, // in practice, should always be a `Function`
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum FieldPresence {
    Required,
    Optional,
}

impl TSType {
    pub fn from_schema(schema: Schema, globals: &BTreeMap<String, Schema>) -> Result<Self> {
        match schema {
            Schema::Properties {
                properties,
                optional_properties,
                nullable,
                ..
            } => {
                let mut converted_properties = BTreeMap::new();
                for (name, value) in properties {
                    let type_ = Self::from_schema(value, globals)
                        .wrap_err_with(|| format!("could not convert the {name} key"))?;

                    converted_properties.insert(name.into(), (type_, FieldPresence::Required));
                }

                for (name, value) in optional_properties {
                    let type_ = Self::from_schema(value, globals)
                        .wrap_err_with(|| format!("could not convert the {name} key"))?;

                    converted_properties.insert(name.into(), (type_, FieldPresence::Optional));
                }

                Ok(Self::Object {
                    properties: converted_properties,
                    nullable,
                })
            }
            Schema::Type {
                type_, nullable, ..
            } => Ok(Self::Scalar {
                value: match type_ {
                    Type::Int8
                    | Type::Int16
                    | Type::Int32
                    | Type::Uint8
                    | Type::Uint16
                    | Type::Uint32
                    | Type::Float32
                    | Type::Float64 => "number",
                    Type::String | Type::Timestamp => "string",
                    Type::Boolean => "boolean",
                },
                nullable,
            }),
            Schema::Enum {
                enum_, nullable, ..
            } => Ok(Self::Union {
                members: enum_.into_iter().map(Self::StringScalar).collect(),
                nullable,
            }),
            Schema::Empty { .. } => Ok(Self::NeverObject),
            Schema::Ref {
                ref_,
                nullable,
                definitions,
                ..
            } => match definitions.get(&ref_).or_else(|| globals.get(&ref_)) {
                Some(schema) => {
                    let mut tstype = Self::from_schema(schema.clone(), globals)
                        .wrap_err_with(|| format!("could not convert the value of ref `{ref_}`"))?;
                    tstype.set_nullable(nullable);
                    Ok(tstype)
                }
                None => bail!("could not find a definition for `{ref_}`"),
            },
            Schema::Elements {
                elements, nullable, ..
            } => Ok(Self::List {
                elements: Box::new(
                    Self::from_schema(*elements, globals)
                        .wrap_err("could not convert the elements type")?,
                ),
                nullable,
            }),
            Schema::Values {
                values, nullable, ..
            } => Ok(Self::Record {
                values: Box::new(
                    Self::from_schema(*values, globals)
                        .wrap_err("could not convert the values type")?,
                ),
                nullable,
            }),
            Schema::Discriminator {
                discriminator,
                mapping,
                nullable,
                ..
            } => {
                let mut members = Vec::with_capacity(mapping.len());

                for (tag, value) in mapping {
                    let mut value_type = Self::from_schema(value, globals)
                        .wrap_err_with(|| format!("could not convert the {tag} tag"))?;

                    // This happens if the payload is empty in a mapping field.
                    if value_type == Self::NeverObject {
                        value_type = Self::new_object(BTreeMap::new())
                    }

                    value_type
                        .add_key_to_object(&discriminator, Self::StringScalar(tag), FieldPresence::Required)
                        .wrap_err("jtd discriminator should have enforced that the value type must be an object")?;

                    members.push(value_type);
                }

                Ok(Self::Union { members, nullable })
            }
        }
    }

    fn set_nullable(&mut self, new_value: bool) {
        match self {
            TSType::Object { nullable, .. } => *nullable = new_value,
            TSType::Record { nullable, .. } => *nullable = new_value,
            TSType::Scalar { nullable, .. } => *nullable = new_value,
            TSType::Union { nullable, .. } => *nullable = new_value,
            TSType::List { nullable, .. } => *nullable = new_value,
            TSType::Function { .. }
            | TSType::TypeDecl { .. }
            | TSType::ModuleDecl { .. }
            | TSType::NamespaceDecl { .. }
            | TSType::NamedFunctionDecl { .. }
            | TSType::StringScalar(_)
            | TSType::TypeRef(_)
            | TSType::NeverObject => (),
        }
    }

    pub fn to_source(&self, is_toplevel: bool) -> Result<String> {
        let mut out = String::new();

        match self {
            Self::Object {
                properties,
                nullable,
            } => {
                out.push_str("{\n");
                for (name, (value, presence)) in properties {
                    out.push_str("  ");
                    out.push_str(&name.to_camel_case()?);
                    if *presence == FieldPresence::Optional {
                        out.push('?');
                    }
                    out.push_str(": ");
                    out.push_str(&value.to_source(false)?.replace('\n', "\n  "));
                    out.push_str(";\n");
                }
                out.push('}');

                if *nullable {
                    out.push_str(" | null");
                }
            }
            Self::NeverObject => out.push_str("Record<string, never>"),
            Self::Record { values, nullable } => {
                out.push_str("Record<string, ");
                out.push_str(&values.to_source(false)?);
                out.push('>');

                if *nullable {
                    out.push_str(" | null");
                }
            }
            Self::Scalar { value, nullable } => {
                out.push_str(value);
                if *nullable {
                    out.push_str(" | null");
                }
            }
            Self::StringScalar(string) => {
                out.push('"');
                out.push_str(string);
                out.push('"');
            }
            Self::TypeRef(ref_) => out.push_str(ref_),
            Self::Union { members, nullable } => {
                for (i, type_) in members.iter().enumerate() {
                    if i != 0 {
                        out.push_str(" | ");
                    }
                    out.push_str(&type_.to_source(false)?)
                }

                if *nullable {
                    out.push_str(" | null")
                }
            }
            Self::List { elements, nullable } => {
                let elements_source = elements.to_source(false)?;

                if elements_source.contains(' ') {
                    out.push('(');
                    out.push_str(&elements_source);
                    out.push_str(")[]");
                } else {
                    out.push_str(&elements_source);
                    out.push_str("[]");
                }

                if *nullable {
                    out.push_str(" | null");
                }
            }
            Self::Function { args, returning } => {
                out.push('(');
                for (i, (name, type_)) in args.iter().enumerate() {
                    if i != 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&name.to_camel_case()?);
                    out.push_str(": ");
                    out.push_str(&type_.to_source(false)?);
                }
                if is_toplevel {
                    out.push_str("): ");
                } else {
                    out.push_str(") => ");
                }
                out.push_str(&returning.to_source(false)?);
            }
            Self::TypeDecl { name, definition } => {
                out.push_str("type ");
                out.push_str(&name.to_pascal_case()?);
                out.push_str(" = ");
                out.push_str(&definition.to_source(false)?);
            }
            Self::ModuleDecl { name, members } => {
                out.push_str("declare module ");
                out.push_str(&name.to_pascal_case()?);
                out.push_str(" {\n");
                for (i, member) in members.iter().enumerate() {
                    if i > 0 {
                        out.push('\n');
                    }
                    out.push_str("  ");
                    out.push_str(&member.to_source(true)?.replace('\n', "\n  "));
                    out.push('\n');
                }
                out.push('}');
            }
            Self::NamespaceDecl { name, members } => {
                out.push_str("namespace ");
                out.push_str(&name.to_pascal_case()?);
                out.push_str(" {\n");
                for (i, member) in members.iter().enumerate() {
                    if i > 0 {
                        out.push('\n');
                    }
                    out.push_str("  ");
                    out.push_str(&member.to_source(true)?.replace('\n', "\n  "));
                    out.push('\n');
                }
                out.push('}');
            }
            Self::NamedFunctionDecl { name, function } => {
                out.push_str("function ");
                out.push_str(&name.to_camel_case()?);
                out.push_str(&function.to_source(true)?);
            }
        }

        Ok(out)
    }

    pub fn new_object(properties: BTreeMap<&str, (TSType, FieldPresence)>) -> Self {
        Self::Object {
            properties: properties.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            nullable: false,
        }
    }

    pub fn new_singleton_object(key: &str, value: TSType, presence: FieldPresence) -> Self {
        Self::new_object(BTreeMap::from([(key, (value, presence))]))
    }

    pub fn add_key_to_object(
        &mut self,
        key: &str,
        value: TSType,
        presence: FieldPresence,
    ) -> Result<()> {
        match self {
            Self::Object { properties, .. } => {
                properties.insert(key.into(), (value, presence));
                Ok(())
            }
            _ => bail!("add_key_to_object only works on objects"),
        }
    }

    pub fn new_neverobject() -> Self {
        Self::NeverObject
    }

    pub fn new_function(args: BTreeMap<&str, TSType>, returning: TSType) -> Self {
        Self::Function {
            args: args.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            returning: Box::new(returning),
        }
    }

    pub fn new_void() -> Self {
        Self::Scalar {
            value: "void",
            nullable: false,
        }
    }

    fn new_init(flags: TSType) -> Self {
        Self::new_named_function(
            "init",
            Self::new_function(
                BTreeMap::from([(
                    "config",
                    Self::new_object(BTreeMap::from([
                        (
                            "node",
                            (
                                Self::Scalar {
                                    value: "HTMLElement",
                                    nullable: false,
                                },
                                FieldPresence::Required,
                            ),
                        ),
                        ("flags", (flags, FieldPresence::Required)),
                    ])),
                )]),
                Self::new_singleton_object(
                    "ports",
                    Self::new_ref("Ports"),
                    // ports is optional because if the app does not set up any ports, Elm will
                    // omit this field in the generated code.
                    FieldPresence::Optional,
                ),
            ),
        )
    }

    pub fn new_send_function(value: TSType) -> Self {
        TSType::new_function(BTreeMap::from([("value", value)]), TSType::new_void())
    }

    pub fn new_subscribe_function(value: TSType) -> Self {
        TSType::new_function(
            BTreeMap::from([(
                "callback",
                TSType::new_function(BTreeMap::from([("value", value)]), TSType::new_void()),
            )]),
            TSType::new_void(),
        )
    }

    pub fn new_ref(name: &str) -> Self {
        Self::TypeRef(name.to_owned())
    }

    fn new_module(name: &str, members: Vec<TSType>) -> Self {
        Self::ModuleDecl {
            name: name.into(),
            members,
        }
    }

    pub fn new_namespace(name: &str, members: Vec<Self>) -> Self {
        Self::NamespaceDecl {
            name: name.into(),
            members,
        }
    }

    pub fn new_named_function(name: &str, function: TSType) -> Self {
        Self::NamedFunctionDecl {
            name: name.into(),
            function: Box::from(function),
        }
    }

    pub fn into_init(self) -> Self {
        Self::new_init(self)
    }

    pub fn into_typedecl(self, name: &str) -> Self {
        Self::TypeDecl {
            name: name.into(),
            definition: Box::from(self),
        }
    }
}

#[derive(Debug)]
pub enum NamespaceBuilder {
    Root {
        name: String,
        below: BTreeMap<String, NamespaceBuilder>,
    },
    Branch {
        name: String,
        members: Vec<TSType>,
        below: BTreeMap<String, NamespaceBuilder>,
    },
}

impl NamespaceBuilder {
    pub fn root(name: &str) -> Self {
        Self::Root {
            name: name.to_owned(),
            below: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, path: &[&str], value: TSType) -> Result<()> {
        let mut here = self;
        for part in path {
            let below = match here {
                Self::Root { below, .. } => below,
                Self::Branch { below, .. } => below,
            };

            here = below
                .entry(part.to_string())
                .or_insert_with(|| Self::Branch {
                    name: part.to_string(),
                    members: Vec::new(),
                    below: BTreeMap::new(),
                });
        }

        match here {
            Self::Root { .. } => {
                eyre::bail!("path provided led to a Root namespace builder. Can't insert!")
            }
            Self::Branch { members, .. } => members.push(value),
        }

        Ok(())
    }

    pub fn into_tstype(self) -> TSType {
        match self {
            Self::Root { name, below } => TSType::new_module(
                &name,
                below.into_values().map(|v| v.into_tstype()).collect(),
            ),
            Self::Branch {
                name,
                members,
                below,
            } => {
                let mut ts_members = Vec::with_capacity(members.len() + below.len());
                ts_members.extend(members);
                ts_members.extend(
                    below
                        .into_values()
                        .map(|v| v.into_tstype())
                        .collect::<Vec<TSType>>(),
                );

                TSType::new_namespace(&name, ts_members)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn from_json(value: Value) -> jtd::Schema {
        let json = serde_json::from_value(value).unwrap();
        jtd::Schema::from_serde_schema(json).unwrap()
    }

    fn from_schema(value: Value) -> TSType {
        TSType::from_schema(from_json(value), &BTreeMap::new())
            .expect("valid schema from JSON value")
    }

    #[test]
    fn interprets_int8() {
        assert_eq!(
            from_schema(json!({"type": "int8"})),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_int16() {
        assert_eq!(
            from_schema(json!({"type": "int16"})),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_int32() {
        assert_eq!(
            from_schema(json!({"type": "int32"})),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_uint8() {
        assert_eq!(
            from_schema(json!({"type": "uint8"})),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_uint16() {
        assert_eq!(
            from_schema(json!({"type": "uint16"})),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_uint32() {
        assert_eq!(
            from_schema(json!({"type": "uint32"})),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_float32() {
        assert_eq!(
            from_schema(json!({"type": "float32"})),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_float64() {
        assert_eq!(
            from_schema(json!({"type": "float64"})),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_string() {
        let type_ = from_schema(json!({"type": "string"}));

        assert_eq!(type_.to_source(true).unwrap(), "string".to_string())
    }

    #[test]
    fn interprets_boolean() {
        let type_ = from_schema(json!({"type": "boolean"}));

        assert_eq!(type_.to_source(true).unwrap(), "boolean".to_string())
    }

    #[test]
    fn interprets_properties() {
        let type_ = from_schema(json!({
            "properties": {
                "a": { "type": "float32" }
            }
        }));

        assert_eq!(
            type_.to_source(true).unwrap(),
            "{\n  a: number;\n}".to_string()
        )
    }

    #[test]
    fn interprets_optional_properties() {
        let type_ = from_schema(json!({
            "optionalProperties": {
                "a": { "type": "float32" }
            }
        }));

        assert_eq!(
            type_.to_source(true).unwrap(),
            "{\n  a?: number;\n}".to_string()
        )
    }

    #[test]
    fn interprets_enum() {
        let type_ = from_schema(json!({"enum": ["a", "b"]}));

        assert_eq!(type_.to_source(true).unwrap(), "\"a\" | \"b\"".to_string())
    }

    #[test]
    fn interprets_elements() {
        let type_ = from_schema(json!({"elements": {"type": "string"}}));

        assert_eq!(
            type_,
            TSType::List {
                elements: Box::new(TSType::Scalar {
                    value: "string",
                    nullable: false
                }),
                nullable: false
            }
        )
    }

    #[test]
    fn interprets_values() {
        let type_ = from_schema(json!({"values": {"type": "string"}}));

        assert_eq!(
            type_,
            TSType::Record {
                values: Box::new(TSType::Scalar {
                    value: "string",
                    nullable: false
                }),
                nullable: false
            }
        )
    }

    #[test]
    fn interprets_discriminator() {
        let type_ = from_schema(json!({
            "discriminator": "tag",
            "mapping": {
                "one": {
                    "properties": {
                        "a": {
                            "type": "float32"
                        }
                    }
                },
                "two": {
                    "properties": {
                        "b": {
                            "type": "string"
                        }
                    }
                }
            }
        }));

        assert_eq!(
            type_,
            TSType::Union {
                members: Vec::from([
                    TSType::new_object(BTreeMap::from([
                        (
                            "tag",
                            (
                                TSType::StringScalar("one".to_string()),
                                FieldPresence::Required
                            )
                        ),
                        (
                            "a",
                            (
                                TSType::Scalar {
                                    value: "number",
                                    nullable: false
                                },
                                FieldPresence::Required
                            )
                        ),
                    ])),
                    TSType::new_object(BTreeMap::from([
                        (
                            "tag",
                            (
                                TSType::StringScalar("two".to_string()),
                                FieldPresence::Required
                            )
                        ),
                        (
                            "b",
                            (
                                TSType::Scalar {
                                    value: "string",
                                    nullable: false
                                },
                                FieldPresence::Required
                            )
                        ),
                    ])),
                ]),
                nullable: false
            }
        )
    }

    #[test]
    fn interprets_ref() {
        let type_ = from_schema(json!({
            "ref": "foo",
            "definitions": {
                "foo": {
                    "type": "string"
                },
            },
        }));

        assert_eq!(
            type_,
            TSType::Scalar {
                value: "string",
                nullable: false,
            }
        );
    }

    #[test]
    fn interprets_ref_nullable() {
        let type_ = from_schema(json!({
            "ref": "foo",
            "nullable": true,
            "definitions": {
                "foo": {
                    "type": "string"
                },
            },
        }));

        assert_eq!(
            type_,
            TSType::Scalar {
                value: "string",
                nullable: true,
            }
        );
    }

    #[test]
    fn interprets_ref_global() {
        let ref_schema = from_json(json!({"ref": "foo"}));
        let def_schema = from_json(json!({"type": "string"}));

        let type_ = TSType::from_schema(
            ref_schema,
            &BTreeMap::from([("foo".to_string(), def_schema)]),
        )
        .unwrap();

        assert_eq!(
            type_,
            TSType::Scalar {
                value: "string",
                nullable: false,
            }
        );
    }

    #[test]
    fn interprets_ref_missing_definition() {
        let ref_schema = from_json(json!({"ref": "foo"}));

        let err = TSType::from_schema(ref_schema, &BTreeMap::new()).unwrap_err();

        assert_eq!(
            err.to_string(),
            "could not find a definition for `foo`".to_string(),
        );
    }

    #[test]
    fn scalar_to_source() {
        let type_ = from_schema(json!({"type": "string"}));

        assert_eq!(type_.to_source(true).unwrap(), "string".to_string());
    }

    #[test]
    fn nullable_scalar_to_source() {
        let type_ = from_schema(json!({"type": "string", "nullable": true}));

        assert_eq!(type_.to_source(true).unwrap(), "string | null".to_string());
    }

    #[test]
    fn function_to_source_toplevel() {
        let type_ = TSType::new_function(
            BTreeMap::from([
                (
                    "one",
                    TSType::Scalar {
                        value: "number",
                        nullable: false,
                    },
                ),
                (
                    "two",
                    TSType::Scalar {
                        value: "string",
                        nullable: false,
                    },
                ),
            ]),
            TSType::Scalar {
                value: "string",
                nullable: false,
            },
        );

        assert_eq!(
            type_.to_source(true).unwrap(),
            "(one: number, two: string): string".to_string()
        )
    }

    #[test]
    fn function_to_source_not_toplevel() {
        let type_ = TSType::new_function(
            BTreeMap::from([
                (
                    "one",
                    TSType::Scalar {
                        value: "number",
                        nullable: false,
                    },
                ),
                (
                    "two",
                    TSType::Scalar {
                        value: "string",
                        nullable: false,
                    },
                ),
            ]),
            TSType::Scalar {
                value: "string",
                nullable: false,
            },
        );

        assert_eq!(
            type_.to_source(false).unwrap(),
            "(one: number, two: string) => string".to_string()
        )
    }

    #[test]
    fn typedecl_to_source() {
        let type_ =
            from_schema(json!({"properties": {"a": {"type": "string"}}})).into_typedecl("Flags");

        assert_eq!(
            type_.to_source(true).unwrap(),
            "type Flags = {\n  a: string;\n}".to_string(),
        )
    }

    #[test]
    fn method_to_source() {
        let method = TSType::new_named_function(
            "init",
            TSType::new_function(
                BTreeMap::new(),
                TSType::Scalar {
                    value: "void",
                    nullable: false,
                },
            ),
        );

        assert_eq!(
            method.to_source(true).unwrap(),
            "function init(): void".to_string()
        );
    }

    #[test]
    fn module_to_source() {
        let namespace = TSType::new_module(
            "Elm",
            Vec::from([TSType::new_namespace("Main", Vec::new())]),
        );

        assert_eq!(
            namespace.to_source(true).unwrap(),
            "declare module Elm {\n  namespace Main {\n  }\n}".to_string()
        );
    }

    #[test]
    fn namespace_to_source() {
        let namespace = TSType::new_namespace("Main", Vec::from([]));

        assert_eq!(
            namespace.to_source(true).unwrap(),
            "namespace Main {\n}".to_string()
        );
    }

    #[test]
    fn list_to_source() {
        let type_ = from_schema(json!({"elements": {"type": "string"}}));

        assert_eq!(type_.to_source(true).unwrap(), "string[]".to_string());
    }

    #[test]
    fn list_to_source_space() {
        let type_ = from_schema(json!({"elements": {"enum": ["a", "b"]}}));

        assert_eq!(
            type_.to_source(true).unwrap(),
            "(\"a\" | \"b\")[]".to_string()
        );
    }

    #[test]
    fn values_to_source_space() {
        let type_ = from_schema(json!({"values": {"type": "string"}}));

        assert_eq!(
            type_.to_source(true).unwrap(),
            "Record<string, string>".to_string()
        );
    }

    #[test]
    fn discriminator_to_source() {
        let type_ = from_schema(json!({
            "discriminator": "tag",
            "mapping": {
                "one": {
                    "properties": {
                        "a": {
                            "type": "float32"
                        }
                    }
                },
                "two": {
                    "properties": {
                        "b": {
                            "type": "string"
                        }
                    }
                }
            }
        }));

        assert_eq!(
            type_.to_source(true).unwrap(),
            "{\n  a: number;\n  tag: \"one\";\n} | {\n  b: string;\n  tag: \"two\";\n}".to_string()
        );
    }
}
