use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
pub struct Command {
    #[clap(subcommand)]
    subcommands: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    #[clap(name = "add")]
    Add(Add),
}

impl Command {
    pub fn run() -> Result<()> {
        let command = Self::parse();
        match command.subcommands {
            Subcommands::Add(add) => add
                .execute()
                .with_context(|| format!("Failed to execute add command")),
        }
    }
}

#[derive(Args)]
struct Add {
    #[clap(name = "NAME")]
    name: String,
}

impl Add {
    fn execute(&self) -> Result<()> {
        println!("Add: {}", self.name);
        Ok(())
    }
}
