# Yiruo Phase Roadmap

Phase 0: freeze the Goose 1.49.0 archive baseline, complete architecture and
threat documentation, and obtain user review. Do not start product coding.

Phase 1: add Yiruo identity, a user-first desktop shell, configuration schema,
Provider Registry, and legal attribution without changing safety assumptions.

Phase 2-4: build the security spine first: Model Gateway, Policy Engine, Tool
Router, classification, DLP, PII detection, and local reversible redaction.

Phase 5-8: add deterministic document processing, allowlisted search, official
document workflows, and isolated knowledge bases.

Phase 9-10: add context/token optimization, cost controls, audit UX, threat
model closure, penetration-style tests, and release hardening.

Every phase has separate commits named `phase-N-*` and must pass build, unit,
integration, security, and regression gates before the next phase begins.
