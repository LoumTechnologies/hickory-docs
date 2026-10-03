# The Pre-Commit Hook `hick init` Installs Names Which Outcome Blocked The Commit

Given a repository set up by `hick init`, when the installed pre-commit
hook blocks a commit because `hick test` exited non-zero, then the message
names the specific outcome that occurred — drift (exit `1`), not verified
(exit `2`), or a failed expectation (exit `3`) — and states the fix for that
outcome and only that outcome.

Specifically:

- exit `1` says DRIFT and tells the author to re-run `hick run <doc>` (or
  `hick refresh <doc>` for a stale `hick:transform`) and commit the result;
- exit `2` says NOT VERIFIED and tells the author to record a baseline by
  running `hick run <doc>` — a cell declaring `freeze="true"` runs once
  there and records itself — or stop freezing the cell;
- exit `3` says FAILED EXPECTATION and explicitly tells the author **not** to
  regenerate it away, because the document claims something untrue of its own
  output and a human has to decide whether the claim or the code is wrong.

When several documents fail, the hook reports the strongest outcome, using the
same precedence the CLI uses: verified(0) < drifted(1) < unverifiable(2) <
expectation failed(3).

This matters because the three outcomes have three different fixes, and one of
them is the opposite of another: drift is fixed *by* regenerating, a false
claim is destroyed by regenerating. A hook that says "documentation drift" for
all of them sends the author to the wrong fix for two cases out of three.

---

Last LLM verification:
- Date: 2026-10-03
- Reviewer: Codex
- Result: verified
- Evidence: `HOOK_BODY` in `crates/hickory-cli/src/init.rs` discovers tracked
  Markdown documents with NUL delimiters and passes them as quoted positional
  arguments through `xargs -0 sh -c`. Each document's test exit is captured and
  the strongest outcome gets its own message. The repair remains advisory.
- Test coverage: `crates/hickory-cli/tests/init_tests.rs` exercises a clean
  document, drift, a false expectation, an unrecorded frozen cell, filenames
  with spaces and (on Unix) newlines, an empty repository, and idempotent
  installation. Inputs are `.md`; drift is checked against a generated `.txt`
  file, because Markdown documents no longer generate Markdown siblings.
- Caveat: the repository's own `.githooks/pre-commit` runs repository checks;
  this guarantee covers the document hook installed into a user's repository
  by `hick init`. Platform-specific CI checks still need their target OS.
