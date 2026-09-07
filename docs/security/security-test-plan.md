# Security Test Plan

## Required automated tests

1. L2/L3/L4 file to a remote provider is blocked.
2. L0/L1 file is allowed only when the provider policy allows it.
3. Redacted L2 request contains no original PII and local restoration works.
4. Injection text in a document or web page cannot invoke a tool or change
   policy.
5. MCP cannot read outside its task workspace or its assigned knowledge base.
6. Arbitrary URL, private IP, redirect, and shell-network attempts are denied.
7. API keys are rejected from prompts and redacted from errors/logs.
8. Full diagnostics and session exports contain no raw content by default.
9. Token, turn, search, tool, and cost budgets stop execution atomically.
10. Telemetry, analytics, crash upload, and OpenTelemetry content capture are
    disabled until explicit opt-in.

## Test layers

Unit tests cover classifiers, redaction, path canonicalization, budgets,
provider policy, and placeholder validation. Integration tests use a local fake
provider, fake MCP server, DNS/HTTP test server, and temp workspaces. Security
tests inspect the exact bytes received by those fakes. Regression tests run
legacy and state-machine paths until the old path is removed.

## Release gate

Build, unit, integration, security, UI, and regression suites must pass. The
release review checks the threat model, dependency licenses, update channel,
default network behavior, legal notices, and a manual confirmation walkthrough.
