# Ingest Does Not Require The Rest Of The Document To Already Pass

Given a document whose target cell (the one named by `hick ingest --from
'#id'`) does not depend on some OTHER cell elsewhere in the document, when
that other cell currently fails, then the ingest still succeeds — it runs
only the target cell and its real dependencies, never the whole document.

## Why

`ingest_from_exec` used to call the ordinary whole-document `run_doc`, and
`run_doc` propagates any cell's failure as the whole run's failure. That
makes ingest a genuine chicken-and-egg problem for the ordinary authoring
order: a scaffolder cell is usually written FIRST, before the rest of the
document that will depend on its output exists — a self-test cell later in
the same document, say, that has nothing not yet been written to test. The
scaffolder cell being ready to ingest and the self-test cell not yet
existing at all is not a contradiction; requiring the whole document to
already pass before ingest would touch anything treated it as one.

## What changed, and what did not

`hick_exec::dag::FlowDag::predecessors` already existed and returns a
cell's DIRECT predecessors; `ingest_exec::transitive_predecessors` walks it
to a fixpoint, giving the target cell's full dependency closure. That
closure — never a partial one — becomes `PipelineConfig::subset`
(`crates/hick-literate/src/lib.rs`): a cell whose `ExecId` is outside the
set is never visited by the exec loop at all, so it is not run, not
required to succeed, and — because `upstream_keys` only ever looks up keys
for cells that were actually visited — never queried for a key that was
never computed. This is safe specifically because the subset is always a
transitive closure: every cell that survives the check has all of its own
predecessors surviving it too.

`hickory_cli::run_doc_cached`'s existing signature and behavior are
completely unchanged — it now delegates to a sibling,
`run_doc_subset(..., subset: Option<HashSet<ExecId>>)`, passing `None`,
which is the ordinary whole-document run for every other caller in the
codebase. `ingest_from_exec` is the only caller that ever passes `Some`.

## What this does not fix

A cell that fails only because THIS ingest hasn't happened yet, and that
the target cell's own subgraph genuinely depends on, is not solved by
this — that dependency is real, and running only the subgraph does not
manufacture output the target itself has not produced yet. What this fixes
is specifically the case where the failing cell has no dependency
relationship with the target at all, which was refused for an unrelated
reason before this existed.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Sonnet 5)
- Result: verified
- Evidence: `crates/hick-literate/src/lib.rs`'s `PipelineConfig::subset` and
  the skip check at the top of the exec loop's cursor iteration.
  `crates/hickory-cli/src/lib.rs`'s `run_doc_subset` (with `run_doc_cached`
  now a thin wrapper passing `None`). `crates/hickory-cli/src/ingest_exec.rs`'s
  `transitive_predecessors` and its use in `ingest_from_exec` to build the
  subset from the real DAG (`hick_exec::dag::build_dag`), matching the
  target `ExecInfo` by `(container, source_line)`.
- Test coverage:
  `crates/hickory-cli/tests/ingest_from_subset.rs`'s
  `ingest_succeeds_even_though_an_unrelated_later_cell_is_still_broken` —
  drives the real binary against a document whose target cell comes first
  and a wholly unrelated, genuinely-broken cell comes after it. Confirms the
  premise (`hick run` on the whole document really does fail) before
  asserting `hick ingest --from` still succeeds and writes the expected
  `<hick:ingested>` block. Confirmed load-bearing by temporarily reverting
  to the whole-document call and observing the test fail at exactly that
  assertion. The full existing ingest suite (`ingest.rs`, `ingest_scaffold.rs`,
  `ingest_naming.rs`) still passes unchanged.
- Caveat requiring LLM review: only ordinary `hick:exec` DAG dependencies
  (mounts, copy/paste edges) are considered. If some OTHER kind of
  document-wide precondition exists that is not expressed as a DAG edge, it
  is not accounted for by this subset computation.
