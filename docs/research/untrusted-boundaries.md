# Untrusted Boundary Register

| Boundary | Current exposure | Yiruo treatment |
|---|---|---|
| Remote provider | Sees request payload and may retain it | Policy matrix, minimization, Gateway, disclosure |
| Relay/proxy | Can inspect and alter model traffic | Treat as untrusted provider; no L2+ by default |
| HTTP MCP | Configured URI and server responses | Host allowlist, server registry, scoped data |
| Stdio MCP | Current-user process and environment | Sandbox, narrow env/cwd, no secret inheritance |
| Shell | Host command execution and possible network | Deny default; explicit sandbox/approval |
| Document/web content | Prompt injection and false instructions | Typed untrusted data, scanning, provenance |
| Session DB | Complete messages and tool results | Encrypt/retention policy; metadata-first design |
| Request logs | May serialize model config and payload | Metadata-only logger; raw payload disabled |
| Diagnostics | Full report includes session/config/logs | Sanitized summary default; explicit sensitive export |
| Telemetry/OTel | External export of metrics/traces/content | Off by default; no content capture |
| Desktop IPC | Broad file/network operations | Main-process authority and task-scoped methods |
