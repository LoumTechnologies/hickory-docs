# An Output Path Declared Twice Warns Before It Runs

Given a document that has already ingested a path — a `hick:ingested`
block's child `hick:file` records it — and a separate, top-level
`<hick:file path="P">` declares that exact same path again, when the
document is run, then a warning names both declarations and their source
lines before any cell executes — never a silent clobber or a competing
declaration nobody notices disagrees with the other.

## Why

`hick:file` content is written to host disk in one upfront pass before any
`hick:exec` cell runs, with no regard for document order — confirmed
directly in `crates/hick-literate/src/lib.rs`: the exec loop runs, and only
after it finishes does the separate "flush output volumes" pass run, while
`hick:file` bodies are resolved by an earlier, unconditional pass the exec
loop never touches. A path already recorded inside `hick:ingested` that also
gets a second, top-level `hick:file` declaration either loses that second
declaration the next time the scaffolder cell reruns with `--force`, or
leaves the two declarations disagreeing about what the file contains with
nothing to say so. This is not a hypothetical: it is what an author reaches
for first trying to give an ingested file an earlier, smaller "stub" version
before growing it — the single most natural move in an incremental tutorial —
found writing exactly that tutorial.

## What it deliberately does NOT check, and why a broader version was wrong

The check is an **exact path match** against paths a `hick:ingested` block
already records — not a directory- or prefix-level check. A first version of
this warned whenever a `hick:file` fell anywhere under an output volume's
declared prefix, on the theory that the whole prefix is "the scaffolder's
territory." Running it against a real ingest-based tutorial immediately
produced four false positives: ordinary hand-authored files
(`Todo.cs`, `TodoStore.cs`, `Cli.cs`, `SelfTest.cs`) living in the same
output directory as the ingested scaffold, at different paths — which is the
normal shape of a project, not a hazard. The corrected version only compares
exact paths already known from a `hick:ingested` record against paths a
`hick:file` separately declares.

This also means a **bare scaffolder cell that has not been ingested yet
cannot be checked this way**: its future output filenames are not known
until it runs, and this check happens before any cell runs. Catching that
earlier case would need to simulate the cell's output, which is exactly the
information a pre-run static check does not have.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Sonnet 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/lib.rs`'s `output_collision_warnings`,
  wired into `run_doc_cached` alongside `self_mounting_warnings` and
  `absolute_mount_warnings`. Verified end to end against the real
  ingest-based Todo-app tutorial built for this exercise: a clean `hick run`
  produces no warnings (confirming the corrected design has no false
  positives on real content), and deliberately appending a second
  `<hick:file path="app/Program.cs">` — the exact mistake this guards
  against — produces the collision warning naming both source lines.
- Test coverage: `output_collision_tests` in `crates/hickory-cli/src/lib.rs`
  (4 tests: the confirmed collision shape warns and names both the path and
  "already ingested"; hand-authored files beside an ingested one at
  different paths are silent — the regression test for the false-positive
  design; a document with no ingest at all is silent, since nothing is known
  to compare against; an ingested path with no competing declaration does
  not self-collide).
