//! The single model-call boundary used by the Yiruo runtime.
//!
//! Provider implementations still come from Goose, but callers only receive
//! this wrapper from the provider registry.  The wrapper deliberately keeps
//! request contents out of diagnostics and provides the seam where the policy
//! engine, DLP, and budget ledger are attached at this boundary.

use async_trait::async_trait;
use goose_providers::base::{MessageStream, ModelInfo, PermissionRouting, Provider};
use goose_providers::conversation::message::Message;
use goose_providers::conversation::token_usage::ProviderUsage;
use goose_providers::errors::ProviderError;
use goose_providers::goose_mode::GooseMode;
use goose_providers::model::ModelConfig;
use goose_providers::permission::PermissionConfirmation;
use goose_providers::retry::RetryConfig;
use goose_providers::thinking::ThinkingEffortSupport;
use rmcp::model::Tool;
use std::sync::Arc;
use tokio::sync::watch;

use crate::policy_engine::DataClassification;
use crate::security::dlp::DlpScanner;

/// Provider calls are intentionally represented by metadata only.  This is
/// the policy-engine input; raw prompts, tool schemas, and attachments must
/// never be placed in an audit event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayCallMetadata {
    pub provider: String,
    pub model: String,
    pub message_count: usize,
    pub tool_count: usize,
    pub data_classification: DataClassification,
    pub finding_count: usize,
}

impl GatewayCallMetadata {
    fn from_request(
        provider: &str,
        model_config: &ModelConfig,
        messages: &[Message],
        tools: &[Tool],
        data_classification: DataClassification,
        finding_count: usize,
    ) -> Self {
        Self {
            provider: provider.to_string(),
            model: model_config.model_name.clone(),
            message_count: messages.len(),
            tool_count: tools.len(),
            data_classification,
            finding_count,
        }
    }
}

/// The only Provider implementation exposed by the registry to the agent.
/// The actual provider, including any credential-bearing HTTP client, remains
/// behind this private field and is never handed to the UI or agent logic.
pub struct ModelGatewayProvider {
    inner: Arc<dyn Provider>,
}

impl ModelGatewayProvider {
    pub(crate) fn new(inner: Arc<dyn Provider>) -> Self {
        Self { inner }
    }

    fn metadata(
        &self,
        model_config: &ModelConfig,
        system: &str,
        messages: &[Message],
        tools: &[Tool],
    ) -> GatewayCallMetadata {
        let dlp = scan_model_input(system, messages, tools);
        GatewayCallMetadata::from_request(
            self.inner.get_name(),
            model_config,
            messages,
            tools,
            dlp.classification,
            dlp.finding_count,
        )
    }

    /// Emits only non-content metadata for the local audit path.
    fn record_start(&self, metadata: &GatewayCallMetadata) {
        tracing::debug!(
            provider = %metadata.provider,
            model = %metadata.model,
            message_count = metadata.message_count,
            tool_count = metadata.tool_count,
            data_classification = ?metadata.data_classification,
            finding_count = metadata.finding_count,
            "model gateway request"
        );
    }
}

#[async_trait]
impl Provider for ModelGatewayProvider {
    fn get_name(&self) -> &str {
        self.inner.get_name()
    }

    fn provider_session_id(&self) -> Option<String> {
        self.inner.provider_session_id()
    }

    async fn resume(&self, session_id: &str) -> Result<(), ProviderError> {
        self.inner.resume(session_id).await
    }

    async fn stream(
        &self,
        model_config: &ModelConfig,
        system: &str,
        messages: &[Message],
        tools: &[Tool],
    ) -> Result<MessageStream, ProviderError> {
        let metadata = self.metadata(model_config, system, messages, tools);
        enforce_dlp_boundary(self.inner.get_name(), &metadata)?;
        self.record_start(&metadata);

        // This is deliberately the only call to a concrete Provider for model
        // inference in the wrapper. Policy confirmation, redaction, and
        // budget settlement remain separate gateway inputs.
        self.inner
            .stream(model_config, system, messages, tools)
            .await
    }

    async fn get_context_limit(&self, model: &str, override_limit: Option<usize>) -> usize {
        self.inner.get_context_limit(model, override_limit).await
    }

    fn retry_config(&self) -> RetryConfig {
        self.inner.retry_config()
    }

    async fn fetch_supported_models(&self) -> Result<Vec<String>, ProviderError> {
        self.inner.fetch_supported_models().await
    }

