# MCP and Extension Audit

## Current behavior

`crates/goose/src/agents/extension.rs` supports builtin, platform, stdio, and
Streamable HTTP extension configurations. `extension_manager.rs` starts stdio
processes with the configured environment and working directory, and creates
HTTP clients for configured URIs. A Streamable HTTP URI is therefore an
explicit network capability, not a restricted search-only capability.

Stdio servers run as child processes under the current user account unless a
container path is selected. Platform extensions run in-process and have the
strongest coupling to the agent. MCP tools can return content, request
sampling, expose resources, and potentially write files depending on the
server.

## Yiruo extension registry

Each server is registered with name, publisher/source, version, hash, transport,
allowed commands, allowed hosts, allowed paths, data levels, read/write/execute
capabilities, retention, and expiry. Installation and first use are separate
approval events. Unverified servers are disabled by default.

The Tool Router presents only the explicitly allowed tool subset. It passes a
capability token rather than the full runtime context and strips unrelated
workspace paths, credentials, and conversation history.

## Required MCP policy checks

- `READ_DOCUMENT`: path must be inside the task workspace and classification
  must be permitted.
- `WRITE_DOCUMENT`: destination must be inside output or working directories;
  overwrite requires confirmation.
- `READ_WEB`: host must be on the search/network allowlist.
- `NETWORK`: exact scheme, host, port, method, and purpose are checked.
- `SAMPLING`: goes through Model Gateway with the server identity attached.
- `EXECUTE`: denied by default; a short-lived approval and sandbox are required.

The server must never receive an API key or unrestricted environment. MCP result
content is untrusted and is scanned for prompt injection before it becomes
model context.
