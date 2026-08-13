# An Agent Recording Is Keyed By The Prompt And The Model

Given a `<hick:agent>` cell whose answer is recorded in
`.hick-cache/transcripts/`, when that recording is looked up, then the key is a
hash of **the prompt and the model together**. Editing either retires the
recording; a retired recording is a missing baseline, never a stale answer
served as a current one.

`hick_literate::cache::agent_cache_key(model, prompt)` computes it. It uses a
different domain separator (`agent:`) from `exec_cache_key`'s (`image:`), so an
agent cell and an exec cell cannot collide even on byte-identical text.

## Why both, and why not more

A recording answers "what did this cell produce last time". For an agent cell
that question is only well posed once you say *what was asked* and *who was
asked*. Key on the prompt alone and editing the prompt serves the old answer —
freeze then verifies a claim the document no longer makes, which is worse than
no verification because it looks green. Key on the model alone and the same
failure happens in the other direction.

`max-turns` is deliberately **not** in the key. It bounds how hard the cell may
try, not what it was asked; a recording made under a larger budget is still an
honest answer to the same question.

## The model must be nameable before the cell runs

A lookup needs a key, and a key needs a model, *before* anything has run. So
the model comes from, in order:

1. the cell's `model=` attribute, when declared;
2. otherwise, the configured runner's `model_name()`.

With neither there is no key, and the cell is reported unverifiable. This is
why **a cell meant to be replayable without credentials should declare
`model=`**: `hick weave`, a dry run, and `hick test` all have no runner
to ask, so only a declared model lets them find the recording. A runner that
is handed a cell naming a model it is not running refuses rather than
substituting one — the substituted answer would be filed under a key the
document never asks for.

## The honest cost

Everything `freeze-is-declared-per-cell.md` says about the cost of freeze
applies here and applies harder: a frozen agent cell's `hick:expect` passes
trivially, and "the document still produces what we recorded" is a much weaker
claim for a nondeterministic cell than for `cargo --version`. Freeze is
nevertheless the *only* honest verification stance for an agent cell, because
re-running one and comparing would fail on wording alone.

---

Last LLM verification:
- Date: 2026-08-10
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hick-literate/src/cache.rs` — `agent_cache_key` hashes
  `agent:` + `model:` + `prompt:` (trimmed). `crates/hick-literate/src/lib.rs`
  — the agent arm of `run_pipeline_live`'s topological loop resolves the model
  as `agent.model.or_else(|| config.agent_runner.model_name())`, looks up
  `cache::cache_lookup(cc, &container, &agent_cache_key(..))` under the cell's
  reserved synthetic container name, and stores under
  `agent_cache_key(&outcome.model, &agent.prompt)` after a real run.
  `run_pipeline_weave` replays an agent cell only from a declared `model=`,
  because it has no runner to ask. `LlmAgentRunner::run`
  (`crates/hickory-cli/src/agent_cell_runner.rs`) refuses a declared model it
  is not configured for.
- Test coverage: `crates/hick-literate/src/cache.rs` —
  `agent_key_deterministic`, `agent_key_changes_with_prompt`,
  `agent_key_changes_with_model`, `agent_and_exec_keys_never_collide`.
  `crates/hick-literate/tests/agent_cells.rs` —
  `a_recorded_agent_cell_replays_without_a_runner`,
  `a_changed_prompt_retires_the_recording`.
  `crates/hickory-cli/tests/agent_cell_vertex.rs` —
  `a_cell_naming_another_model_is_refused`,
  `a_recorded_agent_cell_verifies_without_a_model`.
- Caveat requiring LLM review: the key does not include the tool surface or the
  system prompt, both of which change what a given prompt produces across
  builds. A recording made by an older build is therefore served to a newer one
  with a different tool set. That is acceptable while the tool surface is
  changing every week and the property being verified is "these committed bytes
  came from this document"; revisit it if agent recordings ever become
  long-lived evidence.
