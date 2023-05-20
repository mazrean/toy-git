use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod cat_file;

#[derive(Parser)]
pub struct Command {
    #[clap(subcommand)]
    subcommands: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    #[clap(name = "cat-file")]
    CatFile(cat_file::Command),
}

impl Command {
    pub fn run() -> Result<()> {
        let command = Self::parse();
        match command.subcommands {
            Subcommands::CatFile(cmd) => cmd
                .execute()
                .with_context(|| format!("Failed to execute cat-file command")),
        }
    }
}
