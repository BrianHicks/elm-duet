use crate::typescript::NamespaceBuilder;
use crate::typescript::TSType;
use color_eyre::Result;
use eyre::WrapErr;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Schema {
    #[serde(default)]
    pub definitions: BTreeMap<String, jtd::SerdeSchema>,
    pub modules: BTreeMap<String, Module>,
}

#[derive(Debug, Deserialize)]
pub struct Module {
    pub flags: Option<jtd::SerdeSchema>,
    pub ports: Option<BTreeMap<String, Port>>,
}

#[derive(Debug, Deserialize)]
pub struct Port {
    metadata: PortMeta,

    #[serde(flatten)]
    pub schema: jtd::SerdeSchema,
}

#[derive(Debug, Deserialize)]
pub struct PortMeta {
    direction: PortDirection,
}

#[derive(Debug, Deserialize)]
pub enum PortDirection {
    JsToElm,
    ElmToJs,
}

impl Schema {
    pub fn from_fs(path: &Path) -> Result<Schema> {
        let bytes = std::fs::read(path).wrap_err_with(|| format!("could not read {path:?}"))?;
        serde_json::from_slice(&bytes)
            .wrap_err_with(|| format!("could not read schema from {path:?}"))
    }

    pub fn flags_to_ts(&self) -> Result<String> {
        let mut builder = NamespaceBuilder::root("Elm".to_string());

        for (module_name, module) in &self.modules {
            let module_path: Vec<&str> = module_name.split('.').collect();

            match &module.flags {
                Some(flags_serde) => builder.insert(
                    &module_path,
                    TSType::from_schema(
                        jtd::Schema::from_serde_schema(flags_serde.clone()).wrap_err_with(
                            || {
                                format!(
                                    "could not interpret JTD schema for flags in the {} module",
                                    module_name
                                )
                            },
                        )?,
                    )
                    .into_typedecl("Flags".to_string()),
                )?,
                None => builder.insert(
                    &module_path,
                    TSType::new_neverobject().into_typedecl("Flags".to_string()),
                )?,
            }

            match &module.ports {
                Some(ports) => builder.insert(
                    &module_path,
                    TSType::new_object(
                        ports
                            .iter()
                            .map(|(name, value)| {
                                let type_ = TSType::from_schema(
                                    jtd::Schema::from_serde_schema(value.schema.clone())
                                        .wrap_err_with(|| {
                                            format!(
                                                "could not interpret JTD schema for port {name}"
                                            )
                                        })?,
                                );

                                let func_record = match value.metadata.direction {
                                    PortDirection::JsToElm => {
                                        TSType::new_object(BTreeMap::from([(
                                            "send".to_string(),
                                            TSType::new_function(
                                                BTreeMap::from([("value".to_string(), type_)]),
                                                TSType::new_void(),
                                            ),
                                        )]))
                                    }
                                    PortDirection::ElmToJs => {
                                        TSType::new_object(BTreeMap::from([(
                                            "subscribe".to_string(),
                                            TSType::new_function(
                                                BTreeMap::from([(
                                                    "callback".to_string(),
                                                    TSType::new_function(
                                                        BTreeMap::from([(
                                                            "value".to_string(),
                                                            type_,
                                                        )]),
                                                        TSType::new_void(),
                                                    ),
                                                )]),
                                                TSType::new_void(),
                                            ),
                                        )]))
                                    }
                                };

                                Ok((name.clone(), func_record))
                            })
                            .collect::<Result<BTreeMap<String, TSType>>>()?,
                    )
                    .into_typedecl("Ports".to_string()),
                )?,
                None => builder.insert(
                    &module_path,
                    TSType::new_neverobject().into_typedecl("Ports".to_string()),
                )?,
            }

            builder.insert(
                &module_path,
                TSType::new_ref("Flags".to_string()).into_init(),
            )?;
        }

        Ok(format!(
            "// Warning: this file is automatically generated. Don't edit by hand!\n\n{}",
            builder.into_tstype().to_source()
        ))
    }
}
