use clap::Parser;
use eyre::Result;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Cli {
    /// Location of the definition file
    source: PathBuf,
}

impl Cli {
    fn run(&self) -> Result<()> {
        println!("{self:#?}");
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
