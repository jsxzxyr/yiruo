use crate::policy_engine::DataClassification;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditDecision {
    Allow,
    Ask,
    Deny,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub task_id: String,
    pub timestamp: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub data_classification: Option<DataClassification>,
    pub token_usage: Option<u64>,
    pub tool_name: Option<String>,
    pub decision: AuditDecision,
    pub duration_ms: u64,
    pub status: String,
    pub payload_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuditRecorder {
    events: Arc<Mutex<Vec<AuditEvent>>>,
    max_events: usize,
}

impl AuditRecorder {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            max_events,
        }
    }

    pub fn record(&self, event: AuditEvent) {
        let Ok(mut events) = self.events.lock() else {
            return;
        };
        if self.max_events == 0 {
            return;
        }
        if events.len() == self.max_events {
            events.remove(0);
        }
        events.push(event);
    }

    pub fn snapshot(&self) -> Vec<AuditEvent> {
        self.events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }
}

impl Default for AuditRecorder {
    fn default() -> Self {
        Self::new(10_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_events_are_metadata_only_and_bounded() {
        let recorder = AuditRecorder::new(1);
        recorder.record(AuditEvent {
            task_id: "task".to_string(),
            timestamp: "now".to_string(),
            provider: Some("local".to_string()),
            model: Some("model".to_string()),
            data_classification: Some(DataClassification::L2Sensitive),
            token_usage: Some(10),
            tool_name: Some("document.read".to_string()),
            decision: AuditDecision::Allow,
            duration_ms: 2,
            status: "ok".to_string(),
            payload_hash: Some("hash".to_string()),
        });
        recorder.record(AuditEvent {
            task_id: "task-2".to_string(),
            timestamp: "later".to_string(),
            provider: None,
            model: None,
            data_classification: None,
            token_usage: None,
            tool_name: None,
            decision: AuditDecision::Block,
            duration_ms: 1,
            status: "blocked".to_string(),
            payload_hash: None,
        });

        let snapshot = recorder.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].decision, AuditDecision::Block);
        assert!(!format!("{snapshot:?}").contains("private document"));
    }
}