    async fn fetch_supported_model_info(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        self.inner.fetch_supported_model_info().await
    }

    async fn fetch_model_info(&self, model_name: &str) -> Result<ModelInfo, ProviderError> {
        self.inner.fetch_model_info(model_name).await
    }

    fn skip_canonical_filtering(&self) -> bool {
        self.inner.skip_canonical_filtering()
    }

    async fn fetch_recommended_models(&self, toolshim: bool) -> Result<Vec<String>, ProviderError> {
        self.inner.fetch_recommended_models(toolshim).await
    }

    async fn fetch_recommended_model_info(
        &self,
        toolshim: bool,
    ) -> Result<Vec<ModelInfo>, ProviderError> {
        self.inner.fetch_recommended_model_info(toolshim).await
    }

    async fn map_to_canonical_model(
        &self,
        provider_model: &str,
    ) -> Result<Option<String>, ProviderError> {
        self.inner.map_to_canonical_model(provider_model).await
    }

    fn manages_own_context(&self) -> bool {
        self.inner.manages_own_context()
    }

    fn uses_local_session_naming(&self) -> bool {
        self.inner.uses_local_session_naming()
    }

    fn supports_builtin_tools(&self) -> bool {
        self.inner.supports_builtin_tools()
    }

    async fn configure_oauth(&self) -> Result<(), ProviderError> {
        self.inner.configure_oauth().await
    }

    async fn refresh_credentials(&self) -> Result<(), ProviderError> {
        self.inner.refresh_credentials().await
    }

    async fn update_mode(&self, session_id: &str, mode: GooseMode) -> Result<(), ProviderError> {
        self.inner.update_mode(session_id, mode).await
    }

    fn thinking_effort_support(&self) -> ThinkingEffortSupport {
        self.inner.thinking_effort_support()
    }

    fn subscribe_thinking_effort_support(&self) -> Option<watch::Receiver<ThinkingEffortSupport>> {
        self.inner.subscribe_thinking_effort_support()
    }

    async fn set_thinking_effort(
        &self,
        session_id: &str,
        value: &str,
    ) -> Result<bool, ProviderError> {
        self.inner.set_thinking_effort(session_id, value).await
    }

    async fn apply_model_selection(&self, model_config: &ModelConfig) -> Result<(), ProviderError> {
        self.inner.apply_model_selection(model_config).await
    }

    fn permission_routing(&self) -> PermissionRouting {
        self.inner.permission_routing()
    }

    async fn handle_permission_confirmation(
        &self,
        request_id: &str,
        confirmation: &PermissionConfirmation,
    ) -> bool {
        self.inner
            .handle_permission_confirmation(request_id, confirmation)
            .await
    }
}

struct DlpSummary {
    classification: DataClassification,
    finding_count: usize,
}

fn scan_model_input(system: &str, messages: &[Message], tools: &[Tool]) -> DlpSummary {
    let scanner = DlpScanner::new();
    let mut classification = DataClassification::L1Internal;
    let mut finding_count = 0;

    let system_report = scanner.scan(system);
    classification = classification.max(system_report.classification);
    finding_count += system_report.findings.len();

    for message in messages {
        let serialized = serde_json::to_string(message).unwrap_or_default();
        let report = scanner.scan(&serialized);
        classification = classification.max(report.classification);
        finding_count += report.findings.len();
    }

    let serialized_tools = serde_json::to_string(tools).unwrap_or_default();
    let report = scanner.scan(&serialized_tools);
    classification = classification.max(report.classification);
    finding_count += report.findings.len();

    DlpSummary {
        classification,
        finding_count,
    }
}

fn enforce_dlp_boundary(
    provider: &str,
    metadata: &GatewayCallMetadata,
) -> Result<(), ProviderError> {
    if metadata.data_classification >= DataClassification::L4Restricted {
        tracing::warn!(
            security.event_type = "model_egress_dlp",
            security.action = "BLOCK",
            security.reason = "credential_detected",
            provider = %provider,
            model = %metadata.model,
            data_classification = ?metadata.data_classification,
            finding_count = metadata.finding_count,
            "model gateway blocked restricted data"
        );
        return Err(ProviderError::InvalidValue(
            "Model request blocked by local DLP policy".to_string(),
        ));
    }

    if metadata.data_classification >= DataClassification::L2Sensitive
        && !is_explicitly_local_provider(provider)
    {
        tracing::warn!(
            security.event_type = "model_egress_dlp",
            security.action = "BLOCK",
            security.reason = "sensitive_data_requires_local_or_redacted_route",
            provider = %provider,
            model = %metadata.model,
            data_classification = ?metadata.data_classification,
            finding_count = metadata.finding_count,
            "model gateway blocked sensitive data"
        );
        return Err(ProviderError::InvalidValue(
            "Sensitive model input requires a local or redacted route".to_string(),
        ));
    }

    Ok(())
}

