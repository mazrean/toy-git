use crate::git::db::Database;
use crate::git::object::ObjectType::{Blob, Commit, Tree};
use anyhow::{Context, Result};
use clap::Args;

#[derive(Args, Debug)]
pub struct Command {
    #[clap(name = "OBJECT")]
    object: String,
    #[clap(name = "type", short = 't')]
    object_type: bool,
    #[clap(name = "size", short = 's')]
    object_size: bool,
    #[clap(name = "pretty-print", short = 'p')]
    pretty_print: bool,
}

impl Command {
    pub fn execute(&self) -> Result<()> {
        let object = Database::new(std::path::Path::new(".git").to_path_buf())
            .read_object(&self.object)
            .with_context(|| format!("Failed to read object {}", self.object))?;

        if self.object_type {
            println!("{}", object.object_type.as_str());
            return Ok(());
        }

        if self.object_size {
            println!("{}", object.object_size);
            return Ok(());
        }

        if self.pretty_print {
            match object.object_type {
                Blob(blob) => {
                    println!("{}", std::str::from_utf8(&blob.content)?);
                }
                Tree(tree) => {
                    for entry in tree.entries {
                        println!("{} {} {}", entry.mode, entry.hash, entry.name);
                    }
                }
                Commit(commit) => {
                    println!("tree {}", commit.tree);
                    for parent_hash in commit.parents {
                        println!("parent {}", parent_hash);
                    }
                    println!("author {}", commit.author);
                    println!("committer {}", commit.committer);
                    println!("");
                    println!("{}", commit.message);
                }
                _ => {
                    anyhow::bail!("Pretty print is not supported for this object type");
                }
            }
            return Ok(());
        }

        Ok(())
    }
}
