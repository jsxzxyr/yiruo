# Official Document Agent Product Design

## Supported document types

Phase 7 covers notice, request, report, letter, minutes, decision, opinion,
plan, work summary, speech, briefing material, research report, and work brief.

## Workflow

```text
material intake
-> facts, numbers, people, organizations, dates, sources
-> user confirmation of extracted facts
-> document type/template/style selection
-> structure generation
-> first draft
-> consistency and format validation
-> human review
-> polish
-> DOCX/PDF export confirmation
```

Templates are versioned local assets with structure, style guide, validation
rules, fonts, headings, page header/footer, sign-off, dates, and numbering.
The model fills a typed document model; deterministic rendering owns layout.
The agent must flag unsupported formatting rather than silently inventing it.

## Quality controls

Facts, numbers, names, units, dates, citations, and unresolved assumptions are
shown separately. The final document includes an internal provenance map when
the user requests it, while the exported document contains only the intended
content. Sensitive source data is not copied to logs or analytics.