fn is_explicitly_local_provider(provider: &str) -> bool {
    let provider = provider.to_ascii_lowercase();
    provider.contains("local") || provider.contains("ollama") || provider.contains("vllm")
}

#[cfg(test)]
mod tests {
    use super::*;
    use goose_providers::base::stream_from_single_message;
    use goose_providers::conversation::message::MessageContentBlock;
    use goose_providers::conversation::token_usage::Usage;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct RecordingProvider {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Provider for RecordingProvider {
        fn get_name(&self) -> &str {
            "local-recording"
        }

        async fn stream(
            &self,
            model_config: &ModelConfig,
            _system: &str,
            _messages: &[Message],
            _tools: &[Tool],
        ) -> Result<MessageStream, ProviderError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let message = Message::assistant().with_text("ok");
            Ok(stream_from_single_message(
                message,
                ProviderUsage::new(model_config.model_name.clone(), Usage::default()),
            ))
        }
    }

    #[tokio::test]
    async fn inference_calls_are_delegated_through_the_gateway() {
        let calls = Arc::new(AtomicUsize::new(0));
        let provider: Arc<dyn Provider> =
            Arc::new(ModelGatewayProvider::new(Arc::new(RecordingProvider {
                calls: calls.clone(),
            })));
        let config = ModelConfig::new("test-model");
        let (message, usage) = provider
            .complete(&config, "system", &[], &[])
            .await
            .unwrap();

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(usage.model, "test-model");
        assert!(matches!(
            message.content.first(),
            Some(MessageContentBlock::Text(_))
        ));
    }

    #[test]
    fn call_metadata_contains_counts_but_no_content() {
        let metadata = GatewayCallMetadata::from_request(
            "provider",
            &ModelConfig::new("model"),
            &[Message::user().with_text("private document content")],
            &[],
            DataClassification::L1Internal,
            0,
        );

        assert_eq!(metadata.message_count, 1);
        assert_eq!(metadata.tool_count, 0);
        assert_eq!(metadata.data_classification, DataClassification::L1Internal);
        let debug = format!("{metadata:?}");
        assert!(!debug.contains("private document content"));
    }

    #[tokio::test]
    async fn gateway_blocks_credentials_before_provider_call() {
        let calls = Arc::new(AtomicUsize::new(0));
        let provider: Arc<dyn Provider> =
            Arc::new(ModelGatewayProvider::new(Arc::new(RecordingProvider {
                calls: calls.clone(),
            })));
        let config = ModelConfig::new("test-model");
        let result = provider
            .complete(
                &config,
                "system",
                &[Message::user().with_text("api_key=secret-value-123")],
                &[],
            )
            .await;

        assert!(matches!(result, Err(ProviderError::InvalidValue(_))));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn gateway_accepts_redacted_sensitive_input_without_original_value() {
        let calls = Arc::new(AtomicUsize::new(0));
        let provider: Arc<dyn Provider> =
            Arc::new(ModelGatewayProvider::new(Arc::new(RecordingProvider {
                calls: calls.clone(),
            })));
        let redacted = DlpScanner::new()
            .redact("Contact 13800138000")
            .unwrap()
            .text;
        let config = ModelConfig::new("test-model");

        provider
            .complete(
                &config,
                "system",
                &[Message::user().with_text(redacted.clone())],
                &[],
            )
            .await
            .unwrap();

        assert!(!redacted.contains("13800138000"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn external_gateway_route_blocks_sensitive_classification() {
        let metadata = GatewayCallMetadata {
            provider: "openai".to_string(),
            model: "test-model".to_string(),
            message_count: 1,
            tool_count: 0,
            data_classification: DataClassification::L2Sensitive,
            finding_count: 1,
        };

        assert!(enforce_dlp_boundary("openai", &metadata).is_err());
    }
}
