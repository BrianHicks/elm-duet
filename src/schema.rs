use crate::typescript;
use color_eyre::Result;
use eyre::WrapErr;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Schema {
    #[serde(default)]
    pub definitions: BTreeMap<String, jtd::SerdeSchema>,
    pub flags: Option<jtd::SerdeSchema>,
}

impl Schema {
    pub fn from_fs(path: &Path) -> Result<Schema> {
        let bytes = std::fs::read(path).wrap_err_with(|| format!("could not read {path:?}"))?;
        serde_json::from_slice(&bytes)
            .wrap_err_with(|| format!("could not read schema from {path:?}"))
    }

    pub fn flags_to_ts(&self) -> Result<String> {
        let mut buffer = String::new();

        if let Some(flags_serde) = &self.flags {
            let flags = jtd::Schema::from_serde_schema(flags_serde.clone())
                .wrap_err("could not interpret JTD schema for flags")?;

            buffer.push_str(&typescript::TSType::from_schema(flags).to_source())
        }

        Ok(buffer)
    }
}
