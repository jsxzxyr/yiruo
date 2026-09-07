use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataClassification {
    L0Public,
    L1Internal,
    L2Sensitive,
    L3HighSensitive,
    L4Restricted,
}

impl Default for DataClassification {
    fn default() -> Self {
        Self::L1Internal
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDisclosure {
    pub provider: String,
    pub model: String,
    pub classification: DataClassification,
    pub original_payload: bool,
    pub tool_calling: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow {
        rule: String,
    },
    Ask {
        reason: String,
        disclosure: PolicyDisclosure,
    },
    Deny {
        reason: String,
    },
}

impl PolicyDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }

    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Self::Ask { .. })
    }
}

#[derive(Debug, Clone)]
pub struct ModelPolicyRequest {
    pub provider: String,
    pub model: String,
    pub classification: DataClassification,
    pub original_payload: bool,
    pub tool_calling: bool,
    pub user_confirmed: bool,
}

#[derive(Debug, Clone)]
pub struct ProviderPolicy {
    allowed_levels: BTreeSet<DataClassification>,
    confirmation_levels: BTreeSet<DataClassification>,
    allow_original_payload: bool,
    allow_tool_calling: bool,
}

impl ProviderPolicy {
    pub fn local() -> Self {
        Self {
            allowed_levels: DataClassification::all().collect(),
            confirmation_levels: BTreeSet::new(),
            allow_original_payload: true,
            allow_tool_calling: true,
        }
    }

    pub fn external() -> Self {
        Self {
            allowed_levels: [DataClassification::L0Public, DataClassification::L1Internal]
                .into_iter()
                .collect(),
            confirmation_levels: [DataClassification::L1Internal].into_iter().collect(),
            allow_original_payload: false,
            allow_tool_calling: false,
        }
    }

    pub fn with_allowed_level(mut self, level: DataClassification) -> Self {
        self.allowed_levels.insert(level);
        self
    }

    pub fn with_confirmation_level(mut self, level: DataClassification) -> Self {
        self.confirmation_levels.insert(level);
        self
    }

    pub fn with_original_payload(mut self, allowed: bool) -> Self {
        self.allow_original_payload = allowed;
        self
    }

    pub fn with_tool_calling(mut self, allowed: bool) -> Self {
        self.allow_tool_calling = allowed;
        self
    }
}

impl DataClassification {
    fn all() -> impl Iterator<Item = Self> {
        [
            Self::L0Public,
            Self::L1Internal,
            Self::L2Sensitive,
            Self::L3HighSensitive,
            Self::L4Restricted,
        ]
        .into_iter()
    }
}

#[derive(Debug, Default)]
pub struct PolicyEngine {
    providers: BTreeMap<String, ProviderPolicy>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_provider(&mut self, provider: impl Into<String>, policy: ProviderPolicy) {
        self.providers.insert(provider.into(), policy);
    }

    pub fn evaluate_model(&self, request: &ModelPolicyRequest) -> PolicyDecision {
        let policy = self
            .providers
            .get(&request.provider)
            .cloned()
            .unwrap_or_else(ProviderPolicy::external);

        if request.original_payload && !policy.allow_original_payload {
            return PolicyDecision::Deny {
                reason: "Provider policy does not allow the original payload".to_string(),
            };
        }

        if request.tool_calling && !policy.allow_tool_calling {
            return PolicyDecision::Deny {
                reason: "Provider policy does not allow tool calling".to_string(),
            };
        }

        if !policy.allowed_levels.contains(&request.classification) {
            return PolicyDecision::Deny {
                reason: format!(
                    "Provider policy does not allow {:?} data",
                    request.classification
                ),
            };
        }

        if policy.confirmation_levels.contains(&request.classification) && !request.user_confirmed {
            return PolicyDecision::Ask {
                reason: "External model use requires user confirmation".to_string(),
                disclosure: PolicyDisclosure {
                    provider: request.provider.clone(),
                    model: request.model.clone(),
                    classification: request.classification,
                    original_payload: request.original_payload,
                    tool_calling: request.tool_calling,
                },
            };
        }

        PolicyDecision::Allow {
            rule: format!("Provider policy allows {:?} data", request.classification),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(provider: &str, classification: DataClassification) -> ModelPolicyRequest {
        ModelPolicyRequest {
            provider: provider.to_string(),
            model: "test-model".to_string(),
            classification,
            original_payload: false,
            tool_calling: false,
            user_confirmed: false,
        }
    }

    #[test]
    fn unknown_provider_defaults_to_external_policy() {
        let engine = PolicyEngine::new();
        assert!(!engine
            .evaluate_model(&request("unknown", DataClassification::L2Sensitive))
            .is_allowed());
    }

    #[test]
    fn external_internal_data_requires_confirmation() {
        let engine = PolicyEngine::new();
        let decision = engine.evaluate_model(&request("external", DataClassification::L1Internal));

        assert!(decision.requires_confirmation());
        assert!(matches!(decision, PolicyDecision::Ask { .. }));
    }

    #[test]
    fn confirmation_allows_configured_external_level() {
        let mut engine = PolicyEngine::new();
        engine.register_provider("external", ProviderPolicy::external());
        let mut request = request("external", DataClassification::L1Internal);
        request.user_confirmed = true;

        assert!(engine.evaluate_model(&request).is_allowed());
    }

    #[test]
    fn local_policy_allows_restricted_data_without_external_features() {
        let mut engine = PolicyEngine::new();
        engine.register_provider("local", ProviderPolicy::local());
        let request = request("local", DataClassification::L4Restricted);

        assert!(engine.evaluate_model(&request).is_allowed());
    }

    #[test]
    fn external_tool_calling_is_denied_even_when_data_is_public() {
        let engine = PolicyEngine::new();
        let mut request = request("external", DataClassification::L0Public);
        request.tool_calling = true;

        assert!(matches!(
            engine.evaluate_model(&request),
            PolicyDecision::Deny { .. }
        ));
    }
}
