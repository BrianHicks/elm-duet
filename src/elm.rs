use eyre::{bail, Result, WrapErr};
use jtd::Schema;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Scalar(&'static str),
    Maybe(Box<Type>),
    Unit,
    DictWithStringKeys(Box<Type>),
    List(Box<Type>),
    Ref(String),
    Record(BTreeMap<String, Type>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decl {
    CustomTypeEnum {
        name: String,
        cases: BTreeMap<String, String>,
    },
    TypeAlias {
        name: String,
        type_: Type,
    },
}

impl Type {
    pub fn from_schema(
        schema: Schema,
        name_suggestion: Option<String>,
    ) -> Result<(Self, Vec<Decl>)> {
        let mut is_nullable = false;
        let mut decls = Vec::new();

        let base = match schema {
            Schema::Empty { .. } => Self::Unit,
            Schema::Ref {
                definitions: _,
                metadata: _,
                nullable: _,
                ref_: _,
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
                        cases.insert(value.to_uppercase(), value.to_string());
                    }

                    decls.push(Decl::CustomTypeEnum {
                        name: name.to_string(),
                        cases,
                    });

                    Self::Ref(name.to_string())
                }
                None => bail!("string names are required for enums"),
            },
            Schema::Elements {
                nullable, elements, ..
            } => {
                is_nullable = nullable;

                let (type_, sub_decls) =
                    Self::from_schema(*elements, name_suggestion.map(|n| format!("{n}Elements")))
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
                            Self::from_schema(field_schema, Some(field_name.clone()))
                                .wrap_err_with(|| {
                                    format!("could not convert the type of `{field_name}`")
                                })?;

                        decls.extend(field_decls);
                        fields.insert(field_name.to_string(), field_type);
                    }

                    decls.push(Decl::TypeAlias {
                        name: name.to_string(),
                        type_: Self::Record(fields),
                    });

                    Self::Ref(name.to_string())
                }
                None => bail!("string names are required for properties"),
            },
            Schema::Values {
                nullable, values, ..
            } => {
                is_nullable = nullable;

                let (type_, sub_decls) =
                    Self::from_schema(*values, name_suggestion.map(|n| format!("{n}Values")))
                        .wrap_err("could not convert elements of a list")?;

                decls.extend(sub_decls);

                Self::DictWithStringKeys(Box::new(type_))
            }
            Schema::Discriminator {
                definitions: _,
                metadata: _,
                nullable: _,
                discriminator: _,
                mapping: _,
            } => todo!(),
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
}

// pub struct Module {
//     name: Vec<String>,
//     defs: Vec<()>,
// }
//
// impl Module {
//     pub fn to_source(&self) -> Result<String> {
//         if self.defs.is_empty() {
//             eyre::bail!(
//                 "Module {} didn't contain any definitions",
//                 self.name.join(".")
//             )
//         }
//
//         Ok(String::new())
//     }
// }

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
            Type::from_schema(from_json(value), None).expect("valid schema from JSON value")
        }

        #[test]
        fn interprets_int8() {
            let (type_, _) = from_schema(json!({"type": "int8"}));

            assert_eq!(type_, Type::Scalar("Int"));
        }

        #[test]
        fn interprets_int16() {
            let (type_, _) = from_schema(json!({"type": "int16"}));

            assert_eq!(type_, Type::Scalar("Int"));
        }

        #[test]
        fn interprets_int32() {
            let (type_, _) = from_schema(json!({"type": "int32"}));

            assert_eq!(type_, Type::Scalar("Int"));
        }

        #[test]
        fn interprets_uint8() {
            let (type_, _) = from_schema(json!({"type": "uint8"}));

            assert_eq!(type_, Type::Scalar("Int"));
        }

        #[test]
        fn interprets_uint16() {
            let (type_, _) = from_schema(json!({"type": "uint16"}));

            assert_eq!(type_, Type::Scalar("Int"));
        }

        #[test]
        fn interprets_uint32() {
            let (type_, _) = from_schema(json!({"type": "uint32"}));

            assert_eq!(type_, Type::Scalar("Int"));
        }

        #[test]
        fn interprets_float32() {
            let (type_, _) = from_schema(json!({"type": "float32"}));

            assert_eq!(type_, Type::Scalar("Float"));
        }

        #[test]
        fn interprets_float64() {
            let (type_, _) = from_schema(json!({"type": "float64"}));

            assert_eq!(type_, Type::Scalar("Float"));
        }

        #[test]
        fn interprets_string() {
            let (type_, _) = from_schema(json!({"type": "string"}));

            assert_eq!(type_, Type::Scalar("String"));
        }

        #[test]
        fn interprets_timestamp() {
            let (type_, _) = from_schema(json!({"type": "timestamp"}));

            assert_eq!(type_, Type::Scalar("String"));
        }

        #[test]
        fn interprets_boolean() {
            let (type_, _) = from_schema(json!({"type": "boolean"}));

            assert_eq!(type_, Type::Scalar("Bool"));
        }

        #[test]
        fn interprets_nullable_type() {
            let (type_, _) = from_schema(json!({"type": "string", "nullable": true}));

            assert_eq!(type_, Type::Maybe(Box::new(Type::Scalar("String"))));
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

            assert_eq!(
                type_,
                Type::DictWithStringKeys(Box::new(Type::Scalar("String")))
            );
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
                Type::Maybe(Box::new(Type::DictWithStringKeys(Box::new(Type::Scalar(
                    "String"
                )))))
            );
        }

        #[test]
        fn interprets_elements() {
            let (type_, _) = from_schema(json!({
                "elements": {
                    "type": "string",
                },
            }));

            assert_eq!(type_, Type::List(Box::new(Type::Scalar("String"))));
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
                Type::Maybe(Box::new(Type::List(Box::new(Type::Scalar("String")))))
            );
        }

        #[test]
        fn interprets_enum_no_name() {
            let err = Type::from_schema(
                from_json(json!({
                    "enum": ["a", "b"],
                })),
                None,
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

            assert_eq!(type_, Type::Ref("Foo".to_string()));

            assert_eq!(
                decls,
                Vec::from([Decl::CustomTypeEnum {
                    name: "Foo".to_string(),
                    cases: BTreeMap::from([
                        ("A".to_string(), "a".to_string()),
                        ("B".to_string(), "b".to_string()),
                    ]),
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

            assert_eq!(type_, Type::Ref("Foo".to_string()));

            assert_eq!(
                decls,
                Vec::from([Decl::TypeAlias {
                    name: "Foo".to_string(),
                    type_: Type::Record(BTreeMap::from([
                        ("a".to_string(), Type::Unit),
                        ("b".to_string(), Type::Unit),
                    ])),
                }])
            );
        }
    }

    // mod module {
    //     use super::*;
    //
    //     #[test]
    //     fn error_on_no_defs_to_source() {
    //         let m = Module {
    //             name: Vec::from(["A".to_string(), "B".to_string()]),
    //             defs: Vec::new(),
    //         };
    //
    //         let err = m.to_source().unwrap_err();
    //
    //         assert_eq!(
    //             err.to_string(),
    //             "Module A.B didn't contain any definitions".to_string()
    //         )
    //     }
    // }
}
