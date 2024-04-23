use crate::elm;
use crate::typescript::NamespaceBuilder;
use crate::typescript::TSType;
use color_eyre::Result;
use eyre::WrapErr;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

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

    fn globals(&self) -> Result<BTreeMap<String, jtd::Schema>> {
        let mut out = BTreeMap::new();
        for (name, serde_schema) in &self.definitions {
            out.insert(
                name.clone(),
                jtd::Schema::from_serde_schema(serde_schema.clone()).wrap_err_with(|| {
                    format!("could not interpret JTD schema for the {name} definition")
                })?,
            );
        }

        Ok(out)
    }

    pub fn flags_to_ts(&self) -> Result<String> {
        let mut builder = NamespaceBuilder::root("Elm");

        let globals = self.globals()?;

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
                        &globals,
                    )
                    .wrap_err("could not convert flags")?
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
                            &globals,
                        )
                        .wrap_err_with(|| format!("could not convert port {name}"))?;

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

    pub fn to_elm(&self) -> Result<BTreeMap<PathBuf, String>> {
        let globals = self.globals()?;
        let mut modules = BTreeMap::new();

        for (name, module) in &self.modules {
            let name_base: Vec<String> = name.split(".").map(|s| s.to_owned()).collect();

            // generate flags
            if let Some(flags) = &module.flags {
                let mut flags_name: Vec<String> = Vec::with_capacity(name_base.len() + 1);
                flags_name.extend(name_base.clone());
                flags_name.push("Flags".into());

                let flags_module = elm::Module::from_schema(
                    flags_name,
                    jtd::Schema::from_serde_schema(flags.clone())?,
                    Some("Flags".to_string()),
                    &globals,
                )
                .wrap_err("could not convert flags type to Elm module")?;

                modules.insert(
                    format!("{}.elm", flags_module.name.join("/")).into(),
                    flags_module
                        .to_source()
                        .wrap_err("could not get source for {flags_name}")?,
                );
            };

            // generate ports
        }

        Ok(modules)
    }
}
