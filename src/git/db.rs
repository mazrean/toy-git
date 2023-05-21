use super::object::Object;
use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use std::{fs::File, path::PathBuf};

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
}

impl Database {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
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
