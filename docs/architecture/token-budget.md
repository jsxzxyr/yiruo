# Token Budget Boundary

Goose's TokenCounter estimates prompt cost and caches repeated counts. Yiruo
adds a separate TokenBudgetLedger for enforcement.

Before a model request, the ledger atomically reserves the estimated input and
output tokens against daily, monthly, per-task, and per-provider limits. A
successful call settles the reservation with actual usage and refunds the
difference. A failed or cancelled reservation releases all reserved tokens
when it is dropped. Requests that would exceed any scope are rejected before
the provider call.

ContextBudget keeps input, output, tool, search-result, document-character,
and history limits explicit. Context compaction and document chunk selection
must fit these limits before entering the Model Gateway. Token counts are
metadata only and are safe for audit display; prompts and document text are
not stored in the ledger.
