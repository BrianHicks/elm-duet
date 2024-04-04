use jtd::{Schema, Type};
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum TSType {
    Object(BTreeMap<String, TSType>),
    Scalar(&'static str),
    StringScalar(String),
    Union(Vec<TSType>),
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
                _ => todo!("scalar: {type_:#?}"),
            }),
            Schema::Enum { enum_, .. } => Self::Union(
                enum_
                    .into_iter()
                    .map(|value| Self::StringScalar(value))
                    .collect(),
            ),
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
            Self::Union(types) => {
                for (i, type_) in types.iter().enumerate() {
                    if i != 0 {
                        out.push_str(" | ");
                    }
                    out.push_str(&type_.to_source())
                }
            }
        }

        out
    }
}
