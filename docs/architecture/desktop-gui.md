# Desktop GUI Audit and Target UX

## Current Goose desktop boundary

The desktop application is Electron/React and talks to the Rust service through
ACP and IPC. `ui/desktop/src/preload.ts` exposes directory and file selection,
file path resolution, file writing, directory creation/listing, external URL
opening, settings, and application launching. These are legitimate desktop
features, but the exposed methods are broad enough that Yiruo must make the
main process the sole authority for path and network policy.

The desktop currently exposes both user-facing and developer-oriented concepts.
Diagnostics and session export can be initiated from UI code. The professional
mode may show security details, but the default mode must not require knowledge
of agents, providers, MCP, or tokens.

## Target information architecture

```text
Home
  New task | Recent tasks | Local knowledge bases
Task workspace
  Files | Request | Draft/result | Sources | Review/export
Settings
  Privacy | Model choices | File locations | Templates | Budgets
Professional mode
  Policy decision | Provider/model | Tokens/cost | Tools | Audit | Network
```

The primary user flow is document-first: select files, describe the outcome,
review a data-sharing summary, inspect the generated draft, then confirm export.
The UI names the operation in ordinary language and shows technical details
only on demand.

## IPC requirements

IPC handlers must use typed channel names, validate sender/window identity,
canonicalize paths, enforce workspace roots, apply file size/type limits, and
return metadata rather than raw secrets. Generic `writeFile(directory, content)`
and arbitrary `listFiles(path)` APIs should be replaced by task-scoped commands.
External URL opening must accept only policy-approved destinations.
