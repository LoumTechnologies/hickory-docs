# A Recording Travels With The Outputs It Produced

Given a repository set up with `hick init`, when `.gitignore` is written, then
`.hick-cache/*` is ignored and `.hick-cache/transcripts/` is not — so a cell's
recordings are committed beside the outputs they produced, and a fresh clone
weaves the same bytes it has in git rather than `[never run]`. A bare
`.hick-cache/` line from an earlier init is widened to `.hick-cache/*`,
because git cannot re-include a file under an excluded directory.

Everything else under the cache — installed language servers and adapters,
indexes, the CRDT store, build output — stays this machine's.

## Why

Outputs are committed and were meant to be reproducible from the document.
With the recordings gitignored, "reproducible" held only on the machine that
ran them: every other clone, and every CI job, had documents whose outputs it
could not re-derive and a weave that said so by writing four words over them.
A recording is keyed by the cell's inputs, so a committed one that no longer
matches is detected, not trusted; committing it costs nothing in honesty and
buys the clone the same weave.

---

Last LLM verification:
- Date: 2026-09-02
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hickory-cli/src/init.rs` (`run_init` writes
  `.hick-cache/*` and `!.hick-cache/transcripts/`; `replace_gitignore_line`
  widens the older line); this repository's own `.gitignore`, with its
  transcripts committed in the same change.
- Test coverage: `crates/hickory-cli/src/init.rs::tests`
  (`gitignore_lines_written_once`,
  `an_older_cache_ignore_is_widened_to_let_transcripts_through`).
