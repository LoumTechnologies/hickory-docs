# Re-Ingesting A Scaffold Is A Three-Way Merge Against A Base Recovered From Git

Given a document whose cell already carries an `<hick:ingested>` block, when
`hick ingest --from` runs on that cell again, then the fresh run is merged
against the bytes as they were ingested — with your own edits preserved — and
when the ingest that recorded them was never committed, then there is no base
and the re-ingest says so rather than inventing one.

Three sides, named the way a merge names them:

| | What it is |
|---|---|
| **base** | the bytes as they were ingested — recovered from **git** |
| **theirs** | what the scaffolder produced on this run |
| **ours** | the document now: those bytes plus your four lines |

**Where the base comes from is the load-bearing part.** The document holds
*ours*, and records the run's `sha256` — but the original bytes are gone,
because your edits overwrote them, and a hash verifies rather than
reconstructs. Git holds them: the base is this document at the commit that
introduced that fingerprint (`git log -S`, oldest match, then `git show`). That
is `expression-and-log.md`'s division of labour applied to somebody else's
bytes — **the document describes the present, git holds the past** — and it is
why the sequencing that design argues for on its own merits (the ingest is one
commit, your four lines are the next) is also what makes a re-ingest possible.

Corollaries that are part of the guarantee:

- **Set membership merges too, not just content.** A path only in the run is
  **added**; a path the run stopped producing is **removed** only if you had
  not touched it, and otherwise **kept and named** — deleting somebody's edit
  because a scaffolder changed its mind is not a decision a tool gets to make.
- **A conflict is written into the document, with markers, and exits non-zero.**
  Resolution belongs in the document, which is where the product wants every
  resolution to happen — and once resolved those are ordinary document bytes
  with ordinary provenance. The report says which files conflicted and reminds
  the reader that a scaffolder randomises things, so some of them are noise.
- **One `<hick:ingested>` element, replaced in place.** A second block beside
  the first would make one cell claim two runs produced it.
- **The fresh run is still readable even though it is not written.** A volume a
  document has ingested is deliberately not flushed over the document's bytes,
  so it arrives in `PipelineResult::ingested_volume_files` instead — needed by
  exactly one caller, this one, where it is *theirs*.
- **It is a recording site.** With continuity on, a re-ingest records a
  correspondence at **diff** precision, and that is not a lesser byte-precision:
  two runs of a scaffolder share no history, so no byte-precise thread exists to
  record even with the tool watching the whole time. This is the case that
  forces the distinction between *coarse because nothing was watching* and
  *coarse because nothing exact exists*.

What this does NOT claim: nothing marks a region volatile. A scaffolder
randomises a user-secrets id, a GUID, a timestamp, and those surface as
conflicts. That is deliberate sequencing — volatile regions are designed after
the merge has produced enough false conflicts to show what they actually look
like, rather than against imagined ones. Note also that `volatile` is already a
reserved frontmatter key meaning "this weave is a report", so the marking will
need a different name.

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-cli/src/ingest_exec.rs` — `RecordedIngest`,
    `base_from_git` (`git log -S --reverse`, then `git show`),
    `merge_one` (the fast paths, then `git merge-file`), `merge_ingests` (set
    membership), the in-place replacement, `record_reingest`, and the
    no-base refusal.
  - `crates/hick-literate/src/lib.rs` — `PipelineResult::ingested_volume_files`
    and the flush that fills it instead of `files`.
  - `crates/hickory-cli/src/main.rs` — the per-file report and the non-zero
    exit on conflict.
  - Tests: `crates/hickory-cli/tests/reingest_merge.rs` (9) — the keep-both
    case, the missing base, kept/removed/added set membership, the conflict
    and its markers, continuity off, continuity on, and the gitignore default.
- Caveat requiring LLM review: `base_from_git` finds the commit by searching
  for the fingerprint string. A document that mentions the same hash in prose
  would match it too. In practice a sha256 appears once, in the attribute that
  records it, but nothing enforces that.
