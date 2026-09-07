# Search Gateway Boundary

Goose's existing web-search skill can use shell commands or external MCP
servers. Yiruo does not expose that path as a default capability.

The Search Gateway is the only planned entry point for remote search. Its
policy requires HTTPS, port 443, and an explicit domain allowlist. It rejects
localhost, private, loopback, link-local, unspecified, and multicast
destinations before a request is sent. Redirects must be re-authorized by the
same gateway; an HTTP client must not follow them implicitly.

Queries are trimmed and bounded at 2,000 characters. Results are capped at
Top-K 10, filtered against the destination policy, deduplicated by local
content hash, and reduced to a 2,000-character excerpt before later model
routing. Web content remains untrusted data and is scanned for prompt
injection before it can influence an agent plan.

The current module is a policy and compression boundary. It intentionally
does not implement an HTTP client; provider-specific credentials, DNS
resolution, response-size limits, and redirect handling belong in the
network adapter that calls this gateway.
