# Bisect keeps candidates isolated

Given known good and bad commits, visual bisect uses Git's search in a private
worktree. Each candidate receives a fresh detached inspection worktree. Starting,
classifying, skipping, restoring, and ending never switch the original branch or
alter its tracked worktree or index. Earlier candidate windows remain at their
own commits when the search advances.

A stale candidate or a dirty inspection refuses a verdict. Preserving an
experiment records its binary patch and creates a fresh clean inspection of the
same commit, retaining the edited worktree. Untracked experiments refuse this
operation. A search ends with Git's first bad commit or an explicit ambiguous
skip result. Personal session metadata can resume a search after restart.

Ending removes the control worktree and retains inspections so open windows and
experiments are not deleted. Inspection cleanup is an explicit later Git act.

---

Last LLM verification:

- Date: 2026-10-03
- Reviewer: Codex
- Result: partially verified
- Evidence: `serve/bisect.rs`, `views/BisectPane.tsx`, and private Git-directory
  process locks in `up/mod.rs`.
- Test coverage: `tests/literate_views.rs` finds a seeded first-bad commit,
  protects a dirty original index, refuses a modified candidate, preserves an
  experiment, retains earlier candidate HEADs, resumes metadata, and handles
  ambiguous skips and reuse of an organized reading at a historical candidate.
  `e2e/literate-editor.spec.ts` drives the graph and Good/Bad controls through
  first-bad discovery and ending. Candidate windows use the existing shell seam.
- Limits: tests do not drive an OS window or classify the semantic correctness
  of the person's Good/Bad decisions.
