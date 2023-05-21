use super::object::{Object, ObjectType};
use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    path::PathBuf,
};

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
}

impl Database {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn walk_log(
        &self,
        commit_hash: &str,
        mut walk_func: impl FnMut(&Object) -> Result<()>,
    ) -> Result<()> {
        let mut map = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back(commit_hash.to_string());

        while let Some(hash) = queue.pop_back() {
            if map.contains_key(&hash) {
                continue;
            }

            let object = self
                .read_object(hash.as_str())
                .with_context(|| format!("Failed to read object {}", hash))?;
            if let ObjectType::Commit(ref commit_object) = object.object_type {
                commit_object.parents.iter().for_each(|parent_hash| {
                    queue.push_back(parent_hash.to_string());
                });

                walk_func(&object).with_context(|| format!("Failed to walk object {}", hash))?;
            } else {
                anyhow::bail!("Object {} is not a commit", hash);
            }

            map.insert(hash, ());
        }
        Ok(())
    }

    pub fn read_object(&self, object_hash: &str) -> Result<Object> {
        let obj_file_path = self
            .path
            .join("objects")
            .join(&object_hash[..2])
            .join(&object_hash[2..]);
        let mut obj_file = File::open(obj_file_path)
            .with_context(|| format!("Failed to open object file {}", object_hash))?;

        let mut decoder = ZlibDecoder::new(&mut obj_file);
        let object = Object::from_reader(object_hash.to_string(), &mut decoder)
            .with_context(|| format!("Failed to read object {}", object_hash))?;

        Ok(object)
    }
}
