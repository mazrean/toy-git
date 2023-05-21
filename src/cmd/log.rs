use crate::git::db::Database;
use crate::git::object::ObjectType::Commit;
use anyhow::{Context, Result};
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {
    #[clap(name = "REVISION RANGE")]
    revision_range: String,
}

impl Command {
    pub fn execute(&self) -> Result<()> {
        Database::new(std::path::Path::new(".git").to_path_buf())
            .walk_log(&self.revision_range, |object| {
                if let Commit(ref commit) = object.object_type {
                    println!("commit {}", object.hash);
                    println!("Author: {}", commit.author);
                    println!("");
                    println!("    {}", commit.message);
                }
                Ok(())
            })
            .with_context(|| format!("Failed to read object {}", self.revision_range))?;

        Ok(())
    }
}
