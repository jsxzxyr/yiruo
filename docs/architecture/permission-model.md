# Permission Model

## Current audit

Goose has tool permission levels and inspectors, including `always_allow`,
`ask_before`, and `never_allow`, plus security and egress inspection. The
current permissions are useful for compatibility, but they are principally
tool-oriented. They do not by themselves express data classification,
destination, provider retention, workspace tenancy, cost, or export policy.

## Yiruo decision model

```text
Decision = f(identity, task, capability, resource, action,
             data_level, destination, purpose, budget, user_consent)
```

Decision results are `ALLOW`, `DENY`, or `ASK`. `ASK` includes a human-readable
explanation and exact data summary. The default is deny for shell, arbitrary
filesystem, browser, network, external MCP, export, overwrite, delete, and
third-party transmission.

Permissions are scoped to a task and expire. “Always allow” is reserved for
low-risk deterministic operations such as reading a selected input file or
writing a new file inside the task output directory. It never grants access to
all files, all hosts, or all providers.

## Confirmation contract

The confirmation dialog displays provider, model, destination, classification,
file/content types, estimated tokens, estimated cost, retention setting, and
the exact requested action. Approval is bound to a request hash and cannot be
reused for a changed payload.

## Phase 3 implementation

The policy vocabulary and deterministic decisions live in
`crates/goose/src/policy_engine.rs`; capability decisions live in
`crates/goose/src/tool_router.rs`. Both services are side-effect free and
return metadata-only decisions. Runtime adapters must call them before model
delegation or tool execution. Wiring task classification, confirmation
transport, and both Agent loop paths is required before this phase can be
considered security-complete.
