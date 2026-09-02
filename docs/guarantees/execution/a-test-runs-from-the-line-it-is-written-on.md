# A Test Runs From The Line It Is Written On

Given a plain or generated file in a language whose tests have a shape the
editor knows — `#[test]` above a `fn`, `it("…")` or `test("…")`, `def test_…`,
`[Fact]`/`[Test]` above a method, `func TestX(t *testing.T)` — when the file
is open, then a run mark sits in the gutter on each test's line; and when the
mark is clicked, then that one test runs in its ecosystem's own runner
(`cargo test <name>`, `npx vitest run <file> -t <name>` or jest, `python3 -m
pytest <file>::<name>`, `dotnet test --filter`, `go test -run '^<name>$'`),
in the nearest directory that owns the file, as a terminal session named
`test: <name>` that opens beside the file and reports finished or failed the
way every session does. A file with no manifest above it, or a runner the
package does not name, is refused in words that say what is missing.

A terminal, not a summary, for the reason a build is watched rather than
reported: a failing test says why in its own words, and "1 failed" throws all
of that away. Nothing here is recorded, verified or woven — the transcript is
the record, the terminal is the run happening.

## Boundary

Detection is textual and per language; a test a macro generates has no line
to mark. A `hick:file` block inside a document is not marked — its file exists
only after a weave, and the generated pane, which shows that file, is.
Failure navigation from the terminal's output back to a line is not built.

---

Last LLM verification:
- Date: 2026-09-02
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `apps/web/src/editor/testGutter.ts` (`findTests`, `testGutter`;
  the mark answers its own click), mounted in
  `apps/web/src/components/PlainFilePane.tsx` and
  `apps/web/src/components/OutputEditorPane.tsx`;
  `crates/hickory-cli/src/serve/test_run.rs` (`test_command` walks to the
  nearest manifest and names the command; `run` opens the terminal session)
  routed at `POST /api/tests/run`; `apps/web/src/lib/revealLine.ts`
  `showTerminalRequest`, answered in `apps/web/src/views/WorkspaceView.tsx`
  with `openTerminalTab`.
- Test coverage: `apps/web/src/editor/testGutter.test.ts`;
  `crates/hickory-cli/src/serve/test_run.rs::tests`;
  `crates/hickory-cli/tests/test_run.rs` (a real `cargo test` through the
  endpoint, watched to completion).
- Caveats: the gutter's click is exercised in jsdom on the mark's own
  element; the terminal tab opening is by review.
