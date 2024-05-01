use crate::elm;
use crate::typescript::NamespaceBuilder;
use crate::typescript::{FieldPresence, TSType};
use color_eyre::Result;
use eyre::{bail, WrapErr};
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

        match path.extension().and_then(std::ffi::OsStr::to_str) {
            Some("json") => serde_json::from_slice(&bytes)
                .wrap_err_with(|| format!("could not read schema from {path:?}")),
            Some("yaml") => serde_yaml::from_slice(&bytes)
                .wrap_err_with(|| format!("could not read schema from {path:?}")),
            Some(_) => bail!(
                "I can't deserialize a schema from a {:?} file",
                path.extension()
            ),
            None => bail!(
                "I couldn't figure out what kind of file {} is because it doesn't have an extension!",
                path.display(),
            ),
        }
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

    // TODO: audit how much work this does and consider moving responsibility into the TS module
    pub fn to_ts(&self) -> Result<String> {
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
                                FieldPresence::Required,
                            ),
                            PortDirection::ElmToJs => TSType::new_singleton_object(
                                "subscribe",
                                TSType::new_subscribe_function(type_),
                                FieldPresence::Required,
                            ),
                        };

                        // if a port is defined in Elm but not hooked up, Elm will omit it. That
                        // means this could be optional and we need to deal with that.
                        port_keys.insert(name.as_str(), (func_record, FieldPresence::Optional));
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
            builder.into_tstype().to_source(true)?
        ))
    }

    pub fn to_elm(&self) -> Result<BTreeMap<PathBuf, String>> {
        let globals = self.globals()?;
        let mut files = BTreeMap::new();

        for (name, module) in &self.modules {
            let name_base: Vec<String> = name.split('.').map(|s| s.to_owned()).collect();

            // generate flags
            if let Some(flags) = &module.flags {
                let mut flags_name: Vec<String> = Vec::with_capacity(name_base.len() + 1);
                flags_name.extend(name_base.clone());
                flags_name.push("Flags".into());

                let mut flags_module = elm::Module::new(flags_name);
                flags_module
                    .insert_from_schema(
                        jtd::Schema::from_serde_schema(flags.clone())?,
                        Some("Flags".to_string()),
                        &globals,
                    )
                    .wrap_err("could not convert flags type to Elm module")?;

                files.insert(
                    format!("{}.elm", flags_module.name.join("/")).into(),
                    flags_module.to_source().wrap_err("could not get source")?,
                );
            };

            // generate ports
            if let Some(ports) = &module.ports {
                let mut ports_name: Vec<String> = Vec::with_capacity(name_base.len() + 1);
                ports_name.extend(name_base.clone());
                ports_name.push("Ports".into());

                let mut ports_module = elm::Module::new(ports_name);

                for (port, port_schema) in ports {
                    let port_type = ports_module
                        .insert_from_schema(
                            jtd::Schema::from_serde_schema(port_schema.schema.clone())?,
                            Some(port.into()),
                            &globals,
                        )
                        .wrap_err_with(|| format!("could not convert the `{port}` port to Elm"))?;

                    ports_module.insert_port(elm::Port::new(
                        port.to_owned(),
                        match port_schema.metadata.direction {
                            PortDirection::ElmToJs => elm::PortDirection::Send,
                            PortDirection::JsToElm => elm::PortDirection::Subscribe,
                        },
                        port_type,
                    ))
                }

                files.insert(
                    format!("{}.elm", ports_module.name.join("/")).into(),
                    ports_module.to_source().wrap_err("could not get source")?,
                );
            }
        }

        Ok(files)
    }
}
