mod schema;
mod typescript;

use clap::Parser;
use color_eyre::Result;
use eyre::WrapErr;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(version)]
struct Cli {
    /// Location of the definition file
    source: PathBuf,
}

impl Cli {
    fn run(&self) -> Result<()> {
        let schema = schema::Schema::from_fs(&self.source).wrap_err("could not read schema")?;

        println!("{}", schema.flags_to_ts().unwrap().unwrap());

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
