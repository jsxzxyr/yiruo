# Storage Audit and Design

## Current storage

`crates/goose/src/session/session_manager.rs` creates a SQLite database with
`sessions`, `messages`, `usage_ledger`, schema versioning, and provider
inventory tables. `messages.content_json` and `metadata_json` hold serialized
conversation content; sessions also store working directory, provider/model
configuration, recipes, snippets, and usage. This is useful for continuity and
accounting but means the database is sensitive by default.

`session/diagnostics.rs` can export full session data. Session JSON/Markdown
export is also a deliberate raw-content egress path.

## Yiruo storage layout

```text
workspace/<task-id>/
  input/       selected immutable references or encrypted copies
  working/     intermediate artifacts
  output/      user-visible artifacts
  temp/        short-lived parser/OCR data
  cache/       content-addressed, classified, expiring cache
  audit/       metadata-only append-only events
```

SQLite stores task metadata, provider policy identifiers, usage counters,
knowledge metadata, and audit metadata. Raw sensitive text is stored only when
the user explicitly requests local retention, encrypted with a device-bound
key, and tagged with classification and retention expiry. The reversible
redaction map is separate, encrypted, and never placed in the normal database
or logs.

## Retention and deletion

Every artifact has an owner task, classification, purpose, and expiry. Temp and
cache data are cleaned on task completion and startup recovery. Delete and
overwrite are separate confirmation actions. A “forget task” operation removes
content, mappings, indexes, and derived artifacts, then records only a
non-content deletion event.
