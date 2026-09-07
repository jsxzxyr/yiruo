# Data Flow Inventory

## Provider flow

```text
user/file/tool result -> conversation -> provider request builder
-> system/history/tools/attachments -> provider HTTP client -> remote endpoint
-> response -> conversation/session/usage/log/UI
```

The critical finding is that provider payload construction can include all of
those content classes. A Gateway must be before request construction or own the
complete sanitized request.

## MCP flow

```text
agent -> extension manager -> builtin/platform/stdio/HTTP MCP
-> tool/resource result -> conversation
                         \-> sampling -> provider path
```

MCP is both a tool boundary and a possible model boundary. The sampling branch
must be routed through the same Gateway.

## File flow

```text
desktop picker / workspace -> parser or MCP -> message/document content
-> session DB, request log, diagnostics/export, provider, generated output
```

The parser and export paths are local but still sensitive. Raw content must be
classified and retained only under an explicit local-retention policy.

## Token flow

```text
request estimate -> budget reservation -> provider usage/estimate
-> usage ledger/session counters -> UI/audit aggregates
```

Token metadata is not document content, but cost and model identity are still
privacy and control data. It must be recorded without prompt bodies or keys.
