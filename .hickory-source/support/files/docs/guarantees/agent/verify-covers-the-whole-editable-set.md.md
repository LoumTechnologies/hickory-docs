<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/agent/verify-covers-the-whole-editable-set.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `4baebb23a38a46c1b392bd28d7af420a7dce2d5ddc88f7ddf6af9dfcf417a932`.

<book:entry path="docs/guarantees/agent/verify-covers-the-whole-editable-set.md">
# Verify Covers the Whole Editable Set

Given an agent session whose primary document declares `hick:upstream` edges,
when the agent runs `verify`, then every document in that closure is executed
and its output files are written **in its own directory** — and the result says
how many upstream documents were re-woven. A `PASS` means the whole chain is
consistent, not just the document in front of the agent.

Verification scope must equal edit scope. Once the agent could edit any
document in the closure, a `verify` scoped to the primary stopped being a
narrow check and became a false one: it reported `PASS` for work that had left
the rest of the chain stale.

This was observed exactly once, in a live run. An agent asked to propagate a
new task state amended the decision in the meeting note two hops upstream —
correctly — then ran `verify`, got `PASS`, and reported success in good faith.
`hick test` failed on six documents whose `.md` and tangled ticket outputs
had never been regenerated. The agent was not wrong to trust its tool; the
tool was wrong.

Each upstream document is executed in its own directory, because that is where
its committed outputs live. Weaving an upstream document into the primary's
directory would create files nobody checks while leaving the real ones stale —
the same failure in a new location.

---

Last LLM verification:
- Date: 2026-08-07
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-agent/src/tools/mod.rs` — `verify()` calls
  `verify_upstream()` before executing the primary; that walks
  `EditSession::upstream`, runs `run_pipeline_live` per document with
  `working_dir` set to the document's own parent, writes its output files
  there, and collects failed expectations. The failures are merged into the
  primary's, so any upstream problem turns the whole `verify` into a `FAIL`;
  the success message appends "also re-wove N upstream document(s)".
- Test coverage: `crates/hickory-agent/tests/edits_the_chain.rs` —
  `verify_reweaves_upstream_outputs_too` edits a decision two hops up, runs
  `verify`, and asserts both the report wording and that `decisions.md` and
  `domain.md` on disk carry the new decision. Confirmed live against
  `docs/todo-app/`: `PASS — … ; also re-wove 3 upstream document(s)`.

## Optional verified publication — October 21, 2026 continuation

The unconditional output-writing wording above describes ordinary verification.
Hosts can opt an `EditSession` into `require_verified_publication()`. In that
policy, primary and upstream outputs are provisional in memory until every
expectation in the editable scope passes. Failure creates no generated files,
preserves existing generated files, and does not replace session output reads
with failed execution output. Unavailable/stale execution evidence, an uncaptured
source dependency, or changed captured source bytes/closure refuses publication.
Upstream execution uses captured source bytes rather than silently re-reading a
different revision. Repair followed by verification can publish again.

The native project writer validates each result's whole path inventory and
existing destination symlinks before writing that inventory. Binaries are pinned
before writing. Publication across roots/files is not a filesystem transaction:
I/O errors can leave partial writes, and external writers can race validation.
Execution side effects are not rolled back. This is **not prediction strict
mode**: ordinary diagnostics and existing tool reads remain available. Default
sessions still publish observed outputs after expectation failure, as before.

A pipeline invocation now excludes transcripts retained by the executor from
earlier invocations, including stdin transcript rendering and final weaving.
Otherwise an upstream document's answer can be woven into the primary document
when both use the same container name. This assumes one pipeline at a time per
executor and append-only transcripts during that invocation; it does not reset
container files/process state or establish execution isolation.

LLM implementation review: **partially verified**. Inspected
`tools/verify.rs`, `tools/verify_publish.rs`, `hick-literate/src/run_transcripts.rs`,
the live pipeline call sites, and `hick-project-env/src/tree.rs`. Origin: #77's
provisional/accepted-output distinction under #72. A global silent behavior
change was rejected because #77 explicitly preserves ordinary workflows.
Tests in `crates/hickory-agent/tests/verify_publication.rs` cover failed existing
and absent artifacts, repair, upstream failure, distinct upstream/primary output,
source mutation during real execution, symlink refusal and default behavior.
The legacy `verify_executes_and_reports_expectations` check also passed.
These are actual LocalExecutor host-process witnesses, not sandbox/native/provider
or concurrent-filesystem witnesses. Human judgment: **not supplied**.

Executed focused checks and exact dependency hashes are recorded in the
[continuation handoff](../../developers/handoffs/2026-10-21-issue-72.md).
</book:entry>

</book:doc>
