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

- **Read-only is the default.** A target is written only if the client named it
  in `targets`, which is what it sends for a target the person opened for
  writing. A long-lived release branch sitting in the view must not silently
  receive shared edits.
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
