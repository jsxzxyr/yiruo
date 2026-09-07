# Migration Plan

Phase 0 is this audit and is complete only when the user approves the design.
No Phase 1 implementation is authorized by this document.

| Phase | Scope | Exit gate |
|---|---|---|
| 0 | Source, data-flow, threat, and architecture audit | User review; pinned upstream baseline |
| 1 | Yiruo identity, UX shell, configuration, Provider Registry | Build, UI tests, branding/legal regression |
| 2 | Model Gateway and provider credential isolation | No direct provider bypass; egress tests |
| 3 | Policy Engine and Tool Router | Every model/tool side effect has decision |
| 4 | Classification, DLP, PII, reversible redaction | L0-L4 and redaction security suite |
| 5 | Document Agent and local parsers | File matrix, path isolation, output validation |
| 6 | Search Gateway and Search Agent | Allowlist, citation, injection, bounded Top K tests |
| 7 | Official Document Agent | Template/style/format validation and review gate |
| 8 | Knowledge Base | Per-KB tenancy and hybrid retrieval tests |
| 9 | Token optimization | Budget ledger, compaction, cost regression suite |
| 10 | Audit and security hardening | Threat-model closure and release review |

## Migration constraints

The legacy and state-machine agent paths are both frozen for behavior changes
until the Gateway and policy seams exist. No provider, MCP, logging, telemetry,
or filesystem refactor is accepted without tests for direct and ACP callers.
Legal files, third-party notices, and upstream license obligations remain in
the distribution.
