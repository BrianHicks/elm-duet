use color_eyre::Result;
use jtd::{Schema, Type};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub enum TSType {
    Object {
        properties: BTreeMap<String, TSType>,
        nullable: bool,
    },
    NeverObject,
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
        args: BTreeMap<String, TSType>,
        returning: Box<TSType>,
    },

    // For the following members, we're making no effort to constrain what's valid where. That's up
    // to our tests!
    TypeDecl {
        name: String,
        definition: Box<TSType>,
    },
    ModuleDecl {
        name: String,
        members: Vec<TSType>,
    },
    NamespaceDecl {
        name: String,
        members: Vec<TSType>,
    },
    NamedFunctionDecl {
        name: String,
        function: Box<TSType>, // in practice, should always be a `Function`
    },
}

impl TSType {
    pub fn from_schema(schema: Schema) -> Self {
        match schema {
            Schema::Properties {
                properties,
                nullable,
                ..
            } => Self::Object {
                properties: properties
                    .into_iter()
                    .map(|(name, value)| (name, Self::from_schema(value)))
                    .collect(),
                nullable,
            },
            Schema::Type {
                type_, nullable, ..
            } => Self::Scalar {
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
                    Type::Boolean => "bool",
                },
                nullable,
            },
            Schema::Enum {
                enum_, nullable, ..
            } => Self::Union {
                members: enum_.into_iter().map(Self::StringScalar).collect(),
                nullable,
            },
            Schema::Empty { .. } => Self::NeverObject,
            Schema::Ref { ref_, nullable, .. } => todo!(),
            Schema::Elements {
                elements, nullable, ..
            } => Self::List {
                elements: Box::new(Self::from_schema(*elements)),
                nullable,
            },
            Schema::Values {
                values, nullable, ..
            } => todo!(),
            Schema::Discriminator {
                discriminator,
                mapping,
                nullable,
                ..
            } => todo!(),
        }
    }

    pub fn to_source(&self, is_toplevel: bool) -> String {
        let mut out = String::new();

        match self {
            Self::Object {
                properties,
                nullable,
            } => {
                out.push_str("{\n");
                for (name, value) in properties {
                    out.push_str("  ");
                    out.push_str(name); // TODO: escape?
                    out.push_str(": ");
                    out.push_str(&value.to_source(false).replace('\n', "\n  "));
                    out.push_str(";\n");
                }
                out.push('}');

                if *nullable {
                    out.push_str(" | null");
                }
            }
            Self::NeverObject => out.push_str("Record<string, never>"),
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
                    out.push_str(&type_.to_source(false))
                }

                if *nullable {
                    out.push_str(" | null")
                }
            }
            Self::List { elements, nullable } => {
                let elements_source = elements.to_source(false);

                if elements_source.contains(' ') {
                    out.push_str("(");
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
                    out.push_str(name);
                    out.push_str(": ");
                    out.push_str(&type_.to_source(false));
                }
                if is_toplevel {
                    out.push_str("): ");
                } else {
                    out.push_str(") => ");
                }
                out.push_str(&returning.to_source(false));
            }
            Self::TypeDecl { name, definition } => {
                out.push_str("type ");
                out.push_str(name); // TODO: escape?
                out.push_str(" = ");
                out.push_str(&definition.to_source(false));
            }
            Self::ModuleDecl { name, members } => {
                out.push_str("declare module ");
                out.push_str(name); // TODO: escape?
                out.push_str(" {\n");
                for member in members {
                    out.push_str("  ");
                    out.push_str(&member.to_source(true).replace('\n', "\n  "));
                    out.push('\n'); // TODO: separate by two newlines
                }
                out.push('}');
            }
            Self::NamespaceDecl { name, members } => {
                out.push_str("namespace ");
                out.push_str(name); // TODO: escape?
                out.push_str(" {\n");
                for member in members {
                    out.push_str("  ");
                    out.push_str(&member.to_source(true).replace('\n', "\n  "));
                    out.push('\n'); // TODO: separate by two newlines
                }
                out.push('}');
            }
            Self::NamedFunctionDecl { name, function } => {
                out.push_str("function ");
                out.push_str(name); // TODO: escape?
                out.push_str(&function.to_source(true));
            }
        }

        out
    }

    pub fn new_object(properties: BTreeMap<&str, TSType>) -> Self {
        Self::Object {
            properties: properties
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect(),
            nullable: false,
        }
    }

    pub fn new_singleton_object(key: &str, value: TSType) -> Self {
        Self::new_object(BTreeMap::from([(key, value)]))
    }

    pub fn new_neverobject() -> Self {
        Self::NeverObject
    }

    pub fn new_function(args: BTreeMap<&str, TSType>, returning: TSType) -> Self {
        Self::Function {
            args: args.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
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
                            Self::Scalar {
                                value: "HTMLElement",
                                nullable: false,
                            },
                        ),
                        ("flags", flags),
                    ])),
                )]),
                Self::new_void(),
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
            name: name.to_owned(),
            members,
        }
    }

    pub fn new_namespace(name: &str, members: Vec<Self>) -> Self {
        Self::NamespaceDecl {
            name: name.to_owned(),
            members,
        }
    }

    pub fn new_named_function(name: &str, function: TSType) -> Self {
        Self::NamedFunctionDecl {
            name: name.to_owned(),
            function: Box::from(function),
        }
    }

    pub fn into_init(self) -> Self {
        Self::new_init(self)
    }

    pub fn into_typedecl(self, name: &str) -> Self {
        Self::TypeDecl {
            name: name.to_owned(),
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

    #[test]
    fn interprets_int8() {
        let schema = from_json(json!({"type": "int8"}));

        assert_eq!(
            TSType::from_schema(schema),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_int16() {
        let schema = from_json(json!({"type": "int16"}));

        assert_eq!(
            TSType::from_schema(schema),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_int32() {
        let schema = from_json(json!({"type": "int32"}));

        assert_eq!(
            TSType::from_schema(schema),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_uint8() {
        let schema = from_json(json!({"type": "uint8"}));

        assert_eq!(
            TSType::from_schema(schema),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_uint16() {
        let schema = from_json(json!({"type": "uint16"}));

        assert_eq!(
            TSType::from_schema(schema),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_uint32() {
        let schema = from_json(json!({"type": "uint32"}));

        assert_eq!(
            TSType::from_schema(schema),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_float32() {
        let schema = from_json(json!({"type": "float32"}));

        assert_eq!(
            TSType::from_schema(schema),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_float64() {
        let schema = from_json(json!({"type": "float64"}));

        assert_eq!(
            TSType::from_schema(schema),
            TSType::Scalar {
                value: "number",
                nullable: false
            }
        );
    }

    #[test]
    fn interprets_string() {
        let schema = from_json(json!({"type": "string"}));

        let type_ = TSType::from_schema(schema);

        assert_eq!(type_.to_source(true), "string".to_string())
    }

    #[test]
    fn interprets_boolean() {
        let schema = from_json(json!({"type": "boolean"}));

        let type_ = TSType::from_schema(schema);

        assert_eq!(type_.to_source(true), "bool".to_string())
    }

    #[test]
    fn interprets_object() {
        let schema = from_json(json!({
            "properties": {
                "a": { "type": "float32" }
            }
        }));

        let type_ = TSType::from_schema(schema);

        assert_eq!(type_.to_source(true), "{\n  a: number;\n}".to_string())
    }

    #[test]
    fn interprets_enum() {
        let schema = from_json(json!({"enum": ["a", "b"]}));

        let type_ = TSType::from_schema(schema);

        assert_eq!(type_.to_source(true), "\"a\" | \"b\"".to_string())
    }

    #[test]
    fn interprets_elements() {
        let schema = from_json(json!({"elements": {"type": "string"}}));

        let type_ = TSType::from_schema(schema);

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
    fn scalar_to_source() {
        let type_ = TSType::from_schema(from_json(json!({"type": "string"})));

        assert_eq!(type_.to_source(true), "string".to_string());
    }

    #[test]
    fn nullable_scalar_to_source() {
        let type_ = TSType::from_schema(from_json(json!({"type": "string", "nullable": true})));

        assert_eq!(type_.to_source(true), "string | null".to_string());
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
            type_.to_source(true),
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
            type_.to_source(false),
            "(one: number, two: string) => string".to_string()
        )
    }

    #[test]
    fn typedecl_to_source() {
        let type_ =
            TSType::from_schema(from_json(json!({"properties": {"a": {"type": "string"}}})))
                .into_typedecl("Flags");

        assert_eq!(
            type_.to_source(true),
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

        assert_eq!(method.to_source(true), "function init(): void".to_string());
    }

    #[test]
    fn module_to_source() {
        let namespace = TSType::new_module(
            "Elm",
            Vec::from([TSType::new_namespace("Main", Vec::new())]),
        );

        assert_eq!(
            namespace.to_source(true),
            "declare module Elm {\n  namespace Main {\n  }\n}".to_string()
        );
    }

    #[test]
    fn namespace_to_source() {
        let namespace = TSType::new_namespace("Main", Vec::from([]));

        assert_eq!(namespace.to_source(true), "namespace Main {\n}".to_string());
    }

    #[test]
    fn list_to_source() {
        let type_ = TSType::from_schema(from_json(json!({"elements": {"type": "string"}})));

        assert_eq!(type_.to_source(true), "string[]".to_string());
    }

    #[test]
    fn list_to_source_space() {
        let type_ = TSType::from_schema(from_json(json!({"elements": {"enum": ["a", "b"]}})));

        assert_eq!(type_.to_source(true), "(\"a\" | \"b\")[]".to_string());
    }
}
