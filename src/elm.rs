use crate::inflected_string::InflectedString;
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
    Record(BTreeMap<InflectedString, Type>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decl {
    CustomTypeEnum {
        name: InflectedString,
        cases: BTreeMap<InflectedString, Option<Type>>,
    },
    TypeAlias {
        name: InflectedString,
        type_: Type,
    },
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
                        cases.insert(value.into(), None);
                    }

                    decls.push(Decl::CustomTypeEnum {
                        name: name.into(),
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

                    Self::Ref(name.to_string())
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
                discriminator: _,
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
                        cases,
                    });

                    Self::Ref(name.to_string())
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
            Type::from_schema(from_json(value), None, &BTreeMap::new())
                .expect("valid schema from JSON value")
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

            assert_eq!(type_, Type::Ref("Foo".to_string()));

            assert_eq!(
                decls,
                Vec::from([Decl::CustomTypeEnum {
                    name: "Foo".into(),
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

            assert_eq!(type_, Type::Ref("Foo".to_string()));

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

            assert_eq!(type_, Type::Ref("Foo".to_string()));

            assert_eq!(
                decls,
                Vec::from([
                    Decl::TypeAlias {
                        name: "a".into(),
                        type_: Type::Record(BTreeMap::from([(
                            "value".into(),
                            Type::Scalar("String")
                        )]))
                    },
                    Decl::TypeAlias {
                        name: "b".into(),
                        type_: Type::Record(BTreeMap::from([(
                            "value".into(),
                            Type::Scalar("Float")
                        )]))
                    },
                    Decl::CustomTypeEnum {
                        name: "Foo".into(),
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

            assert_eq!(type_, Type::Scalar("String"));
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

            assert_eq!(type_, Type::Scalar("String"));
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
