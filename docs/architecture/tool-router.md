# Tool Router

`crates/goose/src/tool_router.rs` is the Phase 3 capability decision boundary.
File reads are allowed only for existing paths inside the canonical task
workspace. File writes, exports, and deletes inside that workspace require a
separate confirmation. Paths outside the workspace are denied, including
relative paths that escape through a parent component or symlink.

Shell, arbitrary network, browser, and MCP capabilities are denied by the
default policy. The router returns a decision and never executes a tool; the
execution adapter must consume that decision before invoking an extension.
This keeps capability policy separate from Goose's existing permission UI and
allows the legacy and state-machine Agent paths to share the same check during
migration.

The current tests cover workspace reads, outside-workspace denial, write
confirmation, and default network/shell denial. Provider-specific allowlists,
MCP identities, and user confirmation transport are subsequent integration
work, not implicit permissions in this router.
