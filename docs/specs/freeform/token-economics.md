# Token economics: does the hickory approach actually cost less?

The product thesis has a measurable claim inside it: **anchoring agent edits in
document provenance is cheaper than scripting file edits, and the document
surface is cheaper to read than a whole repo.** That claim deserves numbers,
not vibes. This is the experiment design; results land in
`experiments/token-economics/`.

## Ground rules

- **`count_tokens` is the only measuring instrument.** Never estimate with
  tiktoken or character heuristics — it is a different tokenizer and wrong for
  Claude by 15–20%+ on prose, worse on code.
- **Cost is computed from real `usage`**, split four ways:
  `input_tokens` (full price), `cache_creation_input_tokens` (1.25× at 5-minute
  TTL, 2× at 1h), `cache_read_input_tokens` (~0.1×), `output_tokens`.
  A run that only reports "total tokens" is not a result.
- **Every arm runs the same task list, same model, same effort.** Only the
  variable under test changes.
- **Report per-task medians and the distribution**, not a single mean — agent
  runs are heavy-tailed and one runaway trajectory swamps an average.
- Fixed seed corpus, fixed task list, committed under
  `experiments/token-economics/tasks/`, so runs are comparable across dates.

## E1 — Edit surface: lineage tool vs script-writes (the core claim)

**Hypothesis.** For code edits inside a woven output, `edit_output` (hash-anchored
lines → provenance → source) costs fewer total tokens than the script-first path
(model writes a heredoc/sed script, executor runs it, transcript returns), because
the model emits an anchor plus replacement text instead of re-emitting file
content, and because a failed script costs a full retry loop.

- **Arm A**: tools enabled (`read_output` + `edit_output`/`edit_doc`).
- **Arm B**: script-only agent (tools disabled), same tasks.
- **Tasks**: 12 edits over the grand tour + weave demo — rename a symbol, change
  a constant, add a function to a woven module, fix a failing expectation,
  reword prose, add a new cell.
- **Primary metric**: total cost per completed task (USD, from `usage`).
- **Secondary**: turns to completion, output tokens, failure/retry rate,
  `hickory check` pass rate at the end (a cheap wrong answer is not a win).
- **Threshold to claim the result**: ≥25% median cost reduction with equal or
  better check-pass rate. Below that, report it as "no significant difference" —
  and say so publicly rather than quietly dropping the experiment.

## E2 — Read surface: document vs raw repo

**Hypothesis.** Reading a `.hick` document (prose intent + cells) costs fewer
input tokens to reach a correct edit than reading the generated files plus
hunting for context, because the document colocates rationale with code.

- **Arm A**: agent starts with `read_doc`.
- **Arm B**: agent starts with the woven outputs only (no document).
- **Metric**: input tokens to first correct edit; ratio of context read to
  context used (tokens read that never influenced an edit).

## E3 — Caching: does the harness actually hit cache?

Not a comparison — a **regression test with teeth**. The session's system
prompt + tool definitions are a stable prefix; if `cache_read_input_tokens` is
0 on turn ≥2 of any session, something is invalidating the prefix and the run
fails the test. Deliberate invalidator checks: timestamps in the system prompt,
non-deterministic tool ordering, mid-session tool-set changes.

Note the model matters here: our default `claude-sonnet-5` has a **1024-token**
minimum cacheable prefix (Opus 5 / Fable 5 are 512). A system prompt below the
minimum silently never caches — the test asserts we are above it.

## E4 — Effort sweep

Run the same task list at `low` / `medium` / `high` / `xhigh` and plot cost vs
check-pass rate. The deliverable is a **recommended default per operation**
(cheap doc lookups vs a multi-file refactor), not a single global setting.
Expect the curve to be non-monotonic in *cost*: higher effort often means fewer
turns, so total spend can fall as effort rises.

## E5 — Batch API for verification fan-out

`hickory check` across many documents is latency-insensitive. Route bulk
verification through the Batch API (50% discount) and measure realized savings
against the interactive path.

## Reporting

`just tokens-report` regenerates `experiments/token-economics/report.md` from
the raw JSONL, with a per-experiment table (arm, median cost, p90, turns,
pass rate) and the four-way token split. Honest-result rule: if an experiment
shows no benefit, the report says so — the number is the deliverable, not the
conclusion we wanted.
