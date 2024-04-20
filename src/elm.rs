use eyre::Result;

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
