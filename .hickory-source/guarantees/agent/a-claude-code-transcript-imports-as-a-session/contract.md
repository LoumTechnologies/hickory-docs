<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Claude Code Transcript Imports As A Session

For: Authors using an agent to change documents while retaining review control.

Given a Claude Code transcript (`~/.claude/projects/<project>/<session>.jsonl`),
when `hick ingest --from claude-code <file>` runs, then a `hick:session` document is
written to `sessions/<date>-<title>.md` that the product reads back as the
conversation it was: every prompt a turn with its `parent=` (the tree
survives), every reply's prose, reasoning, and tool calls with their full
input, every tool result, every token count — and what the harness put in
front of the model besides the conversation (compaction summaries, meta
prompts, hook output, attached files) as `hick:context`; the document parses,
opens in the app as bubbles with the work folded, and is offline, model-free,
and idempotent (the same transcript names one file). Everything not carried
is counted and printed, by reason; nothing is silently dropped.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/a-claude-code-transcript-imports-as-a-session.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
