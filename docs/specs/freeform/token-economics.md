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

## Implementation

Everything below is built and tested; live runs additionally need
`ANTHROPIC_API_KEY` (never set in CI — network tests are `#[ignore]`d and
gated on `HICKORY_AGENT_LIVE=1`).

### Cost optimizations in `crates/hickory-agent`

- **Prompt caching by construction** (`llm_anthropic.rs`): system messages
  become system content blocks; the frozen protocol prompt is the FIRST
  system message and per-session doc context a SEPARATE second one, so the
  frozen prefix stays byte-identical across sessions. Breakpoints (max 4):
  first + last system block, a rolling breakpoint on the last content block
  of the latest turn, and an intermediate one ~15 blocks back on histories
  longer than the 20-block lookback. The request builder is deterministic
  (`AnthropicClient::request_body_bytes` exposes the exact bytes; the E3
  tests assert byte-stability and catch an injected timestamp).
- **Prefix-minimum assertion**: on the first request per client a background
  `count_tokens` probe logs (debug/warn) whether the system prefix clears
  the model minimum (1024 tokens on sonnet-5, 512 on opus-5/fable-5) —
  below it the API silently never caches. `verify_cacheable_prefix` exposes
  the same check; the live E3 test asserts it before asserting
  `cache_read_input_tokens > 0` on turn 2.
- **Usage capture**: every call reports `Usage` split four ways (input,
  cache write, cache read, output); `usage.rs` prices it per model
  (sonnet-5 $3/$15, haiku-4-5 $1/$5, opus-5 $5/$25 per MTok; cache write
  1.25x at 5m TTL, read ~0.1x; unknown models cost `None`, never a guess).
  Per-turn and session totals flow to `AgentEvent::TurnUsage` (live spend
  for server/UI), to the session log as self-closing `<hick:usage .../>`
  elements (unknown tags — old parsers skip them), and to
  `AgentOutcome::{total_usage, total_cost_usd}`.
- **Effort**: `AnthropicClient::with_effort(low|medium|high|xhigh|max)`
  sends `output_config.effort`; omitted by default (API default high).
  Nothing ever sends `temperature`/`top_p`/`top_k`/`thinking`/
  `budget_tokens` (400s on sonnet-5); adaptive thinking stays on by
  omission. Tested.
- **Long-loop hygiene**: `with_context_editing(true)` opts into the
  `context-management-2025-06-27` beta with `clear_tool_uses_20250919`
  (off by default; it targets API-native tool_use blocks, so it becomes
  effective when the loop moves from the text `<hick:tool>` protocol to
  native tool blocks). Compaction (beta `compact-2026-01-12`, which
  requires appending the FULL `response.content` back into history) is the
  documented next step if sessions approach context limits — not yet
  implemented because history here is plain text, not content blocks.
- **Batch API** (`llm_batch.rs`): `AnthropicBatchClient`
  (submit / wait_until_ended / results keyed by `custom_id`) is the
  complete 50%-discount transport for `hickory check` fan-out (E5). Wiring
  it into the `check` command lives in `hickory-cli` and is left to that
  workstream; note that concurrent identical-prefix requests cannot read a
  cache entry still being written — warm the cache with one request first.

### Harness (`hickory_agent::harness` + `experiments/token-economics/`)

- Committed specs: `tasks/e1-edit-surface.json` (12 edit tasks, script-only
  baseline vs tools), `tasks/e2-read-surface.json` (doc-first vs
  outputs-first), `tasks/e4-effort-sweep.json` (low/medium/high/xhigh).
  Fixed seed corpus under `corpus/` (grand tour, text-tools tour,
  bootstrap-ci); each task gets a fresh copy, `{corpus}` in prompts/checks
  expands to it. E3 is tests (below), not a runner arm.
- Runs land as JSONL under `runs/` (per-turn rows + a task-summary row,
  each carrying the four-way split). The report renders medians, p90,
  turns, check-pass rate, the token split, and a verdict per arm vs the
  baseline — including the plain-words "NO significant benefit" line when
  the 25% threshold is not met. Tools arms open an `EditSession` on the
  task's staged document (`AgentConfig::doc_path`), enabling the full
  read_doc/read_output/edit_output/edit_doc/verify set; rows record
  `tools_requested`/`tools_active`, and the report prints a PENDING caveat
  for any row where a tools arm fell back to script-only.

### Commands

```sh
# Offline (no API key):
just tokens-report                                     # regenerate report.md
cargo test -p hickory-agent -- --test-threads=1        # includes E3 offline tests

# Live (ANTHROPIC_API_KEY required):
just tokens-run experiments/token-economics/tasks/e1-edit-surface.json
just tokens-run experiments/token-economics/tasks/e2-read-surface.json
just tokens-run experiments/token-economics/tasks/e4-effort-sweep.json
just tokens-report
just tokens-count examples/grand-tour.hick examples/grand-tour.md   # E2 static sizes
HICKORY_AGENT_LIVE=1 cargo test -p hickory-agent -- --ignored       # E3 live cache test
```

Task `check_cmd`s invoke `${HICKORY_BIN:-hickory}`; point `HICKORY_BIN` at
a built CLI (e.g. `target/debug/hickory`) for live runs.
