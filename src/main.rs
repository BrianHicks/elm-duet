use clap::Parser;
use eyre::Result;

#[derive(Debug, Parser)]
struct Cli {}

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
