# Goose Reuse Matrix

| Goose area | Decision | Reason |
|---|---|---|
| `goose-agent` machine protocol | Reuse behind policy-aware runtime | Clean operation/inference/effect seams |
| State-machine operations | Reuse selectively, wrap side effects | Existing loop is useful; policy must be mandatory |
| Provider types and format adapters | Reuse | Multi-provider normalization and usage types |
| Provider HTTP clients | Refactor behind Model Gateway | Current transport is not data/egress policy |
| Context management | Reuse and extend | Compaction/token counting are useful foundations |
| Session SQLite | Reuse schema approach, redesign retention | Messages and model data are raw-content sensitive |
| MCP protocol client | Reuse protocol implementation, isolate adapters | Current transports are broad capabilities |
| Platform extensions | Replace or heavily restrict | In-process coupling and broad context |
| Stdio extensions | Isolate | Child processes inherit user authority unless sandboxed |
| Desktop ACP/IPC | Reuse transport, replace file/network API surface | Broad preload methods need task scoping |
| Security inspectors | Reuse as signals only | Findings are not a universal enforcement boundary |
| Request logs/diagnostics | Disable or redesign | Current forms can serialize raw payloads/config/session |
| Telemetry/analytics | Remove defaults and content capture | Yiruo requires explicit opt-in and metadata-only data |
| Goose branding/UI | Redesign within license obligations | Product is for ordinary office users |
