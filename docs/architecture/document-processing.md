# Document Processing Architecture

## Pipeline

```text
select file -> verify type/size -> hash -> parse locally -> normalize
-> extract sections/tables/metadata -> classify and scan
-> chunk with source spans -> local/hybrid retrieval
-> bounded model operation -> deterministic validation
-> user review -> DOCX/PDF export confirmation
```

PDF, DOCX, XLSX, PPTX, TXT, Markdown, HTML, images, and scans are parsed by
deterministic adapters first. OCR is a local worker where available. Parser
workers receive one file or bounded byte ranges and cannot access unrelated
workspace paths or the network.

## Content controls

Each chunk carries file hash, page/section/cell/slide span, classification,
language, extraction confidence, and injection findings. A prompt injection
finding changes handling and may require user review; it does not become an
instruction. Tables and entities are represented as structured data so the
model does not need the full source repeatedly.

## Document operations

`read`, `extract_text`, `extract_table`, `extract_metadata`, `summarize`,
`compare`, `rewrite`, `proofread`, `generate_docx`, and `generate_pdf` are
typed application operations. Only content selection, interpretation, and
language generation use a model. Parsing, OCR, diffing, token counting,
validation, and export are deterministic whenever possible.

## Sensitive data

Default classification is L1_INTERNAL. L2-L4 content is routed to a local model
or remains local unless a provider policy explicitly permits a redacted form
and the user approves. Reversible placeholders use a local encrypted mapping;
the remote request contains placeholders only.
