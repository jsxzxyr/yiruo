# Token Management

## Current capability

Goose tracks per-session and accumulated usage and has automatic context
compaction, tool-pair compaction, and configurable automatic compaction
thresholds. The documented default `GOOSE_MAX_TURNS` is 1000 and the default
output limit is 4096 in the inspected configuration. These are useful building
blocks, but they do not implement Yiruo's daily, monthly, provider, or task
budgets, nor do they bound all tool/search/document inputs.

## Yiruo context budget

```text
TaskBudget {
  max_input_tokens
  max_output_tokens
  max_tool_tokens
  max_history_tokens
  max_document_chars
  max_search_results
  max_turns
  max_cost
  max_wall_time
}
```

The budget ledger reserves before a call and settles using provider usage or a
conservative estimate. A failed or timed-out request releases only the portion
that was not consumed. Daily, monthly, per-task, and per-provider counters are
transactional in SQLite. Exceeding a budget stops execution and asks the user;
it never silently selects a more expensive provider.

## Compression strategy

- Parse documents deterministically and select sections before inference.
- Use hierarchical summaries for long documents, retaining source spans.
- Strip navigation, boilerplate, duplicate search results, and irrelevant HTML.
- Compress tool results to typed fields plus bounded evidence.
- Replace old conversation turns with a structured summary after validation.
- Keep the latest user intent, policy decisions, unresolved questions, and
  provenance outside the compressible text.

Every model request records estimated and actual token counts without storing
the prompt body.
