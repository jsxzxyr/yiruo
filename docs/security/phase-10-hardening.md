# Phase 10 Security Hardening

Yiruo telemetry is opt-in. PostHog event submission returns before creating
or sending an event until the user has explicitly enabled telemetry. The
telemetry preference is local state; it is not a reason to bypass the
opt-in gate.

The security audit recorder stores bounded metadata only: task, timestamp,
provider, model, classification, token usage, tool name, decision, duration,
status, and an optional payload hash. It has no fields for prompts, document
text, tool arguments, tool results, credentials, or error bodies.

Release verification must run the security test plan against both the legacy
agent loop and the state-machine loop while both are enabled. The remaining
environment gate in this checkout is the Windows SDK required by the MSVC
linker; formatting and diff checks are available locally.
