use anyhow::{Context, Result};
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {
    #[clap(name = "NAME")]
    name: String,
}

impl Command {
    pub fn execute(&self) -> Result<()> {
        println!("Add: {}", self.name);
        Err(anyhow::anyhow!("Failed to execute command: {:?}", self))
    }
}
