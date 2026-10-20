# The Merged View Is A Lens Over Several Worktrees, Never A Document

Given a file that exists in several worktrees of one repository, when a merged
view is opened over it, then regions **every** source agrees on appear once,
regions that differ appear as variants naming each source, and the view says
plainly that it is read-only by default and exists on disk nowhere. Explicit
region edits name the targets opened for writing and use the captured source
preconditions; they do not give the lens a save path.

**It is a lens, not a document** — one of four, named and given one rule in
`docs/specs/freeform/lenses.md` (2026-09-03): every block declares what it
views, how an edit gets home, and whether an edit is allowed right now. No
save path, no `.md` extension, no place
in the folder tree — the same category as a diff view, and nobody mistakes
`git diff` output for a source artifact. Getting this wrong reverses the
product's central claim: a document is the source of its generated files, and a
synthetic document assembled *from* files points the other way.

**The sources are PEERS** (decided 2026-08-23, and it is the decision the edit
routing waits on). Shared means agreed by **all** of them, and there is no
order, because the view removes the question. The alternative — a stack, where
each branch lands on the one below — makes "shared" ambiguous, and answering
that late would mean rewriting the routing.

Corollaries that are part of the guarantee:

- **Alignment is conservative on purpose.** A line is shared only when every
  source has it, aligned, in a position monotonic with its neighbours in every
  source. Nothing is fused on similarity. The failure of over-sharing is an
  edit in the wrong file; the failure of under-sharing is a little redundant
  typing, and this errs the second way.
- **Every source is reconstructible from the view.** Walking the regions in
  order and taking each source's own side of every variant gives that source
  back byte-for-byte — including a file with no trailing newline. That is the
  invariant that makes routing possible at all, and it is what a later writing
  step will rest on.
- **A shared region is agreed by construction**, so it cannot conflict later:
  it never diverged. A divergence that already existed does not disappear — it
  *appears*, where it can be resolved at leisure instead of at merge time.
- **"Across branches" means across worktrees.** A branch that is not checked
  out cannot be written to without going behind the working tree into the
  object database, which bypasses hooks and produces commits nobody watched. So
  every source is a real worktree on disk, from `git worktree list`.
- **A bare worktree is not a source** (there is no working tree to read), and a
  detached HEAD has no branch rather than a wrong one.
- **A worktree that does not have the file is named, not dropped.** "This
  branch does not have this file yet" is an answer somebody opened the view to
  get.
- **A path that escapes the repository is refused.**
- **A one-worktree repository says what a second one would show**, rather than
  rendering an empty pane.

What this does NOT claim: nothing writes through the view. Writing — one target
at a time, then shared, with per-target status and undo across targets — is the
next step, and it is deliberately after the alignment is proved, because a bad
alignment routes an edit silently into the wrong file.

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hick-merge/src/nway.rs` — `merged_view`, the all-sources-agree
    rule, the monotonicity pass that refuses to fuse a reordered line,
    `lines_with_endings`, and `Region`/`MergedView`.
  - `crates/hickory-cli/src/serve/merged.rs` — `worktrees` /
    `parse_worktrees`, `GET /api/worktrees`, `GET /api/merged`, the
    path-escape refusal, and `missing`.
  - `apps/web/src/views/MergedView.tsx` — the lens marker, the per-source
    variants, the missing-worktree line, and the one-worktree case. Reachable
    since 2026-08-24 via `openMergedTab` (one tab per path) and the
    "Compare across worktrees" action; before that it was written, tested and
    rendered by nothing, which its own tests could not have caught.
  - Tests: `crates/hick-merge/src/nway.rs` unit tests (9, including the
    round-trip invariant and the not-fused case);
    `crates/hickory-cli/tests/merged_view.rs` (5, over two real worktrees);
    `apps/web/src/views/MergedView.test.tsx` (6).
- Caveat requiring LLM review: N is not bounded. Four targets is legible;
  twelve is not, and nothing here says what happens when someone opens a view
  over every branch in a repository.


### Shared containing-view lifecycle (issue #72)

The worktree view now exposes paged displayed-region/state reading, membership,
explicit per-target write permission, region disclosure, complete-project
comparison, refresh and guarded undo through the same native/model controller.
Membership alone does not grant writing. File/membership lifetime changes mask
old data immediately, reset write permissions and retire old controls/results;
late reads/writes cannot adopt into a different file or retarget a cached call.
One-worktree/non-repository status views expose no hidden file bodies or detailed
controls. Complete-project inspection refuses a selected file that no longer
matches the displayed capture; it does not claim a frozen dependency tree.

A host write request invalidates the editable capture until refresh. Drafts
remain read-only while the parent is busy or needs refresh. Actual partial write
replies remain available when refresh fails; explicit undo uses their exact
returned guard bytes and original file. An earlier received reply is labeled
as earlier if a newer request fails, so its success is not attributed to that
failed request. No rollback, atomic multi-target write or cross-process lock is
asserted by this UI integration.

Focused web evidence: five fixture-backed controller cases cover selection/
permission lifetimes, empty membership, late path-bound results, stale initial
reads, hidden status-view payloads, snapshot-mismatched project inspection,
partial replies, refresh failure and exact undo. One integrated UI case uses real
CodeMirror draft edits between native permission/disclosure and native undo,
then verifies read-only retention after an uncertain mocked write. Six earlier
view-presentation cases pass; agreement wording is now scoped to the capture.
Strict TypeScript/file-length/diff checks pass. Host writes in these UI tests are
mocked; they are not a real native/provider/released/deployed workflow.

The versioned selection route and its separate real HTTP read witness are
recorded in `a-merged-view-write-reports-per-target.md`.
