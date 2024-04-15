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
        let mut builder = NamespaceBuilder::root("Elm");

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
                    .into_typedecl("Flags"),
                )?,
                None => builder.insert(
                    &module_path,
                    TSType::new_neverobject().into_typedecl("Flags"),
                )?,
            }

            match &module.ports {
                Some(ports) => {
                    let mut port_keys = BTreeMap::new();

                    for (name, value) in ports {
                        let type_ = TSType::from_schema(
                            jtd::Schema::from_serde_schema(value.schema.clone()).wrap_err_with(
                                || format!("could not interpret JTD schema for port {name}"),
                            )?,
                        );

                        let func_record = match value.metadata.direction {
                            PortDirection::JsToElm => TSType::new_singleton_object(
                                "send",
                                TSType::new_send_function(type_),
                            ),
                            PortDirection::ElmToJs => TSType::new_singleton_object(
                                "subscribe",
                                TSType::new_subscribe_function(type_),
                            ),
                        };

                        port_keys.insert(name.as_str(), func_record);
                    }

                    builder.insert(
                        &module_path,
                        TSType::new_object(port_keys).into_typedecl("Ports"),
                    )?
                }
                None => builder.insert(
                    &module_path,
                    TSType::new_neverobject().into_typedecl("Ports"),
                )?,
            }

            builder.insert(&module_path, TSType::new_ref("Flags").into_init())?;
        }

        Ok(format!(
            "// Warning: this file is automatically generated. Don't edit by hand!\n\n{}",
            builder.into_tstype().to_source(true)
        ))
    }
}
