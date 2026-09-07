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
