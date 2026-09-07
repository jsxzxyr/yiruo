# Security Boundary

## Trust zones

1. **Local trusted core**: policy, classification, DLP, budget ledger, audit
   recorder, credential adapter, and task/workspace authority.
2. **Local untrusted workers**: parsers, OCR, previewers, and MCP stdio
   processes. They receive narrow capabilities and bounded data.
3. **Untrusted content**: user documents, images, web pages, search results,
   model output, tool output, recipes, and MCP resources.
4. **External services**: every remote model, proxy, search provider, MCP HTTP
   server, update server, telemetry endpoint, and classifier endpoint.

## Boundary rules

Content never changes trust level merely because a model labels it as an
instruction. All content entering the planner is wrapped as data with source,
classification, provenance, and injection status. Only signed application
policy and explicit user intent can create authority.

The core does not expose raw credentials to workers. Workers cannot select
arbitrary paths or destinations. A policy decision is made immediately before
the side effect and records the request hash, so stale approvals cannot authorize
new data.

## High-risk findings from Goose

- direct provider call paths outside a unified gateway;
- arbitrary Streamable HTTP MCP URI configuration;
- stdio and shell execution under the current user identity;
- session/database/log/diagnostic paths that can retain raw content;
- optional content-capturing OpenTelemetry attributes;
- advisory rather than mandatory egress inspection;
- desktop IPC methods accepting broad paths or external URLs.
