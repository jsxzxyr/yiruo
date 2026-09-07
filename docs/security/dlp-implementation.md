# Phase 4 DLP Implementation

crates/goose/src/security/dlp.rs provides the deterministic local DLP
building block used by the model gateway.

## Classification

DlpScanner starts every request at L1_INTERNAL and only escalates. It
detects credentials, national identifiers, bank cards, phone numbers, email
addresses, and prompt-injection markers. Findings contain a type, byte span,
and classification, never a content snippet. Reports contain only a BLAKE3
hash for correlation.

The detector is intentionally conservative. It is not an authority for
lowering a user-declared level, and an unknown detector failure must remain a
deny or local-only decision in the policy layer.

## Reversible redaction

DlpScanner::redact replaces detected spans with collision-checked
[YIRUO_*_NNN] placeholders. The mapping is private to a task-scoped
RedactionMap; it has no Debug implementation and exposes only the source
hash, placeholder count, and restoration operation. Restoration rejects
unknown or malformed placeholders and refuses responses containing an
original sensitive value.

The current map is memory-only. Production persistence must use a separate
encrypted device-key store with task scoping and expiry. It must never use
session messages, ordinary SQLite tables, telemetry, analytics, crash reports,
or prompt content.

## Gateway enforcement

The Model Gateway scans every system prompt, message, tool schema, and tool
result before provider delegation. Credential findings are blocked for all
providers. Any other L2 or higher finding is blocked unless the provider is
explicitly local or the request has already been redacted. This conservative
default keeps the Phase 4 boundary fail-closed until provider-specific policy
confirmation and remote redaction routing are wired in.
