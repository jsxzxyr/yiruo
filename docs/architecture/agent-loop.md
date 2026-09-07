# Agent Loop Audit and Target Design

## Current Goose loop

The new generic loop is in `crates/goose-agent/src/machine.rs`. It loads a
session, walks ordered operations, builds inference input from tools and prompt
parts, invokes inference, applies effects, and repeats until no operation
applies or the runtime yields. Goose's concrete state-machine operations live
under `crates/goose/src/agents/state_machine/`.

The repository also retains the legacy implementation in
`crates/goose/src/agents/agent.rs`. `GOOSE_STATE_MACHINE` controls the new path.
Until Yiruo removes the dual-path condition, every security-sensitive behavior
must be tested against both paths or the legacy path must be disabled.

## Risk in the current composition

The state machine's inference preparation collects tools and prompt parts from
operations. That is an extensible mechanism, but it does not itself classify
content, reserve budget, or guarantee that a later adapter cannot call a
provider directly. MCP sampling is an additional provider entry path.

Tool inspectors and permission managers can produce allow/ask/deny outcomes,
but they are advisory pipeline participants rather than an unskippable policy
kernel for all tool, file, network, export, and model operations.

## Yiruo loop

```text
INTAKE
  -> classify inputs and establish workspace
PLAN
  -> produce bounded, typed plan
SELECT_CAPABILITY
  -> choose deterministic operation, tool, or model capability
CHECK_POLICY
  -> classify payload, check DLP, destination, permission, and budget
EXECUTE
  -> only through Tool Router or Model Gateway
OBSERVE
  -> validate result, mark trust origin, scan for injection
COMPRESS
  -> summarize or rank bounded results; preserve provenance
REPLAN
  -> stop on budget, turn, risk, or repeated failure thresholds
FINALIZE
  -> deterministic validation, user review, export confirmation
```

Each transition produces a typed event. A failed policy check is terminal for
that action and is never converted into a model-visible success result. The
model sees a redacted policy outcome, not secrets or internal rule details.

## Loop limits

Yiruo requires all of the following limits: maximum turns, maximum tool calls,
maximum wall time, maximum retries per tool, maximum search expansion, maximum
input/output/tool tokens, and maximum cost. Defaults are finite and small; a
user confirmation may raise a limit for one task without changing global
defaults.
