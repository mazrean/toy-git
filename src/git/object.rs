use std::io::{BufRead, BufReader, Read};

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
    Commit(Commit),
    Tag(Tag),
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
            "commit" => Self::Commit(
                Commit::from_reader(content_reader).with_context(|| "Failed to read commit")?,
            ),
            "tag" => {
                Self::Tag(Tag::from_reader(content_reader).with_context(|| "Failed to read tag")?)
            }
            _ => Self::Undefined,
        };
        Ok(object_type)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Undefined => "undefined",
            Self::Blob(_) => "blob",
            Self::Tree(_) => "tree",
            Self::Commit(_) => "commit",
            Self::Tag(_) => "tag",
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

pub struct Commit {
    pub tree: String,
    pub parents: Vec<String>,
    pub author: String,
    pub committer: String,
    pub message: String,
}

impl Commit {
    pub fn from_reader(reader: impl Read) -> Result<Self> {
        let mut tree: Option<String> = Option::None;
        let mut parents: Vec<String> = Vec::new();
        let mut author: Option<String> = Option::None;
        let mut committer: Option<String> = Option::None;
        let mut message = String::new();
        let mut end_of_header = false;
        for line in BufReader::new(reader).lines() {
            let line = line.with_context(|| "Failed to read commit object")?;

            if end_of_header {
                message.push_str(&line);
                message.push('\n');
                continue;
            }

            if line.is_empty() {
                end_of_header = true;
                continue;
            }
            if line.starts_with("tree ") {
                tree = Some(line.trim_start_matches("tree ").to_string());
            } else if line.starts_with("parent ") {
                parents.push(line.trim_start_matches("parent ").to_string());
            } else if line.starts_with("author ") {
                author = Some(line.trim_start_matches("author ").to_string());
            } else if line.starts_with("committer ") {
                committer = Some(line.trim_start_matches("committer ").to_string());
            }
        }

        if parents.is_empty() {
            anyhow::bail!("Missing parents");
        }

        Ok(Self {
            tree: tree.ok_or(anyhow::anyhow!("Missing tree"))?,
            parents,
            author: author.ok_or(anyhow::anyhow!("Missing author"))?,
            committer: committer.ok_or(anyhow::anyhow!("Missing committer"))?,
            message,
        })
    }
}

pub struct Tag {
    pub object: String,
    pub object_type: String,
    pub tag: String,
    pub tagger: String,
    pub message: String,
}

impl Tag {
    pub fn from_reader(reader: impl Read) -> Result<Self> {
        let mut object: Option<String> = Option::None;
        let mut object_type: Option<String> = Option::None;
        let mut tag: Option<String> = Option::None;
        let mut tagger: Option<String> = Option::None;
        let mut message = String::new();
        let mut end_of_header = false;
        for line in BufReader::new(reader).lines() {
            let line = line.with_context(|| "Failed to read tag object")?;

            if end_of_header {
                message.push_str(&line);
                message.push('\n');
                continue;
            }

            if line.is_empty() {
                end_of_header = true;
                continue;
            }
            if line.starts_with("object ") {
                object = Some(line.trim_start_matches("object ").to_string());
            } else if line.starts_with("type ") {
                object_type = Some(line.trim_start_matches("type ").to_string());
            } else if line.starts_with("tag ") {
                tag = Some(line.trim_start_matches("tag ").to_string());
            } else if line.starts_with("tagger ") {
                tagger = Some(line.trim_start_matches("tagger ").to_string());
            }
        }

        Ok(Self {
            object: object.ok_or(anyhow::anyhow!("Missing object"))?,
            object_type: object_type.ok_or(anyhow::anyhow!("Missing object type"))?,
            tag: tag.ok_or(anyhow::anyhow!("Missing tag"))?,
            tagger: tagger.ok_or(anyhow::anyhow!("Missing tagger"))?,
            message,
        })
    }
}
