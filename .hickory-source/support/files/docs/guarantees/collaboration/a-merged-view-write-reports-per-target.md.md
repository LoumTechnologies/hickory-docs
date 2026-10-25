<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/collaboration/a-merged-view-write-reports-per-target.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `e5240eabbaa6e5d7dafc9e9200b32535bfe3f8be81ab3aa514b4a130544529a0`.

<book:entry path="docs/guarantees/collaboration/a-merged-view-write-reports-per-target.md">
# A Write Through The Merged View Reports Per Target, And Undo Reaches All Of Them

Given a merged view over several worktrees, when an edit is made through it,
then it is routed either to **one** target ("just here") or to **every** target
opened for writing ("shared"); every target reports for itself; and a target
that was not opened for writing is never written at all.

**A multi-target edit is not atomic.** Three targets accept the write and the
fourth is gone, read-only, or unwritable. Refusing the edit is intolerable and
pretending it landed is worse — so the answer carries a status per target and
says `partial` explicitly rather than leaving it to be inferred from counting.

Corollaries that are part of the guarantee:

- **Read-only is the default.** `targets` names comparison membership; a target
  is written only if the client also named it in `writable`, after the person
  opened it for writing. A long-lived release branch sitting in the view must
  not silently receive shared edits.
- **The view is rebuilt from disk at write time**, never from the client's
  copy: between opening the tab and typing, a source may have moved, and
  writing a stale rebuild would silently revert somebody.
- **A shared edit writes N branches in one keystroke.** That is what makes a
  shared region agreed by construction — it never diverged, so it cannot
  conflict later — and it is exactly why it is a sharp tool.
- **"Just here" means write to one target**, and nothing else. The gesture no
  longer wraps text into a conditional in a file, which is both easier to
  implement and easier to explain; the divergence it creates is the one the
  person asked for.
- **Undo across targets restores from the recorded before-bytes of every
  target the write touched** — including the ones the write did not reach. A
  half-applied edit that is then undone in three of four places is a state
  somebody reaches on the first day, so the before-bytes are returned with the
  write rather than reconstructed later.
- **A write routed at nothing is an error, not a no-op**: an unknown region
  index or a source not in the view is refused, because a write that landed
  nowhere must not look like one that landed.
- **A path that escapes the repository is refused.**

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hick-merge/src/nway.rs` — `Route`, `Write`, `rebuild`,
    `writes_for_edit` and its refusals.
  - `crates/hickory-cli/src/serve/merged.rs` — `POST /api/merged/write`
    (rebuild-from-disk, the `targets` allowlist, `TargetStatus`, `partial`,
    the returned `undo`), `POST /api/merged/undo`, `check_relative`.
  - Tests: `crates/hick-merge/src/nway.rs` write tests (5);
    `crates/hickory-cli/tests/merged_view.rs` (6 write cases over two real
    worktrees, including the not-opened-for-writing case and undo).
- Caveat requiring LLM review: intent state is not kept. The view is
  synthesized on open, so a divergence you made deliberately and one that
  drifted in are indistinguishable tomorrow. That is the next step of the
  design and is deliberately after writing exists, because a view that
  remembers nothing is correct — just forgetful.

## Continuation review (2026-10-12)

The old verification above is historical. Publication and undo now live in
`serve/merged_write.rs`. Both compare displayed/returned expected bytes before
writing; a subsequent user edit refuses undo. Undo records cover every attempted
target, including targets not reached. An unchanged target is already restored
and requires no write. Writes prepare bytes and permissions in a sibling temporary
file before atomic replacement. Live Markdown uses its room guard; plain files
do not create Markdown rooms.

`just test-literate-worktrees` passed twelve real HTTP/worktree tests and six
browser tests. A partial-write witness proves that the unreached side reports
restored without writing while a newer edit on the written side remains intact.
Per-target guards do not make a multi-target filesystem transaction. Current
dependency hashes and failure/correction evidence are in
`../literate-projects/comparison-integration-evidence.md`. Human judgment is not recorded.

### Source-bound region draft controller (issue #72)

Worktree region editing now exposes shared route/reset/apply controls and paged
state reading alongside the existing comparison editors. Drafts bind the parent
host snapshot plus region identity, path, values and writable membership. Changed
input retains the old draft read-only and requires explicit reset; it cannot
silently reuse a newer apply callback. Missing host snapshots disable writes.
Apply captures the actual edited target/text/route, excludes duplicate pending
writes and returns the actual callback reply. Per-target replies are separately
disclosed; a partial reply or transport failure never asserts rollback or
independent verification. Parent refresh failure after receiving a write reply
does not relabel that received write reply as a failed write.

Two focused UI cases with real CodeMirror comparison drafts and mocked apply
callbacks pass: native/model route/draft/apply sharing, duplicate exclusion and
partial per-target reply reading; and snapshot replacement, stale-handle refusal,
read-only draft retention and explicit reset. Strict TypeScript/file-length/diff
checks pass. These tests cover the region child; the containing view's loading,
selection, undo and late-response lifetimes remain separate integration work.
No user's worktrees, real host writes or native/provider releases are exercised.


### Exact versioned read membership (issue #72)

Current clients read `/api/v1/merged` with JSON-array target membership. Omitted
membership means all worktrees; `[]` means none; commas, spaces and Unicode in
individual names are not separators. Duplicate/unbounded/non-array selections
are refused. V1 preserves the exact repository-relative filename, including
surrounding spaces. The legacy comma-separated/trimmed read remains available
to legacy callers; current clients never downgrade to it. A missing v1 route
returns an app/engine update recovery message. Peer `view` grants permit only
the exact read endpoint, without adding write or subpath authority.

Focused validation: two selection-parser cases; one exact peer-read authority
case; one client all/empty/literal-name/no-downgrade case; and one real loopback
HTTP test using three temporary Git worktrees. The HTTP case reads a literal
comma-named worktree and a filename with surrounding spaces, distinguishes all
from none, refuses malformed CSV on v1 and verifies the fixture files remain
unchanged. No user worktrees, provider or released native build are involved.

This completes the containing-view UI integration gaps named in the previous
region-child checkpoint for the tested control paths. It does not certify the
entire #72 surface inventory, adversarial filesystem compare-and-swap, or the
native/provider/released/deployed acceptance witnesses.
</book:entry>

</book:doc>
