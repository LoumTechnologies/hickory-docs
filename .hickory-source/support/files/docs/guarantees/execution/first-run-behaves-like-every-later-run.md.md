<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/execution/first-run-behaves-like-every-later-run.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `a8ab352aa59fc93d2a35005503114a08016c68539bb34618193f0cd46f0ca330`.

<book:entry path="docs/guarantees/execution/first-run-behaves-like-every-later-run.md">
# A Document's First Run Behaves Like Every Later Run

Given a document that assembles a file with `hick:file` and then executes that
file from a volume, when it is run for the FIRST time — with nothing previously
committed — then the cell finds the file and the run succeeds, exactly as it
would on any later run.

`hick:file` content is a *result* of the pipeline, produced during the content
phase. Input volumes, however, are seeded from the working directory before
execution begins. So a cell running a file its own document assembles — the
central move of literate programming, and the thing the grand tour is built to
demonstrate — could only ever work on the second run, once an earlier run had
committed that file. On a fresh document the cell died with
`can't open file '…/project/analysis.py': No such file or directory`.

Runs now weave the document and write its files into the checkout before
execution, so volumes are seeded from a directory that contains them. Weaving
performs no execution, so this costs milliseconds and cannot itself fail a run:
a document that will not weave fails the real run a moment later with a better
message. Files whose content depends on exec output are woven again from the
real transcripts afterwards — staging only guarantees that they EXIST when a
volume is seeded.

---

Last LLM verification:
- Date: 2026-08-06
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `apps/server/src/runs.rs` — before the executor starts, a
  `spawn_blocking` weave (`RunMode::Weave`) writes the document's output files
  into the per-run checkout; failures are logged at debug and never pre-empt
  the real run's error.
- Test coverage: end-to-end against the running server —
  `examples/grand-tour.md` created as a brand new document and run once goes
  from a failed run with zero output files to a successful run weaving
  `analysis.py`, `grand-tour.md`, and `regression-explorer.html`. The existing
  server integration tests cover the run/commit path around it.
</book:entry>

</book:doc>
