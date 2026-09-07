# Yiruo 壹若 Architecture Overview

## Scope and baseline

This Phase 0 audit examines the Goose source archive obtained from the official
repository on 2026-09-07. The workspace baseline reports Goose version 1.49.0
from `Cargo.toml`; the archive was taken from `main`, but no immutable commit
SHA was recovered. Future implementation work must pin a tag or commit before
relying on this document as a reproducible design baseline.

Yiruo keeps the Goose runtime as a starting point, but the security boundary is
new. A Goose inspector, user confirmation dialog, or provider implementation is
not sufficient to enforce the Yiruo boundary unless every path passes through
the same policy services.

## Target layers

```text
Desktop UI / Professional UI
        |
Application workflows and task state
        |
Yiruo Agent Runtime (plan -> policy -> execute -> observe -> compact)
        |
Policy Engine <-> Audit Recorder <-> Budget Ledger
        |
Tool Router ---- Document / Search / Knowledge / MCP adapters
        |
Model Gateway ---- Provider Router ---- local or remote model
        |
Network Policy Gateway / OS credential store / isolated workers
```

Every data-bearing edge carries a task identity, workspace identity, data
classification, purpose, destination, and an allow/deny/ask decision. The
Gateway owns provider credentials and the agent receives only an opaque model
capability.

## Reuse boundary

The state-machine protocol, provider type conversions, context-management
algorithms, SQLite migration approach, MCP protocol adapters, and Electron ACP
transport are useful foundations. The direct provider call sites, extension
launch path, diagnostics export, request logging, telemetry, shell execution,
and unrestricted filesystem/network paths require isolation or replacement.

## Non-negotiable invariants

1. No remote model request without a Policy Engine decision.
2. No tool execution without a Tool Router decision.
3. Classification defaults to L1_INTERNAL, never public.
4. L2 and above cannot leave the device unless an explicit policy and user
   confirmation both allow it.
5. Prompt, document, web, and MCP content is untrusted data, never authority.
6. Secrets are resolved only inside the Gateway or credential adapter.
7. Audit records contain metadata and hashes, not raw content.
8. Budgets fail closed before provider or tool execution.
9. Workspace isolation is enforced by canonicalized paths and OS-level checks.
10. Telemetry, analytics, crash upload, and content capture are opt-in.

## Current evidence

- `crates/goose/src/agents/state_machine/` provides an ordered, re-entrant
  operation pipeline, but the legacy `agent.rs` path still exists and must be
  kept behaviorally aligned during migration (`AGENTS.md`).
- `crates/goose/src/session/session_manager.rs` persists conversations and usage
  in SQLite.
- `crates/goose/src/agents/extension_manager.rs` starts platform, builtin,
  stdio, and Streamable HTTP extensions.
- `crates/goose/src/session/diagnostics.rs` can assemble full session/config/log
  diagnostics, so Yiruo must replace its export contract before handling
  sensitive material.
