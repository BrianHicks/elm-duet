use color_eyre::Result;
use eyre::WrapErr;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Schema {
    pub flags: jtd::SerdeSchema,
}

impl Schema {
    pub fn from_fs(path: &Path) -> Result<Schema> {
        let bytes = std::fs::read(path).wrap_err_with(|| format!("could not read {path:?}"))?;
        serde_json::from_slice(&bytes)
            .wrap_err_with(|| format!("could not read schema from {path:?}"))
    }
}
