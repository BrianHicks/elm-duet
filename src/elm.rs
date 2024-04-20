use eyre::{Result, WrapErr};
use jtd::Schema;

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Scalar(&'static str),
    Maybe(Box<Type>),
    Unit,
    DictWithStringKeys(Box<Type>),
}

impl Type {
    fn from_schema(schema: Schema) -> Result<Self> {
        let mut is_nullable = false;

        let base = match schema {
            Schema::Empty { .. } => Self::Unit,
            Schema::Ref {
                definitions,
                metadata,
                nullable,
                ref_,
            } => todo!(),
            Schema::Type {
                nullable, type_, ..
            } => {
                is_nullable = nullable;

                match type_ {
                    jtd::Type::Boolean => Self::Scalar("Bool"),
                    jtd::Type::Int8
                    | jtd::Type::Uint8
                    | jtd::Type::Int16
                    | jtd::Type::Uint16
                    | jtd::Type::Int32
                    | jtd::Type::Uint32 => Self::Scalar("Int"),
                    jtd::Type::Float32 | jtd::Type::Float64 => Self::Scalar("Float"),
                    jtd::Type::String | jtd::Type::Timestamp => Self::Scalar("String"),
                }
            }
            Schema::Enum {
                definitions,
                metadata,
                nullable,
                enum_,
            } => todo!(),
            Schema::Elements {
                definitions,
                metadata,
                nullable,
                elements,
            } => todo!(),
            Schema::Properties {
                definitions,
                metadata,
                nullable,
                properties,
                optional_properties,
                properties_is_present,
                additional_properties,
            } => todo!(),
            Schema::Values {
                nullable, values, ..
            } => {
                is_nullable = nullable;

                Self::DictWithStringKeys(Box::new(
                    Self::from_schema(*values)
                        .wrap_err("could not interpret a type for the values of the type")?,
                ))
            }
            Schema::Discriminator {
                definitions,
                metadata,
                nullable,
                discriminator,
                mapping,
            } => todo!(),
        };

        if is_nullable {
            Ok(Self::Maybe(Box::new(base)))
        } else {
            Ok(base)
        }
    }
}

pub struct Module {
    name: Vec<String>,
    defs: Vec<()>,
}

impl Module {
    pub fn to_source(&self) -> Result<String> {
        if self.defs.is_empty() {
            eyre::bail!(
                "Module {} didn't contain any definitions",
                self.name.join(".")
            )
        }

        Ok(String::new())
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

        fn from_schema(value: Value) -> Type {
            Type::from_schema(from_json(value)).expect("valid schema from JSON value")
        }

        #[test]
        fn interprets_int8() {
            let type_ = from_schema(json!({"type": "int8"}));

            assert_eq!(type_, Type::Scalar("Int"));
        }

        #[test]
        fn interprets_int16() {
            let type_ = from_schema(json!({"type": "int16"}));

            assert_eq!(type_, Type::Scalar("Int"));
        }

        #[test]
        fn interprets_int32() {
            let type_ = from_schema(json!({"type": "int32"}));

            assert_eq!(type_, Type::Scalar("Int"));
        }

        #[test]
        fn interprets_uint8() {
            let type_ = from_schema(json!({"type": "uint8"}));

            assert_eq!(type_, Type::Scalar("Int"));
        }

        #[test]
        fn interprets_uint16() {
            let type_ = from_schema(json!({"type": "uint16"}));

            assert_eq!(type_, Type::Scalar("Int"));
        }

        #[test]
        fn interprets_uint32() {
            let type_ = from_schema(json!({"type": "uint32"}));

            assert_eq!(type_, Type::Scalar("Int"));
        }

        #[test]
        fn interprets_float32() {
            let type_ = from_schema(json!({"type": "float32"}));

            assert_eq!(type_, Type::Scalar("Float"));
        }

        #[test]
        fn interprets_float64() {
            let type_ = from_schema(json!({"type": "float64"}));

            assert_eq!(type_, Type::Scalar("Float"));
        }

        #[test]
        fn interprets_string() {
            let type_ = from_schema(json!({"type": "string"}));

            assert_eq!(type_, Type::Scalar("String"));
        }

        #[test]
        fn interprets_timestamp() {
            let type_ = from_schema(json!({"type": "timestamp"}));

            assert_eq!(type_, Type::Scalar("String"));
        }

        #[test]
        fn interprets_boolean() {
            let type_ = from_schema(json!({"type": "boolean"}));

            assert_eq!(type_, Type::Scalar("Bool"));
        }

        #[test]
        fn interprets_nullable_type() {
            let type_ = from_schema(json!({"type": "string", "nullable": true}));

            assert_eq!(type_, Type::Maybe(Box::new(Type::Scalar("String"))));
        }

        #[test]
        fn interprets_empty() {
            let type_ = from_schema(json!({}));

            assert_eq!(type_, Type::Unit);
        }

        #[test]
        fn interprets_values() {
            let type_ = from_schema(json!({
                "values": {
                    "type": "string",
                },
            }));

            assert_eq!(
                type_,
                Type::DictWithStringKeys(Box::new(Type::Scalar("String")))
            );
        }

        #[test]
        fn interprets_nullable_values() {
            let type_ = from_schema(json!({
                "values": {
                    "type": "string",
                },
                "nullable": true,
            }));

            assert_eq!(
                type_,
                Type::Maybe(Box::new(Type::DictWithStringKeys(Box::new(Type::Scalar(
                    "String"
                )))))
            );
        }
    }

    mod module {
        use super::*;

        #[test]
        fn error_on_no_defs_to_source() {
            let m = Module {
                name: Vec::from(["A".to_string(), "B".to_string()]),
                defs: Vec::new(),
            };

            let err = m.to_source().unwrap_err();

            assert_eq!(
                err.to_string(),
                "Module A.B didn't contain any definitions".to_string()
            )
        }
    }
}
