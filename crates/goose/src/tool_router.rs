use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCapability {
    FileRead,
    FileWrite,
    Network,
    Shell,
    Browser,
    Mcp,
    Export,
    Delete,
}

#[derive(Debug, Clone)]
pub struct ToolRequest {
    pub tool_name: String,
    pub capability: ToolCapability,
    pub workspace_root: PathBuf,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolDecision {
    Allow,
    Ask { reason: String },
    Deny { reason: String },
}

#[derive(Debug, Default)]
pub struct ToolRouter;

impl ToolRouter {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(&self, request: &ToolRequest) -> ToolDecision {
        match request.capability {
            ToolCapability::FileRead => {
                if path_is_within_workspace(&request.workspace_root, request.path.as_deref()) {
                    ToolDecision::Allow
                } else {
                    ToolDecision::Deny {
                        reason: "File access is outside the task workspace".to_string(),
                    }
                }
            }
            ToolCapability::FileWrite | ToolCapability::Export | ToolCapability::Delete => {
                if path_is_within_workspace(&request.workspace_root, request.path.as_deref()) {
                    ToolDecision::Ask {
                        reason: format!("{} requires user confirmation", request.tool_name),
                    }
                } else {
                    ToolDecision::Deny {
                        reason: "File access is outside the task workspace".to_string(),
                    }
                }
            }
            ToolCapability::Network
            | ToolCapability::Shell
            | ToolCapability::Browser
            | ToolCapability::Mcp => ToolDecision::Deny {
                reason: "This capability is disabled by the default task policy".to_string(),
            },
        }
    }
}

fn path_is_within_workspace(workspace_root: &Path, path: Option<&Path>) -> bool {
    let Some(path) = path else {
        return false;
    };
    let Ok(workspace_root) = std::fs::canonicalize(workspace_root) else {
        return false;
    };
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    };

    let resolved = if path.exists() {
        std::fs::canonicalize(path).ok()
    } else {
        path.parent()
            .filter(|parent| parent.exists())
            .and_then(|parent| {
                let file_name = path.file_name()?;
                Some(std::fs::canonicalize(parent).ok()?.join(file_name))
            })
    };

    resolved.is_some_and(|resolved| resolved.starts_with(workspace_root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn request(root: &Path, capability: ToolCapability, path: Option<PathBuf>) -> ToolRequest {
        ToolRequest {
            tool_name: "test-tool".to_string(),
            capability,
            workspace_root: root.to_path_buf(),
            path,
        }
    }

    #[test]
    fn reads_existing_file_inside_workspace() {
        let root = tempdir().unwrap();
        fs::write(root.path().join("input.txt"), "data").unwrap();
        let router = ToolRouter::new();

        assert_eq!(
            router.evaluate(&request(
                root.path(),
                ToolCapability::FileRead,
                Some(PathBuf::from("input.txt")),
            )),
            ToolDecision::Allow
        );
    }

    #[test]
    fn denies_file_outside_workspace() {
        let root = tempdir().unwrap();
        let outside = tempdir().unwrap();
        fs::write(outside.path().join("secret.txt"), "secret").unwrap();
        let router = ToolRouter::new();

        assert!(matches!(
            router.evaluate(&request(
                root.path(),
                ToolCapability::FileRead,
                Some(outside.path().join("secret.txt")),
            )),
            ToolDecision::Deny { .. }
        ));
    }

    #[test]
    fn new_file_inside_workspace_requires_confirmation() {
        let root = tempdir().unwrap();
        let router = ToolRouter::new();

        assert!(matches!(
            router.evaluate(&request(
                root.path(),
                ToolCapability::FileWrite,
                Some(PathBuf::from("output.txt")),
            )),
            ToolDecision::Ask { .. }
        ));
    }

    #[test]
    fn shell_and_network_are_denied_by_default() {
        let root = tempdir().unwrap();
        let router = ToolRouter::new();

        for capability in [ToolCapability::Shell, ToolCapability::Network] {
            assert!(matches!(
                router.evaluate(&request(root.path(), capability, None)),
                ToolDecision::Deny { .. }
            ));
        }
    }
}
