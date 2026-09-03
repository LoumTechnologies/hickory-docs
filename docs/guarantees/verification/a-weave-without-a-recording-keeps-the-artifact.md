# A Weave Without A Recording Keeps The Artifact It Cannot Reproduce

Given a document that produces a file from a cell, when the file is written
after a weave (`hick weave`, or any writer that goes through
`write_outputs`), then a file **already on disk** whose bytes came from a cell
with no recording is **left exactly as it is**, and the writer says so by
name. The file is still created when it is not there yet — there is nothing to
destroy, and the `[never run]` marker is then the honest content of a document
that has never run.

**The weave target is kept too**, whenever it already exists and any cell has
no recording. It used to be exempt as "this weave's own report", and the
report it wrote over a committed rendering was `[never run]` in place of the
recorded output that rendering held: true about the recordings the weave
found, and a destruction of the ones the file had. A rendering that lags is
consistent with itself, the writer says it kept the file and why, and a run
brings it forward. (Amended 2026-09-02, after opening the app on this
repository re-wove fifteen committed renderings to `[never run]`.)

## Why

A weave never executes. A cell whose recording it cannot find is woven as
`[never run]`, and every file that cell fed carries the marker with it. On a
fresh clone — where `.hick-cache/` is gitignored and therefore absent — that
made `hick weave` replace a committed SVG with four words and exit `0`, while
the app, which executes rather than replays, went on rendering the chart from
a live run. Disk and app disagreed and nothing said so (issue #25).

The bytes on disk are the product of a run that really happened. A weave that
ran nothing has nothing truer to put in their place, so it keeps its hands
off. This is the same rule `write_missing_outputs` already states for staging:
a file already on disk is the committed baseline, and replacing it would
manufacture drift.

The exit code is unchanged: `hick weave` still exits `0` and reports the
never-run blocks on stderr. `hick test` is the verb that fails — a cell with
no baseline is *unverifiable* there
(`test-separates-unverifiable-from-drifted.md`).

## What this is not

It is **not** a line-shift fix. A recording is found by the cell's cache key —
image, capabilities, command, secret names, input digest, upstream keys
(`a-recording-is-keyed-by-the-cells-inputs.md`) — which contains no line
number at all. Editing prose above a cell does not orphan its recording, and
never did; issue #25's stated mechanism was wrong even though the symptom it
reported was real.


> **Amended 2026-09-03 (three-axes, step 1).** A cell whose recording exists
> but no longer matches its inputs is **stale**, not unrecorded: the weave
> shows its last recording (`cache::stale_lookup`, matched by the command it
> ran), reports it as stale, and writes the rendering — nothing turns into a
> marker. Only a cell with no recording at all is *unrecorded*, and that is
> the only case the keep rule above still fires for. The app's block status
> is `stale` or `unrecorded`; the word "never-run" is gone from every
> surface except the marker text a weave writes where nothing has ever been.

---

Last LLM verification:
- Date: 2026-08-26
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `PipelineResult::outputs_missing_a_recording`
  (`crates/hick-literate/src/lib.rs`) intersects `never_run` with the
  per-output `provenance_maps`, matching a span whose origin is
  `SourceOrigin::Exec { container, tag_line }` against
  `CellId::exec(container, tag_line)` (and `SourceOrigin::Script` against
  `CellId::containerless`). `write_outputs_detailed`
  (`crates/hickory-cli/src/lib.rs`) skips such a path when it already exists
  and is not `run.doc.weave_path`, returning it in `WrittenOutputs::preserved`;
  `write_outputs` keeps its old signature and returns only the written half.
  `print_run_summary` (`crates/hickory-cli/src/main.rs`) prints a `kept …`
  line per preserved file, for `hick run` and `hick weave` alike.
- Test coverage: `crates/hickory-cli/tests/weave_keeps_artifacts.rs` —
  `a_weave_without_a_recording_keeps_the_artifact_and_still_writes_the_report`
  and `a_weave_still_creates_an_artifact_that_is_not_there_yet`.
- Caveat requiring LLM review: coverage is **provenance-shaped**. A file with
  no provenance map, or one produced as `FileContent::Binary` outside the
  provenance path, cannot be matched to the cell that fed it and would still
  be overwritten. Agent cells are also outside it by construction:
  `SourceOrigin::Agent` carries a session and turn rather than a source line,
  so an agent cell's bytes cannot be matched back to its `CellId` — agent
  cells write into documents rather than into `hick:file` products, so this
  does not affect the artifact case, but it is an omission by decision rather
  than by oversight.
