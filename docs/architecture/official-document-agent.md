# Official Document Agent

The Official Document Agent turns extracted local facts into a typed draft
contract. Supported document types include notices, requests, reports,
official letters, minutes, decisions, opinions, plans, summaries, speeches,
briefs, and research reports.

Each type has deterministic required sections and validation rules. The model
may propose a title, structure, and wording, but it cannot silently omit
required sections or lower the draft's data classification. Human review is
required before export.

Export is routed through the Tool Router. Destinations outside the task
workspace are denied, and writes inside the workspace require confirmation.
Overwriting an input document must remain a separate confirmation decision.

This phase defines the document contract and validation layer. DOCX/PDF
rendering and organization-specific typography templates are parser/export
adapters that must consume a validated draft and remain deterministic where
possible.
