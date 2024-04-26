mod elm;
mod inflected_string;
mod schema;
mod typescript;

use clap::Parser;
use color_eyre::Result;
use eyre::WrapErr;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(version, about)]
struct Cli {
    /// Location of the definition file
    source: PathBuf,

    /// Destination for TypeScript types
    #[clap(long, default_value = "elm.ts")]
    typescript_dest: PathBuf,

    /// Destination for Elm types
    #[clap(long, default_value = "src/")]
    elm_dest: PathBuf,
}

impl Cli {
    fn run(&self) -> Result<()> {
        let schema = schema::Schema::from_fs(&self.source).wrap_err("could not read schema")?;

        // TODO: better error message in all of this

        std::fs::write(&self.typescript_dest, schema.to_ts()?)?;
        println!("wrote {}", self.typescript_dest.display());

        for (name, contents) in schema.to_elm()? {
            let dest = self.elm_dest.join(name);
            if let Some(parent) = dest.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?
                };
            }

            std::fs::write(&dest, contents)?;
            println!("wrote {}", dest.display());
        }

        Ok(())
    }
}

fn main() {
    color_eyre::install().unwrap();

    let cli = Cli::parse();

    if let Err(problem) = cli.run() {
        println!("{problem:?}");
        std::process::exit(1);
    }
}
