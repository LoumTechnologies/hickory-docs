# A Drift Report Names What Differs

Given `hick test` reports a committed output file as drifted, then the
message names where the mismatch starts — a byte offset, and a line number
when the content is text — and how the two lengths compare, not only the
fact that the two differ.

## Why

`check_failures`' `CheckFailure::Drift` always carried a `detail` field, but
until now it was one of two fixed sentences — "committed file differs from
freshly produced output" or "output file missing on disk" — regardless of
what actually differed. An author had no way to see what `hick test` thought
was wrong short of diffing the file by hand outside the tool.

This was found, and fixed, while root-causing a real report: `hick test`
returning `DRIFTED` against a tree that was byte-identical across repeated
`hick run`s and clean in `git status`. The generic message gave no traction
on that at all — `hick test --json` showed every cell's own `status` as
`"ok"`, so the mismatch had to be somewhere the per-cell status doesn't
cover. The actual cause turned out to be upstream of this comparison
entirely: cache-key instability from a build cell's own gitignored output
polluting a shared volume's digest
(`docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md`),
now fixed there. This diagnostic did not find that root cause by itself —
the investigation that found it was sequential elimination across the other
fixes in this pass — but it is real, standing value independent of that one
incident: the NEXT report of drift, whatever causes it, now says where.

## What it does not do

This is not a diff. It reports the first point of disagreement and the
length delta, not every differing region — the two failure shapes worth
telling apart are "one file is a prefix of the other" (nothing to point a
byte offset AT — there is no differing byte, only where one ends) and "the
content actually diverges somewhere in the middle" (a byte offset, and a
line number when both sides parse as UTF-8). A reader who needs the full
diff still reaches for `diff` themselves, now knowing where to point it.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Sonnet 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/lib.rs`'s `describe_drift`, called from
  `check_failures` in place of the two fixed sentences.
- Test coverage: `describe_drift_tests` in `crates/hickory-cli/src/lib.rs`
  (3 tests: a middle-of-content byte difference names both the byte offset
  and the line number; one file being an exact prefix of the other is
  reported as "identical for the first N bytes, then one ends" rather than
  a false byte-offset claim; non-UTF-8 content still reports a byte offset
  with no line number, since none can be computed).
- Caveat requiring LLM review: no test exercises this through the real
  `hick test` CLI path end to end (the existing `CheckFailure::Drift` tests
  only check that the FAILURE fires, e.g. `volatile_outputs.rs`) — the unit
  tests above cover `describe_drift` directly, which is the only function
  this change touches.
