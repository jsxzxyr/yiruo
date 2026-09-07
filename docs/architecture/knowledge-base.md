# Knowledge Base Isolation

Knowledge bases are separate security scopes, not merely labels on one
shared search index. Each registered base has a canonical root directory.
Overlapping roots are rejected so one base cannot contain another.

Every read resolves relative paths against the selected base root and
canonicalizes existing paths or their parents before checking containment.
Paths outside the root, paths that traverse into another base, directories,
and unregistered base identifiers are denied.

The first implementation stores only local file metadata and a BLAKE3 content
hash. FTS and vector indexes are later adapters that must carry the base ID
as a mandatory partition key and enforce the same authorization check before
returning a chunk. A result from base A must never be used to satisfy a query
authorized only for base B.
