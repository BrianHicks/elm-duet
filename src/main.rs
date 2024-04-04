mod schema;

use clap::Parser;
use color_eyre::Result;
use eyre::WrapErr;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Cli {
    /// Location of the definition file
    source: PathBuf,
}

impl Cli {
    fn run(&self) -> Result<()> {
        println!("{self:#?}");

        let schema = schema::Schema::from_fs(&self.source).wrap_err("could not read schema")?;
        println!("{schema:#?}");

        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();

    if let Err(problem) = cli.run() {
        println!("{problem:?}");
        std::process::exit(1);
    }
}
