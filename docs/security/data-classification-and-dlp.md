# Data Classification and DLP Design

## Levels

| Level | Meaning | Default external handling |
|---|---|---|
| L0_PUBLIC | Public or intentionally shareable | Provider policy may allow |
| L1_INTERNAL | Ordinary user/workplace material | Default new-file classification; policy-specific |
| L2_SENSITIVE | PII, internal plans, non-public business data | Local first; remote requires sanitize/policy |
| L3_HIGH_SENSITIVE | Important personal, personnel, financial, or regulated material | Local only by default |
| L4_RESTRICTED | Secrets, credentials, highly restricted government/enterprise material | Never remote |

Classification combines user declaration, file metadata, deterministic detectors,
document context, and conservative escalation. A model may suggest a level but
cannot lower it. Unknown or detector failure means the higher applicable level.

## DLP checks

Before model, search, MCP, export, or external request, inspect file type and
size, PII patterns, credentials, identifiers, URLs, embedded files, macros,
hidden text, and prompt-injection markers. Check both raw bytes and normalized
text. DLP returns findings and spans to the local policy service; logs store
only finding type/count and a content hash.

## Reversible redaction

```text
local source -> detector span -> [PERSON_001] placeholder -> remote request
remote response -> placeholder validator -> local encrypted map -> restored draft
```

Mappings are task-scoped, encrypted with a device-held key, access-controlled,
excluded from prompt, logs, telemetry, analytics, crash reports, and ordinary
database tables. The user can choose irreversible masking when restoration is
not needed. Placeholder collisions, malformed placeholders, and model leakage
of mapping guesses are treated as failures.
