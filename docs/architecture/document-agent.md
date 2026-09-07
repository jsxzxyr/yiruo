# Document Agent Boundary

The Document Agent is the trusted local preparation layer for document
workflows. It does not give an agent arbitrary file access and it does not
send raw files directly to a model.

Its first responsibilities are:

1. Authorize a read through the task workspace Tool Router.
2. Reject directories, over-sized files, and paths outside the workspace.
3. Identify the format and hash bytes locally.
4. Use a format-specific parser adapter for structured formats.
5. Classify extracted text through local DLP before model routing.
6. Split text into bounded, overlapping chunks with per-chunk hashes.

The initial implementation supports UTF-8 text, Markdown, and HTML
preparation. PDF, DOCX, XLSX, PPTX, and images are recognized but require
explicit parser adapters before they can enter the text preparation path.
Existing Goose MCP parsers are reuse candidates, but their outputs must enter
through this boundary and their write operations must remain confirmation
gated.

Chunks carry provenance through the parent path and character offsets. The
model gateway receives only the selected chunks after classification,
minimization, and policy evaluation. Raw bytes, full extracted text, and DLP
finding spans remain local.
