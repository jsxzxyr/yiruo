use crate::policy_engine::DataClassification;
use crate::security::dlp::{DlpFinding, DlpScanner};
use crate::tool_router::{ToolCapability, ToolDecision, ToolRequest, ToolRouter};
use anyhow::{bail, Context, Result};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Text,
    Markdown,
    Html,
    Image,
    Unknown,
}

impl DocumentFormat {
    fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "pdf" => Self::Pdf,
            "docx" => Self::Docx,
            "xlsx" | "xls" => Self::Xlsx,
            "pptx" | "ppt" => Self::Pptx,
            "txt" => Self::Text,
            "md" | "markdown" => Self::Markdown,
            "html" | "htm" => Self::Html,
            "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tiff" => Self::Image,
            _ => Self::Unknown,
        }
    }

    pub fn is_textual(self) -> bool {
        matches!(self, Self::Text | Self::Markdown | Self::Html)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentMetadata {
    pub path: PathBuf,
    pub format: DocumentFormat,
    pub byte_len: u64,
    pub content_hash: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DocumentChunk {
    pub ordinal: usize,
    pub char_start: usize,
    pub char_end: usize,
    pub text: String,
    pub content_hash: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct PreparedDocument {
    pub metadata: DocumentMetadata,
    pub classification: DataClassification,
    pub findings: Vec<DlpFinding>,
    pub chunks: Vec<DocumentChunk>,
}

impl fmt::Debug for DocumentChunk {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DocumentChunk")
            .field("ordinal", &self.ordinal)
            .field("char_start", &self.char_start)
            .field("char_end", &self.char_end)
            .field("content_hash", &self.content_hash)
            .field("text_len", &self.text.len())
            .finish()
    }
}

impl fmt::Debug for PreparedDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedDocument")
            .field("metadata", &self.metadata)
            .field("classification", &self.classification)
            .field("finding_count", &self.findings.len())
            .field("chunks", &self.chunks)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct DocumentAgent {
    max_document_bytes: u64,
    chunk_chars: usize,
    overlap_chars: usize,
    tool_router: ToolRouter,
    dlp: DlpScanner,
}

impl Default for DocumentAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentAgent {
    pub fn new() -> Self {
        Self {
            max_document_bytes: 10 * 1024 * 1024,
            chunk_chars: 8 * 1024,
            overlap_chars: 512,
            tool_router: ToolRouter::new(),
            dlp: DlpScanner::new(),
        }
    }

    pub fn with_limits(mut self, max_document_bytes: u64, chunk_chars: usize) -> Result<Self> {
        if max_document_bytes == 0 || chunk_chars == 0 {
            bail!("document limits must be positive")
        }
        self.max_document_bytes = max_document_bytes;
        self.chunk_chars = chunk_chars;
        self.overlap_chars = self.overlap_chars.min(chunk_chars.saturating_sub(1));
        Ok(self)
    }

    pub fn inspect_path(&self, workspace_root: &Path, path: &Path) -> Result<DocumentMetadata> {
        let resolved_path = resolve_workspace_path(workspace_root, path);
        self.authorize_read(workspace_root, &resolved_path)?;
        let metadata = std::fs::metadata(&resolved_path)
            .with_context(|| format!("unable to inspect document metadata"))?;
        if !metadata.is_file() {
            bail!("document path is not a file")
        }
        if metadata.len() > self.max_document_bytes {
            bail!("document exceeds the configured size limit")
        }

        let bytes = std::fs::read(&resolved_path).context("unable to hash document")?;
        Ok(DocumentMetadata {
            path: resolved_path,
            format: DocumentFormat::from_path(path),
            byte_len: metadata.len(),
            content_hash: blake3::hash(&bytes).to_hex().to_string(),
        })
    }

    pub fn prepare_text(
        &self,
        name: impl AsRef<Path>,
        text: &str,
        minimum: DataClassification,
    ) -> PreparedDocument {
        let report = self.dlp.scan_with_minimum(text, minimum);
        let metadata = DocumentMetadata {
            path: name.as_ref().to_path_buf(),
            format: DocumentFormat::from_path(name.as_ref()),
            byte_len: text.len() as u64,
            content_hash: report.content_hash.clone(),
        };
        let chunks = chunk_text(text, self.chunk_chars, self.overlap_chars);

        PreparedDocument {
            metadata,
            classification: report.classification,
            findings: report.findings,
            chunks,
        }
    }

    pub fn prepare_file(
        &self,
        workspace_root: &Path,
        path: &Path,
        minimum: DataClassification,
    ) -> Result<PreparedDocument> {
        let metadata = self.inspect_path(workspace_root, path)?;
        if !metadata.format.is_textual() {
            bail!(
                "format {:?} requires a trusted parser adapter before text preparation",
                metadata.format
            )
        }
        let text =
            std::fs::read_to_string(&metadata.path).context("document is not valid UTF-8 text")?;
        Ok(self.prepare_text(&metadata.path, &text, minimum))
    }

    fn authorize_read(&self, workspace_root: &Path, path: &Path) -> Result<()> {
        let request = ToolRequest {
            tool_name: "document.read".to_string(),
            capability: ToolCapability::FileRead,
            workspace_root: workspace_root.to_path_buf(),
            path: Some(path.to_path_buf()),
        };
        match self.tool_router.evaluate(&request) {
            ToolDecision::Allow => Ok(()),
            ToolDecision::Ask { reason } | ToolDecision::Deny { reason } => {
                bail!("document read is not allowed: {reason}")
            }
        }
    }
}

fn chunk_text(text: &str, chunk_chars: usize, overlap_chars: usize) -> Vec<DocumentChunk> {
    if text.is_empty() {
        return Vec::new();
    }

    let chars: Vec<char> = text.chars().collect();
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let end = (start + chunk_chars).min(chars.len());
        let chunk_text: String = chars[start..end].iter().collect();
        chunks.push(DocumentChunk {
            ordinal: chunks.len(),
            char_start: start,
            char_end: end,
            content_hash: blake3::hash(chunk_text.as_bytes()).to_hex().to_string(),
            text: chunk_text,
        });
        if end == chars.len() {
            break;
        }
        start = end.saturating_sub(overlap_chars);
    }
    chunks
}

fn resolve_workspace_path(workspace_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn chunks_are_bounded_and_overlap_without_splitting_utf8() {
        let agent = DocumentAgent::new().with_limits(1024, 4).unwrap();
        let prepared = agent.prepare_text(
            "材料.txt",
            "甲乙丙丁戊己庚辛",
            DataClassification::L1Internal,
        );

        assert_eq!(prepared.chunks.len(), 3);
        assert_eq!(prepared.chunks[0].text, "甲乙丙丁");
        assert_eq!(prepared.chunks[1].text, "丁戊己庚");
        assert_eq!(prepared.chunks[2].text, "庚辛");
        assert_eq!(
            prepared.chunks[0].char_end,
            prepared.chunks[1].char_start + 1
        );
    }

    #[test]
    fn path_access_is_workspace_scoped_and_size_bounded() {
        let workspace = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let inside_path = workspace.path().join("input.txt");
        let outside_path = outside.path().join("secret.txt");
        fs::write(&inside_path, "ordinary material").unwrap();
        fs::write(&outside_path, "secret material").unwrap();
        let agent = DocumentAgent::new().with_limits(1024, 128).unwrap();

        assert!(agent
            .inspect_path(workspace.path(), Path::new("input.txt"))
            .is_ok());
        assert!(agent
            .prepare_file(
                workspace.path(),
                Path::new("input.txt"),
                DataClassification::L1Internal,
            )
            .is_ok());
        assert!(agent.inspect_path(workspace.path(), &outside_path).is_err());
    }

    #[test]
    fn document_classification_is_preserved_before_model_routing() {
        let prepared = DocumentAgent::new().prepare_text(
            "input.md",
            "api_key=secret-value-123",
            DataClassification::L1Internal,
        );

        assert_eq!(prepared.classification, DataClassification::L4Restricted);
        assert_eq!(prepared.findings.len(), 1);
        assert!(!format!("{prepared:?}").contains("secret-value-123"));
    }

    #[test]
    fn binary_formats_require_explicit_parser_adapter() {
        let workspace = tempdir().unwrap();
        let path = workspace.path().join("input.pdf");
        fs::write(&path, b"%PDF-1.7").unwrap();

        assert!(DocumentAgent::new()
            .prepare_file(workspace.path(), &path, DataClassification::L1Internal)
            .is_err());
    }
}
