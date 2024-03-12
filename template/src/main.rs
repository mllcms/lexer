mod generate;
mod replace;

use clap::Parser;

fn main() {
    let cli = Cli::parse();
    if let Err(err) = cli.run() {
        eprintln!("Error: {err}")
    }
}

#[derive(Parser)]
pub enum Cli {
    Gen(generate::Generate),
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Cli::Gen(gen) => gen.run(),
        }
    }
}
