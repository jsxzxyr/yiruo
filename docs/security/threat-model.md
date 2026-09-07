# Yiruo Threat Model

## Assets

User documents, identities and PII, API credentials, redaction maps, knowledge
base boundaries, draft history, provider configuration, token/cost records,
audit integrity, and generated official documents.

## Adversaries

- malicious or compromised MCP server;
- prompt injection in a document, web page, search result, or tool result;
- remote provider or relay retaining more data than expected;
- malicious package or child process launched as an extension;
- another local process or task attempting cross-workspace access;
- accidental user export, overwrite, deletion, or provider selection;
- compromised update, telemetry, or diagnostics endpoint.

## Main abuse cases and controls

| Abuse case | Required control |
|---|---|
| Sensitive file sent to remote model | Classification, DLP, provider matrix, fail-closed Gateway, confirmation |
| Injection asks agent to upload a file | Untrusted-content typing, tool policy, network allowlist, no authority from content |
| MCP reads another KB | Task-scoped capability, canonical path/KB ID check, deny-by-default |
| Shell exfiltrates data | Shell disabled by default, sandbox, no network, command approval, OS enforcement |
| Key appears in prompt/log | Secret process isolation, redaction, secret scanning, negative tests |
| Full diagnostics leak session | Metadata-only default, sanitized export, explicit sensitive-content warning |
| Cost runaway | Atomic reservation, per-task/provider/day/month budgets, hard stop |

## Residual risks

Local administrator compromise, malicious signed application updates, provider
side retention outside Yiruo control, and model hallucination cannot be solved
by prompt text alone. Yiruo must disclose these limits and reduce blast radius
through least privilege, local-first routing, provenance, review, and audit.
