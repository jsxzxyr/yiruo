# Network Flow Audit

## Current flows

Goose can make provider HTTP requests through provider clients, create
Streamable HTTP MCP clients, open external URLs from the desktop, run shell
commands that can perform network operations, and contact auxiliary services
such as update, malware-check, telemetry, or configured classification
endpoints. These paths do not share one allowlist.

The existing egress inspector parses several shell command patterns, but a
detected destination is not equivalent to a network block. Shell commands can
also hide network activity in scripts, binaries, redirects, DNS, or encoded
payloads.

## Yiruo Network Policy Gateway

All remote traffic originates from a small set of adapters: Model Gateway,
Search Gateway, MCP Gateway, update service, and optional opt-in telemetry.
Each adapter uses a separate allowlist and purpose-specific credential. Direct
network calls from agent tools and arbitrary child processes are denied.

The Gateway checks scheme, resolved IP, host, port, certificate, proxy route,
request size, method, upload classification, redirect target, and response
size. DNS rebinding, localhost/private-address access, file URLs, and implicit
redirects are denied unless explicitly approved for a local adapter.

## User-visible disclosure

Before an external request containing user-derived data, the UI shows the
destination, provider, data level, minimized/redacted fields, estimated tokens,
and retention policy. A search request may send the query but never gets access
to arbitrary local files.
