use color_eyre::{Help, SectionExt};
use eyre::{eyre, Result, WrapErr};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct Formatter {
    name: String,
    command: PathBuf,
}

impl Formatter {
    pub fn discover(binary_name: &str) -> Result<Option<Self>> {
        // first look in PATH
        for source in std::env::var("PATH")?.split(":") {
            let command = PathBuf::from(source).join(binary_name);
            if command.exists() {
                return Ok(Some(Self {
                    name: binary_name.to_owned(),
                    command,
                }));
            }
        }

        // then search for node_modules up the cwd tree
        let cwd = std::env::current_dir()?;
        let mut search = Some(cwd.as_path());
        loop {
            match search {
                Some(dir) => {
                    let command = dir.join("node_modules").join(".bin").join(binary_name);
                    if command.exists() {
                        return Ok(Some(Self {
                            name: binary_name.to_owned(),
                            command,
                        }));
                    }

                    search = dir.parent()
                }

                None => break,
            }
        }

        Ok(None)
    }

    pub(crate) fn format(&self, args: &[&str], files: &[&Path]) -> Result<()> {
        let process = Command::new(&self.command)
            .args(args)
            .args(files)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .wrap_err_with(|| format!("could not start `{}`", self.name))?;

        let out = process
            .wait_with_output()
            .wrap_err_with(|| format!("could not get output for `{}`", self.name))?;

        if !out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            return Err(eyre!("cmd exited with non-zero status code"))
                .with_section(move || stdout.trim().to_string().header("Stdout:"))
                .with_section(move || stderr.trim().to_string().header("Stderr:"));
        }

        Ok(())
    }
}
