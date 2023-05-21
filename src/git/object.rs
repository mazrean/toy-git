use std::io::Read;

use super::utils::read_null_terminated_string;
use anyhow::{Context, Result};

pub struct Object {
    pub hash: String,
    pub object_size: usize,
    pub object_type: ObjectType,
}

impl Object {
    pub fn from_reader(hash: String, mut reader: impl std::io::Read) -> Result<Self> {
        let header = read_null_terminated_string(&mut reader)
            .with_context(|| "Failed to parse object header")?;

        let header_parts = header.split(' ').collect::<Vec<&str>>();
        if header_parts.len() != 2 {
            anyhow::bail!("Invalid object header");
        }

        let object_type = ObjectType::new(header_parts[0], &mut reader)
            .with_context(|| "Failed to parse object type")?;
        let object_size = header_parts[1]
            .parse::<usize>()
            .with_context(|| "Failed to parse object size")?;

        Ok(Self {
            hash,
            object_type,
            object_size,
        })
    }
}

pub enum ObjectType {
    Undefined,
    Blob(Blob),
    Tree,
    Commit,
    Tag,
}

impl ObjectType {
    fn new(object_type: &str, content_reader: &mut impl Read) -> Result<Self> {
        let object_type = match object_type {
            "blob" => Self::Blob(
                Blob::from_reader(content_reader).with_context(|| "Failed to read blob")?,
            ),
            "tree" => Self::Tree,
            "commit" => Self::Commit,
            "tag" => Self::Tag,
            _ => Self::Undefined,
        };
        Ok(object_type)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Undefined => "undefined",
            Self::Blob(_) => "blob",
            Self::Tree => "tree",
            Self::Commit => "commit",
            Self::Tag => "tag",
        }
    }
}

pub struct Blob {
    pub content: Vec<u8>,
}

impl Blob {
    pub fn from_reader(mut reader: impl Read) -> Result<Self> {
        let mut content = Vec::new();
        reader
            .read_to_end(&mut content)
            .with_context(|| "Failed to read blob content")?;
        Ok(Self { content })
    }
}
