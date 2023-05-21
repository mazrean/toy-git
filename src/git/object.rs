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
    Tree(Tree),
    Commit,
    Tag,
}

impl ObjectType {
    fn new(object_type: &str, content_reader: &mut impl Read) -> Result<Self> {
        let object_type = match object_type {
            "blob" => Self::Blob(
                Blob::from_reader(content_reader).with_context(|| "Failed to read blob")?,
            ),
            "tree" => Self::Tree(
                Tree::from_reader(content_reader).with_context(|| "Failed to read tree")?,
            ),
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
            Self::Tree(_) => "tree",
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

pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

pub struct TreeEntry {
    pub mode: u32,
    pub name: String,
    pub hash: String,
}

impl Tree {
    pub fn from_reader(mut reader: impl Read) -> Result<Self> {
        let mut entries: Vec<TreeEntry> = Vec::new();
        while let Some(entry) =
            TreeEntry::from_reader(&mut reader).with_context(|| "Failed to read tree entry")?
        {
            entries.push(entry);
        }

        Ok(Self { entries })
    }
}

impl TreeEntry {
    pub fn from_reader(mut reader: impl Read) -> Result<Option<Self>> {
        let header = read_null_terminated_string(&mut reader)
            .with_context(|| "Failed to read tree entry mode")?;
        if header.is_empty() {
            return Ok(None);
        }

        let header_parts = header.split(' ').collect::<Vec<&str>>();
        if header_parts.len() != 2 {
            anyhow::bail!("Invalid tree entry header");
        }

        let mode = header_parts[0]
            .parse::<u32>()
            .with_context(|| "Failed to parse tree entry mode")?;
        let name = header_parts[1].to_string();

        let hash = reader
            .bytes()
            .take_while(|byte| byte.as_ref().map(|b| *b != 3).unwrap_or(false))
            .map(|n| format!("{:02x}", n.unwrap()))
            .collect::<String>();
        if hash.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self { mode, name, hash }))
    }
}
