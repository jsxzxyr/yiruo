# Model Provider and Gateway Audit

## Current Goose behavior

Goose exposes provider abstractions in `crates/goose-provider-types` and
provider implementations in `crates/goose-providers`. Provider requests can
contain system instructions, conversation history, tool definitions, tool
results, images, and documents. `crates/goose-providers/src/api_client.rs`
provides HTTP transport, but it is not a Yiruo-wide egress or data policy
gateway.

The normal agent path and MCP sampling path can reach provider implementations.
This means adding a gateway at only one call site would leave a bypass.

## Target Model Gateway

The only public model interface inside the runtime is:

```text
ModelRequest {
  task_id, purpose, classification, provider_capability,
  bounded_messages, bounded_tools, attachments, budget_reservation
}
```

The Gateway performs, in order:

1. Validate caller identity and task scope.
2. Resolve the provider policy from the local registry.
3. Reclassify and scan every message, attachment, tool schema, and tool result.
4. Minimize content to the requested purpose and apply reversible redaction.
5. Reserve input/output/cost budget.
6. Ask for confirmation when policy requires it.
7. Resolve the secret inside the Gateway process.
8. Route through an allowlisted network client or a local model adapter.
9. Strip secrets and raw payloads from logs and return a typed response.

The Gateway owns provider keys, endpoint URLs, headers, retry policy, retention
metadata, and model capability declarations. The UI receives only masked
identifiers and policy summaries.

## Provider policy

Every provider record has independent rules for L0 through L4, original versus
redacted files, network access, tool calling, context retention, maximum token
and cost, and allowed purposes. A third-party relay is treated as a distinct
untrusted destination, even if it speaks an OpenAI-compatible protocol.

## MCP sampling

MCP `sampling` requests must be converted into the same `ModelRequest` and
cannot receive a provider object, key, or direct HTTP client. The MCP server
identity, workspace, and data classification are part of the policy input.

## Phase 2 implementation

`crates/goose/src/model_gateway.rs` now provides `ModelGatewayProvider`, a
Provider-compatible boundary that keeps the credential-bearing Goose provider
behind a private field. The three `ProviderEntry` construction paths (normal,
default-model, and working-directory providers) wrap their result before it is
returned to the agent. This also covers declarative providers because they use
the same registry entry path.

The wrapper emits only provider name, model name, message count, and tool count
to debug tracing. It never serializes system prompts, messages, tool schemas,
request headers, or provider objects. Provider capability and session methods
are delegated so existing Goose behavior remains intact while the agent sees
only the Provider interface.

This is an ingress boundary, not the final security decision. Phase 3 must add
the policy decision before delegation; Phase 4 must add classification, DLP,
PII detection, and redaction; Phase 9 must add budget reservation and
settlement. Direct providers constructed inside unit-test fixtures are test
dependencies and are not runtime registry outputs.
