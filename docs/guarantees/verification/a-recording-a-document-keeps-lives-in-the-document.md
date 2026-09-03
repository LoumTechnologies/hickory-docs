# A Recording A Document Keeps Lives In The Document

Given a document whose cells are recorded, when `hick ingest --from
recording <doc>` runs, then each recorded cell's output is written into the
document as `<hick:ingested key="…" sha256="…" at="…">` inside the cell,
verbatim; the weave and `hick run` look there first and in the cache
second, match it by the same key the cache uses, and report it *stale* the
same way when the cell's inputs move; `hick run` brings a kept recording
forward when it re-executes the cell, and never adds one the document did
not keep; a clone with no cache weaves the same bytes; and nothing is
written unless the document with its recordings inside weaves, against an
empty cache, every byte the cache-backed weave did. `.hick-cache/` is a
cache again, whole, in `.gitignore`.

A recording is evidence a document makes about itself. Kept in the cache it
was a durable claim on a gitignored artifact — the `from=` mistake — and
the one day it was committed from there (2026-09-02) it was evidence in a
directory whose name says cache. Axis 1 and 2 of
`docs/specs/freeform/three-axes.md`: the document either owns its evidence
or points at something that may be gone.

## Boundary

An output containing `<prefix:` would parse as markup, and the body is
written verbatim by the language's one invariant, so such a cell's recording
is refused by name and stays in the cache. The documents' own bytes are not
in their cells' keys, or keeping a recording would stale it.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hick-literate/src/lib.rs` (`document_recordings`,
  `RefreshedRecording`, `PipelineResult::keys`/`refreshed`; document-first
  lookup in both `run_pipeline_weave` and `run_pipeline_live`; documents
  excluded from `unstable_products`); `crates/hick-lang/src/lib.rs`
  `HickDocument::all_tags`; `crates/hickory-cli/src/ingest_recording.rs`
  (`ingest_recordings` with the equivalence gate, `refresh_recordings`);
  `crates/hickory-cli/src/main.rs` (`--from recording`, refresh after
  `hick run`, weave always given the cache config);
  `crates/hickory-cli/src/init.rs` (the ignore line, and the fold-back);
  `crates/hick-literate/src/weave.rs` and `render.rs` skip a `key=` ingested
  as content.
- Test coverage: `crates/hickory-cli/tests/round_trip.rs`
  (`a_recording_kept_in_the_document_survives_without_the_cache_and_follows_a_run`);
  `crates/hickory-cli/src/ingest_recording.rs::tests`;
  `crates/hickory-cli/src/init.rs::tests`.
