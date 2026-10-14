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
