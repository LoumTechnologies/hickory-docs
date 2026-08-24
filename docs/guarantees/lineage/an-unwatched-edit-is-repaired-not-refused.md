# An Unwatched Edit Leaves Its Correspondence Behind Before It Is Committed

Given continuity is on for a project, when a `.hick` document is staged whose
edit no recording site saw, then the pre-commit hook reconstructs what moved
and records it at **line** precision — and the commit is never blocked.

The workflow this product is built around — editing a generated file in your
own editor and letting the loop carry it back — is by definition unwatched. A
rule that rejected an unwatched edit would tax hardest exactly the thing the
product exists for. At commit time both sides are in hand, HEAD's document and
the working tree's, which is the same position `refactor/end` is in. So:

> **The check IS the repair.** There is no enforcement mode to reason about,
> and a rule with no escape hatch gets the hook disabled entirely, taking the
> `hick test` drift gate down with it.

Corollaries that are part of the guarantee:

- **The message says what it recorded, never what it refused.**
- **It exits zero whatever happens**, including when the journal cannot be
  written: the record is the feature, not a gate.
- **Two rungs, honestly labelled.** Recorded at edit time is byte-precise;
  reconstructed at commit time is line-precise — but recorded **once**, so it
  does not decay across later hops the way matching done fresh at query time
  would.
- **A moved run is one entry, not one per line**, so a block that moved reads
  as a block.
- **A run that did not move records nothing**, because `(commit, path, line)`
  already answers that. An edit that only changes a line's content in place
  therefore records nothing at all.
- **Off unless continuity is on.** No continuity, no journal, no check — and
  with it off the hook prints nothing and creates no directory.
- **The repair runs before the drift gate** and is `|| true`, so it can never
  take that gate down with it.

**Where the invariant genuinely breaks**, written as scope rather than
discovered: merges (which are their own, richer, recording site), your other
machines (a journal on the Windows box is not on the laptop), and the phone,
which edits and syncs by git with no loop watching. So the promise is *edits
you author locally* — never *continuity is always exact*, which would be
believed.

**Pre-commit is stricter than CI here, and that is not a parity violation.**
`pre-commit-ci-parity` forbids passing a hook and then failing CI; this is the
other direction, and this hook fails nothing.

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-cli/src/continuity.rs` — `repair` (line spans, equal-run
    matching, the did-not-move drop) and `repair_staged` (gated on
    `enabled`, reads `HEAD:<path>` and `:<path>`).
  - `crates/hickory-cli/src/main.rs` — the hidden `repair` subcommand, which
    reports and always exits zero.
  - `crates/hickory-cli/src/init.rs` — `hick repair || true` installed ahead
    of the drift gate in the managed hook block.
  - Tests: `crates/hickory-cli/src/continuity.rs` repair unit tests (4);
    `crates/hickory-cli/tests/continuity_repair.rs` (5, over a real
    repository, including the hook's ordering and the `|| true`).
- Caveat requiring LLM review: the repair keys its old side on HEAD, which is
  above the publication floor on a feature branch — so those entries are
  provisional and would need rewriting if frontier re-emission ever replaced
  that commit. Nothing re-emits today.
