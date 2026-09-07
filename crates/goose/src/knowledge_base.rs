use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeBase {
    pub id: String,
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeDocument {
    pub knowledge_base_id: String,
    pub path: PathBuf,
    pub byte_len: u64,
    pub content_hash: String,
}

#[derive(Debug, Default, Clone)]
pub struct KnowledgeBaseRegistry {
    bases: BTreeMap<String, KnowledgeBase>,
}

impl KnowledgeBaseRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, id: impl Into<String>, root: &Path) -> Result<KnowledgeBase> {
        let id = id.into().trim().to_string();
        if id.is_empty() {
            bail!("knowledge base id must not be empty")
        }
        if self.bases.contains_key(&id) {
            bail!("knowledge base id is already registered")
        }
        let root = std::fs::canonicalize(root).context("knowledge base root is not accessible")?;
        if !root.is_dir() {
            bail!("knowledge base root is not a directory")
        }
        if self
            .bases
            .values()
            .any(|base| root.starts_with(&base.root) || base.root.starts_with(&root))
        {
            bail!("knowledge base roots must not overlap")
        }

        let base = KnowledgeBase {
            id: id.clone(),
            root,
        };
        self.bases.insert(id, base.clone());
        Ok(base)
    }

    pub fn get(&self, id: &str) -> Option<&KnowledgeBase> {
        self.bases.get(id)
    }

    pub fn authorize_read(&self, id: &str, path: &Path) -> Result<PathBuf> {
        let base = self
            .bases
            .get(id)
            .context("knowledge base is not registered")?;
        let resolved = resolve_path(&base.root, path);
        let canonical = canonicalize_existing_or_parent(&resolved)?;
        if !canonical.starts_with(&base.root) {
            bail!("path is outside the selected knowledge base")
        }
        if self
            .bases
            .values()
            .any(|other| other.id != base.id && canonical.starts_with(&other.root))
        {
            bail!("path belongs to another knowledge base")
        }
        if !canonical.is_file() {
            bail!("knowledge base path is not a file")
        }
        Ok(canonical)
    }

    pub fn index_file(&self, id: &str, path: &Path) -> Result<KnowledgeDocument> {
        let path = self.authorize_read(id, path)?;
        let bytes = std::fs::read(&path).context("unable to read knowledge base file")?;
        Ok(KnowledgeDocument {
            knowledge_base_id: id.to_string(),
            path,
            byte_len: bytes.len() as u64,
            content_hash: blake3::hash(&bytes).to_hex().to_string(),
        })
    }
}

fn resolve_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn canonicalize_existing_or_parent(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return std::fs::canonicalize(path).context("unable to resolve knowledge base path");
    }
    let parent = path.parent().context("knowledge base path has no parent")?;
    let file_name = path
        .file_name()
        .context("knowledge base path has no filename")?;
    Ok(std::fs::canonicalize(parent)
        .context("unable to resolve knowledge base parent")?
        .join(file_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn knowledge_bases_cannot_overlap() {
        let root = tempdir().unwrap();
        let nested = root.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let mut registry = KnowledgeBaseRegistry::new();

        registry.register("a", root.path()).unwrap();
        assert!(registry.register("b", &nested).is_err());
    }

    #[test]
    fn each_base_can_only_read_its_own_files() {
        let parent = tempdir().unwrap();
        let first = parent.path().join("first");
        let second = parent.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        fs::write(first.join("a.txt"), "a").unwrap();
        fs::write(second.join("b.txt"), "b").unwrap();
        let mut registry = KnowledgeBaseRegistry::new();
        registry.register("first", &first).unwrap();
        registry.register("second", &second).unwrap();

        assert!(registry.authorize_read("first", Path::new("a.txt")).is_ok());
        assert!(registry
            .authorize_read("first", &second.join("b.txt"))
            .is_err());
        assert!(registry
            .authorize_read("first", Path::new("../second/b.txt"))
            .is_err());
    }

    #[test]
    fn indexed_metadata_contains_hash_but_not_content() {
        let base = tempdir().unwrap();
        fs::write(base.path().join("material.txt"), "private material").unwrap();
        let mut registry = KnowledgeBaseRegistry::new();
        registry.register("materials", base.path()).unwrap();

        let document = registry
            .index_file("materials", Path::new("material.txt"))
            .unwrap();
        assert_eq!(document.byte_len, 16);
        assert!(!format!("{document:?}").contains("private material"));
    }
}
