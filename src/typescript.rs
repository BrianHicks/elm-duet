use color_eyre::Result;
use jtd::{Schema, Type};
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum TSType {
    Object(BTreeMap<String, TSType>),
    Scalar(&'static str),
    StringScalar(String),
    TypeRef(String),
    Union(Vec<TSType>),
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
    ClassDecl {
        name: String,
        members: Vec<TSType>,
    },
    ClassProperty {
        name: String,
        definition: Box<TSType>,
    },
    NamedFunctionDecl {
        name: String,
        function: Box<TSType>, // in practice, should always be a `Function`
    },
}

impl TSType {
    pub fn from_schema(schema: Schema) -> Self {
        match schema {
            Schema::Properties { properties, .. } => Self::Object(
                properties
                    .into_iter()
                    .map(|(name, value)| (name, Self::from_schema(value)))
                    .collect(),
            ),
            Schema::Type { type_, .. } => Self::Scalar(match type_ {
                Type::Float32 => "number",
                Type::String => "string",
                _ => todo!("scalar: {type_:#?}"),
            }),
            Schema::Enum { enum_, .. } => {
                Self::Union(enum_.into_iter().map(Self::StringScalar).collect())
            }
            _ => todo!("{:#?}", schema),
        }
    }

    pub fn to_source(&self) -> String {
        let mut out = String::new();

        match self {
            Self::Object(fields) => {
                out.push_str("{\n");
                for (name, value) in fields {
                    out.push_str("  ");
                    out.push_str(name); // TODO: escape?
                    out.push_str(": ");
                    out.push_str(&value.to_source());
                    out.push_str(";\n");
                }
                out.push('}');
            }
            Self::Scalar(literal) => out.push_str(literal),
            Self::StringScalar(string) => {
                out.push('"');
                out.push_str(string);
                out.push('"');
            }
            Self::TypeRef(ref_) => out.push_str(ref_),
            Self::Union(types) => {
                for (i, type_) in types.iter().enumerate() {
                    if i != 0 {
                        out.push_str(" | ");
                    }
                    out.push_str(&type_.to_source())
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
                    out.push_str(&type_.to_source());
                }
                out.push_str("): ");
                out.push_str(&returning.to_source());
            }
            Self::TypeDecl { name, definition } => {
                out.push_str("type ");
                out.push_str(name); // TODO: escape?
                out.push_str(" = ");
                out.push_str(&definition.to_source());
            }
            Self::ModuleDecl { name, members } => {
                out.push_str("declare module ");
                out.push_str(name); // TODO: escape?
                out.push_str(" {\n");
                for member in members {
                    out.push_str("  ");
                    out.push_str(&member.to_source().replace('\n', "\n  "));
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
                    out.push_str(&member.to_source().replace('\n', "\n  "));
                    out.push('\n'); // TODO: separate by two newlines
                }
                out.push('}');
            }
            Self::ClassDecl { name, members } => {
                out.push_str("class ");
                out.push_str(name); // TODO: escape?
                out.push_str(" {\n");
                for member in members {
                    out.push_str("  ");
                    out.push_str(&member.to_source().replace('\n', "\n  "));
                    out.push('\n'); // TODO: separate by two newlines
                }
                out.push('}');
            }
            Self::ClassProperty { name, definition } => {
                out.push_str(name); // TODO: escape?
                out.push_str(": ");
                out.push_str(&definition.to_source());
            }
            Self::NamedFunctionDecl { name, function } => {
                out.push_str("function ");
                out.push_str(name); // TODO: escape?
                out.push_str(&function.to_source());
            }
        }

        out
    }

    pub fn new_object(properties: BTreeMap<String, TSType>) -> Self {
        Self::Object(properties)
    }

    fn new_function(args: BTreeMap<String, TSType>, returning: TSType) -> Self {
        Self::Function {
            args,
            returning: Box::new(returning),
        }
    }

    fn new_init(flags: TSType) -> Self {
        Self::new_named_function(
            "init".to_string(),
            Self::new_function(
                BTreeMap::from([(
                    "config".to_string(),
                    Self::Object(BTreeMap::from([
                        ("node".to_string(), Self::Scalar("HTMLElement")),
                        ("flags".to_string(), flags),
                    ])),
                )]),
                Self::Scalar("void"),
            ),
        )
    }

    pub fn new_ref(name: String) -> Self {
        Self::TypeRef(name)
    }

    fn new_module(name: String, members: Vec<TSType>) -> Self {
        Self::ModuleDecl { name, members }
    }

    pub fn new_namespace(name: String, members: Vec<Self>) -> Self {
        Self::NamespaceDecl { name, members }
    }

    pub fn new_class(name: String, members: Vec<Self>) -> Self {
        Self::ClassDecl { name, members }
    }

    pub fn new_named_function(name: String, function: TSType) -> Self {
        Self::NamedFunctionDecl {
            name,
            function: Box::from(function),
        }
    }

    pub fn into_init(self) -> Self {
        Self::new_init(self)
    }

    pub fn into_typedecl(self, name: String) -> Self {
        Self::TypeDecl {
            name,
            definition: Box::from(self),
        }
    }

    pub fn into_class_property(self, name: String) -> TSType {
        Self::ClassProperty {
            name,
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
    pub fn root(name: String) -> Self {
        Self::Root {
            name,
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
                name,
                below.into_iter().map(|(_, v)| v.into_tstype()).collect(),
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
                        .into_iter()
                        .map(|(_, v)| v.into_tstype())
                        .collect::<Vec<TSType>>(),
                );

                TSType::new_namespace(name, ts_members)
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
    fn interprets_float32() {
        let schema = from_json(json!({"type": "float32"}));

        let type_ = TSType::from_schema(schema);

        assert_eq!(type_.to_source(), "number".to_string())
    }

    #[test]
    fn interprets_string() {
        let schema = from_json(json!({"type": "string"}));

        let type_ = TSType::from_schema(schema);

        assert_eq!(type_.to_source(), "string".to_string())
    }

    #[test]
    fn interprets_object() {
        let schema = from_json(json!({
            "properties": {
                "a": { "type": "float32" }
            }
        }));

        let type_ = TSType::from_schema(schema);

        assert_eq!(type_.to_source(), "{\n  a: number;\n}".to_string())
    }

    #[test]
    fn interprets_enum() {
        let schema = from_json(json!({"enum": ["a", "b"]}));

        let type_ = TSType::from_schema(schema);

        assert_eq!(type_.to_source(), "\"a\" | \"b\"".to_string())
    }

    #[test]
    fn function_to_source() {
        let type_ = TSType::new_function(
            BTreeMap::from([
                ("one".to_string(), TSType::Scalar("number")),
                ("two".to_string(), TSType::Scalar("string")),
            ]),
            TSType::Scalar("string"),
        );

        assert_eq!(
            type_.to_source(),
            "(one: number, two: string): string".to_string()
        )
    }

    #[test]
    fn typedecl_to_source() {
        let type_ =
            TSType::from_schema(from_json(json!({"properties": {"a": {"type": "string"}}})))
                .into_typedecl("Flags".to_string());

        assert_eq!(
            type_.to_source(),
            "type Flags = {\n  a: string;\n}".to_string(),
        )
    }

    #[test]
    fn method_to_source() {
        let method = TSType::new_named_function(
            "init".to_string(),
            TSType::new_function(BTreeMap::new(), TSType::Scalar("void")),
        );

        assert_eq!(method.to_source(), "function init(): void".to_string());
    }

    #[test]
    fn class_to_source() {
        let class = TSType::new_class(
            "Main".to_string(),
            Vec::from([TSType::new_named_function(
                "init".to_string(),
                TSType::new_function(BTreeMap::new(), TSType::Scalar("void")),
            )]),
        );

        assert_eq!(
            class.to_source(),
            "class Main {\n  function init(): void\n}".to_string()
        );
    }

    #[test]
    fn class_property_to_source() {
        let class = TSType::new_class(
            "Main".to_string(),
            Vec::from([
                TSType::new_ref("Flags".to_string()).into_class_property("flags".to_string())
            ]),
        );

        assert_eq!(
            class.to_source(),
            "class Main {\n  flags: Flags\n}".to_string()
        );
    }

    #[test]
    fn module_to_source() {
        let namespace = TSType::new_module(
            "Elm".to_string(),
            Vec::from([TSType::new_namespace("Main".to_string(), Vec::new())]),
        );

        assert_eq!(
            namespace.to_source(),
            "declare module Elm {\n  namespace Main {\n  }\n}".to_string()
        );
    }

    #[test]
    fn namespace_to_source() {
        let namespace = TSType::new_namespace("Main".to_string(), Vec::from([]));

        assert_eq!(namespace.to_source(), "namespace Main {\n}".to_string());
    }
}
