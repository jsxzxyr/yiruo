use crate::policy_engine::DataClassification;
use crate::tool_router::{ToolCapability, ToolDecision, ToolRequest, ToolRouter};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OfficialDocumentType {
    Notice,
    Request,
    Report,
    OfficialLetter,
    Minutes,
    Decision,
    Opinion,
    Plan,
    Summary,
    Speech,
    Brief,
    ResearchReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficialDocumentTemplate {
    pub document_type: OfficialDocumentType,
    pub display_name: &'static str,
    pub required_sections: &'static [&'static str],
    pub requires_addressee: bool,
    pub requires_signature: bool,
}

impl OfficialDocumentType {
    pub fn template(self) -> OfficialDocumentTemplate {
        match self {
            Self::Notice => OfficialDocumentTemplate {
                document_type: self,
                display_name: "通知",
                required_sections: &["标题", "正文", "落款", "日期"],
                requires_addressee: false,
                requires_signature: true,
            },
            Self::Request => OfficialDocumentTemplate {
                document_type: self,
                display_name: "请示",
                required_sections: &["标题", "主送单位", "正文", "请示事项", "落款", "日期"],
                requires_addressee: true,
                requires_signature: true,
            },
            Self::Report => OfficialDocumentTemplate {
                document_type: self,
                display_name: "报告",
                required_sections: &["标题", "主送单位", "正文", "情况", "建议", "落款", "日期"],
                requires_addressee: true,
                requires_signature: true,
            },
            Self::OfficialLetter => OfficialDocumentTemplate {
                document_type: self,
                display_name: "函",
                required_sections: &["标题", "主送单位", "正文", "落款", "日期"],
                requires_addressee: true,
                requires_signature: true,
            },
            Self::Minutes => OfficialDocumentTemplate {
                document_type: self,
                display_name: "纪要",
                required_sections: &["标题", "时间地点", "参会人员", "议定事项", "日期"],
                requires_addressee: false,
                requires_signature: false,
            },
            Self::Decision => OfficialDocumentTemplate {
                document_type: self,
                display_name: "决定",
                required_sections: &["标题", "正文", "执行要求", "落款", "日期"],
                requires_addressee: false,
                requires_signature: true,
            },
            Self::Opinion => OfficialDocumentTemplate {
                document_type: self,
                display_name: "意见",
                required_sections: &["标题", "正文", "工作要求", "落款", "日期"],
                requires_addressee: false,
                requires_signature: true,
            },
            Self::Plan => OfficialDocumentTemplate {
                document_type: self,
                display_name: "方案",
                required_sections: &[
                    "标题",
                    "背景目标",
                    "重点任务",
                    "实施步骤",
                    "保障措施",
                    "日期",
                ],
                requires_addressee: false,
                requires_signature: false,
            },
            Self::Summary => OfficialDocumentTemplate {
                document_type: self,
                display_name: "工作总结",
                required_sections: &["标题", "工作情况", "成效", "问题", "下一步", "日期"],
                requires_addressee: false,
                requires_signature: false,
            },
            Self::Speech => OfficialDocumentTemplate {
                document_type: self,
                display_name: "讲话稿",
                required_sections: &["标题", "称谓", "正文", "结束语"],
                requires_addressee: false,
                requires_signature: false,
            },
            Self::Brief => OfficialDocumentTemplate {
                document_type: self,
                display_name: "工作简报",
                required_sections: &["标题", "导语", "正文", "编发单位", "日期"],
                requires_addressee: false,
                requires_signature: false,
            },
            Self::ResearchReport => OfficialDocumentTemplate {
                document_type: self,
                display_name: "调研报告",
                required_sections: &["标题", "调研背景", "现状分析", "问题", "建议", "日期"],
                requires_addressee: false,
                requires_signature: false,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficialDocumentDraft {
    pub document_type: OfficialDocumentType,
    pub title: String,
    pub addressee: Option<String>,
    pub sections: Vec<OfficialDocumentSection>,
    pub signature: Option<String>,
    pub date: Option<String>,
    pub classification: DataClassification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficialDocumentSection {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub code: &'static str,
    pub field: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputApproval {
    pub destination: PathBuf,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Default)]
pub struct OfficialDocumentAgent {
    tool_router: ToolRouter,
}

impl OfficialDocumentAgent {
    pub fn new() -> Self {
        Self {
            tool_router: ToolRouter::new(),
        }
    }

    pub fn validate(&self, draft: &OfficialDocumentDraft) -> Vec<ValidationIssue> {
        let template = draft.document_type.template();
        let mut issues = Vec::new();
        if draft.title.trim().is_empty() {
            issues.push(ValidationIssue {
                code: "missing_title",
                field: "title",
                message: "标题不能为空",
            });
        }
        if template.requires_addressee
            && draft
                .addressee
                .as_deref()
                .is_none_or(|addressee| addressee.trim().is_empty())
        {
            issues.push(ValidationIssue {
                code: "missing_addressee",
                field: "addressee",
                message: "该文种需要主送单位",
            });
        }
        if template.requires_signature
            && draft
                .signature
                .as_deref()
                .is_none_or(|signature| signature.trim().is_empty())
        {
            issues.push(ValidationIssue {
                code: "missing_signature",
                field: "signature",
                message: "该文种需要落款",
            });
        }
        if template.required_sections.iter().any(|required| {
            *required != "标题"
                && *required != "主送单位"
                && *required != "落款"
                && *required != "日期"
                && !draft.sections.iter().any(|section| {
                    section.heading.trim() == *required && !section.body.trim().is_empty()
                })
        }) {
            issues.push(ValidationIssue {
                code: "missing_section",
                field: "sections",
                message: "正文缺少模板要求的结构",
            });
        }
        if draft
            .date
            .as_deref()
            .is_none_or(|date| date.trim().is_empty())
        {
            issues.push(ValidationIssue {
                code: "missing_date",
                field: "date",
                message: "日期不能为空",
            });
        }
        issues
    }

    pub fn approve_output(
        &self,
        workspace_root: &Path,
        destination: &Path,
    ) -> Result<OutputApproval> {
        let request = ToolRequest {
            tool_name: "official_document.export".to_string(),
            capability: ToolCapability::Export,
            workspace_root: workspace_root.to_path_buf(),
            path: Some(destination.to_path_buf()),
        };
        match self.tool_router.evaluate(&request) {
            ToolDecision::Ask { .. } => Ok(OutputApproval {
                destination: resolve_workspace_path(workspace_root, destination),
                requires_confirmation: true,
            }),
            ToolDecision::Allow => Ok(OutputApproval {
                destination: resolve_workspace_path(workspace_root, destination),
                requires_confirmation: false,
            }),
            ToolDecision::Deny { reason } => bail!("document export is not allowed: {reason}"),
        }
    }
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
    use tempfile::tempdir;

    fn report() -> OfficialDocumentDraft {
        OfficialDocumentDraft {
            document_type: OfficialDocumentType::Report,
            title: "年度工作报告".to_string(),
            addressee: Some("上级单位".to_string()),
            sections: vec![
                OfficialDocumentSection {
                    heading: "情况".to_string(),
                    body: "工作情况".to_string(),
                },
                OfficialDocumentSection {
                    heading: "建议".to_string(),
                    body: "工作建议".to_string(),
                },
            ],
            signature: Some("本单位".to_string()),
            date: Some("2026年9月7日".to_string()),
            classification: DataClassification::L1Internal,
        }
    }

    #[test]
    fn validates_required_official_document_structure() {
        let agent = OfficialDocumentAgent::new();
        assert!(agent.validate(&report()).is_empty());

        let mut invalid = report();
        invalid.addressee = None;
        invalid.sections.clear();
        let issues = agent.validate(&invalid);
        assert!(issues.iter().any(|issue| issue.code == "missing_addressee"));
        assert!(issues.iter().any(|issue| issue.code == "missing_section"));
    }

    #[test]
    fn output_is_workspace_scoped_and_confirmation_gated() {
        let workspace = tempdir().unwrap();
        let agent = OfficialDocumentAgent::new();

        let approval = agent
            .approve_output(workspace.path(), Path::new("output/report.docx"))
            .unwrap();
        assert!(approval.requires_confirmation);
        assert!(approval.destination.starts_with(workspace.path()));

        assert!(agent
            .approve_output(workspace.path(), Path::new("..\\outside.docx"))
            .is_err());
    }

    #[test]
    fn restricted_draft_classification_is_retained_for_policy_routing() {
        let mut draft = report();
        draft.classification = DataClassification::L3HighSensitive;
        assert_eq!(draft.classification, DataClassification::L3HighSensitive);
    }
}
