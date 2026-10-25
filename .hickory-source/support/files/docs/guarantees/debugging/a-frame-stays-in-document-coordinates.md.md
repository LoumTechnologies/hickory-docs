<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/debugging/a-frame-stays-in-document-coordinates.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `6ca5769d32cba25b17169d2b66da1118cc3b94156e750f29d3576611cc894f94`.

<book:entry path="docs/guarantees/debugging/a-frame-stays-in-document-coordinates.md">
# A frame stays in document coordinates

Given a woven file with document lineage, when a debug adapter reports a frame
through a filesystem alias or symlink, then the frame still resolves to the
owning document and source line. Canonical spellings such as macOS `/private/var`
and `/var` name the same file. Files without document lineage keep their own file
coordinates. This lookup does not change breakpoint placement or file bytes.

---

Verification (2026-10-13, Codex):

- Implementation: `crates/hick-dap/src/mapping.rs::file_map` first checks the
  adapter's spelling, then its canonical path, against the known file maps.
- Regression: `crates/hick-dap/tests/project.rs` weaves a real file and checks
  both reverse mapping operations through a symlink. The existing real debugpy
  tests in `live_session.rs` cover breakpoints, stepping, jumping and run-to-line.
- Observed failure before the change: four real Python debug-session tests
  stopped correctly but reported `line: None` and `in_document: false` because
  the frame used `/var` while the map used `/private/var`.
- Limits: a nonexistent alias cannot be canonicalized; it needs an exact map
  entry. No case-insensitive string matching or guessed document lineage is used.
</book:entry>

</book:doc>
