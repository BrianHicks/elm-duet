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
    TypeDecl {
        name: String,
        definition: Box<TSType>,
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
                out.push('\n');
            }
        }

        out
    }

    fn new_function(args: BTreeMap<String, TSType>, returning: TSType) -> Self {
        Self::Function {
            args,
            returning: Box::new(returning),
        }
    }

    fn new_init(flags: TSType) -> Self {
        Self::new_function(
            BTreeMap::from([(
                "config".to_string(),
                Self::Object(BTreeMap::from([
                    ("node".to_string(), Self::Scalar("HTMLElement")),
                    ("flags".to_string(), flags),
                ])),
            )]),
            Self::Scalar("void"),
        )
    }

    pub fn new_ref(name: String) -> Self {
        Self::TypeRef(name)
    }

    pub fn to_init(self) -> Self {
        Self::new_init(self)
    }

    pub fn to_typedecl(self, name: String) -> Self {
        Self::TypeDecl {
            name,
            definition: Box::from(self),
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
    fn new_function() {
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
    fn to_typedecl() {
        let type_ =
            TSType::from_schema(from_json(json!({"properties": {"a": {"type": "string"}}})))
                .to_typedecl("Flags".to_string());

        assert_eq!(
            type_.to_source(),
            "type Flags = {\n  a: string;\n}\n".to_string(),
        )
    }
}
