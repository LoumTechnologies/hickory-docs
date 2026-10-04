# A commit reads as a literate change

Given a commit in History or Story, **Read commit** opens a read-only document
reading with its complete message above its changed files. Markdown reads as
prose; other text files read as code. Additions and removals use the same
`comparisonField` as literate Git comparisons and agent proposals. Removed
passages render through the literate formatting layer and are expanded in a
read-only comparison. They never enter the current buffer or its saved bytes.

The before and after sides are exact Git blob bytes. An initial commit has an
empty before side. Deleted files have an empty after side; renames name both
paths; binary files, non-UTF-8 blobs, and submodules are identified explicitly.
Merge commits compare with their first parent. Reading a commit never checks it
out, changes the index, writes files, or supplies a historical execution path.
The view has no save path. Its formatting carries no guessed provenance.

---

Last LLM verification:

- Date: 2026-10-04
- Reviewer: Codex
- Result: partially verified
- Evidence: `serve/commit_reading.rs`, `CommitReading`, `DocumentComparison`,
  `comparisonField`, `GitPane`, `HistoryLens`, and `representationTabs`.
- Tests: real HTTP `git_ops::commit_reading_preserves_messages_blobs_and_the_working_tree`
  checks initial commits, complete messages, Unicode, renames, additions,
  deletions, binary files, immutable reads, and invalid revisions. Existing
  `comparison.test.ts` protects restoration and buffer coordinates. The ACP
  browser flow creates a commit and reads its message and changed note.
- Visual evidence: the browser proposal and commit readings were inspected at
  1280 × 720. New readings open below the main document and reuse that pane;
  the Agent pane retains its full height and the decision stays beside the change.
- Limits: no receipt is inferred between historical files. The view shows
  complete text files, so large commits can take time to render.
