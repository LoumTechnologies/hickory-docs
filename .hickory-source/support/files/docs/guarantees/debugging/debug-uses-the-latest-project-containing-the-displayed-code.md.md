<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `2d8afbff2878e9fd0734d84302c973e046e775652e914dd2f9561c3bd11c99d7`.

<book:entry path="docs/guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code.md">
# Debug uses the latest project containing the displayed code

Given a file construction, generated-code pane, or project comparison displaying
code from an evaluated project, when its Debug action is pressed, then the
selected project is the newest evaluated whole tree containing that file's exact
bytes as a regular file. Later changes in other files are included if that
version still contains the displayed file. A newer replacement of the file is
never substituted for the code being read. A unique provenance-preserving rename
can select the renamed file. Equal text in an unrelated file is not a rename.

Breakpoints address lines in the displayed code and launch uses that same
selection. The global project-version selector is absent. Launch does not open
a replacement code editor. If no captured version matches, tools explain why
instead of launching different code. Native document launches check the editor's
source against the source read for evaluation and refuse a changed reading.

Candidate versions are the captured input and atomic prefixes produced by the
project evaluator in dependency order. A prefix inside an explicit scope is not
a candidate until all of that scope has completed. Selection never mixes files from different
versions and never searches VCS history. An immutable revision with one captured
tree has that tree as its only candidate. Ordinary files continue running in their
own workspace. Choosing a compatible tree does not certify that it builds or that
a source line is executable; build errors and adapter breakpoint binding states
remain visible through the existing debugger.

---

Last LLM verification:

- Date: 2026-10-17
- Reviewer: Codex
- Result: verified for selection and mapping in the executed native/WASM tests
  and the real Python/macOS browser flows; platform limits are listed below.
- Originating requirement: the user requested exact displayed source breakpoints
  with the latest possible project, then authorized implementation in this session.
- Audit timing: this durable interpretation was recorded retrospectively after
  the initial implementation. The session authorized exact-source/latest-project
  behavior before the edits; the implementation chose whole-file compatibility
  rather than selecting independently for each breakpoint line.
- Interpretation and assumptions: compatibility covers the entire displayed file,
  not merely the breakpoint line. "Latest" means the evaluator's captured ordered
  states, with the final project preferred whenever compatible. It does not imply
  synthesizing a new combination of independent later edits or speculative builds.
  These choices avoid running code that was never an evaluated version.
- Evidence: `hick-lang/src/project/debug.rs::latest_debug_project` is shared by
  native endpoints and WASM; `evaluate.rs` retains cumulative dependency-ordered
  versions separately from dependency-local edit states. `hick-lsp/src/project.rs`
  and `document.rs` select native trees, fence editor source, and map clicked file
  origins even after a byte-identical revert. `project_documents.rs` and
  `debug_sessions.rs` launch the selected revision-local filename. Frontend paths:
  `documentDebugContext.ts`, `documentSession.tsx::startDebugFile`,
  `GeneratedDebugTools.tsx`, `ComparisonTools.tsx`, and `patch/Editor.tsx`.
- Witnesses and challenges: `hick-lang/tests/project_debug.rs` checks a later
  other-file change, replaced code, dependency order differing from identifier
  order, exact reverts, renamed code and an unmatched draft.
  `hick-lsp/tests/project.rs` checks clicked-source breakpoint mapping and a
  changed reading. Frontend tests cover launch context, captured bytes and no
  replacement editor; real-adapter browser cases exercise file, Before and After
  surfaces with an independent later helper change.
- Coverage gaps: adapter/platform behavior outside the executed Python/macOS flow
  is not established. A late edit to a separate contributing source is captured at
  native evaluation; the primary editor-source fence is not a transaction over all
  open contributing buffers. A compatible evaluated prefix may fail its native
  build, which remains a visible failure rather than a reason to change versions
  silently. No claim is made that every code line can bind an executable stop.
- Dependencies: the files cited above and regenerated `hick-lang` WASM; changes
  to evaluation order, capture lifetime, source mapping or launch invalidate this
  review. Executed checks below apply to this working change, not a committed SHA.
- Executed checks: `just test-web` passed 234 files / 2,164 tests;
  `just test-literate-comparison-editor` passed 36 tests including the additional
  changed-buffer race regression. `just lint-literate-projects`,
  `just check-codegen`, `just check-file-length` and `git diff --check` passed.
  `just test-exact-source-debug` passed 19 existing project cases, six new
  selection cases, seven native project/mapping cases and 56 frontend cases.
  The final `just test-exact-source-debug-ui` run passed all six real-adapter
  browser flows, including file-block, Before, After, and cross-file stepping. No C# adapter acceptance run was performed for this change.
- Failed witnesses and corrections: the first full web run found an existing
  native `title` on the comparison breakpoint button; it now uses `data-tip`.
  Rust all-target lint found an existing nested condition in source-breakpoint
  event handling; it was collapsed without changing its condition. Removing the
  selector exposed the file-length script's failure on deleted tracked files;
  it now skips absent paths. An initial browser run lost its first session while
  implementation edits triggered Vite hot reload; the settled rerun passed all
  six flows. These failures are retained here rather than counted as passes.
- Human judgment: user approved the requested behavior; no human acceptance of
  the finished implementation is recorded. Codex inspected screenshots of the
  file-block and Before launches; automated runtime assertions check values.

Evidence dependency SHA-256 values (working tree, 2026-10-17):

- `crates/hick-lang/src/project/debug.rs`: `1f8398daf7f13224745dfe048ce789ac264c566efda71103f9734fdb15f159c7`
- `crates/hick-lang/src/project/evaluate.rs`: `3c33b603bfba9e9b4c181101ef8bcb9843a791ae1e5aceefcfc2d7e64ff3e266`
- `crates/hick-lsp/src/project.rs`: `d93385b52409c805d8368c05bb9dfbcc2ef3efe3e94bb16900b5975bc2a81385`
- `crates/hick-lsp/src/document.rs`: `a615ae7c81bc9ca13f01d9f4c9f847f3275a940a3202b8a7f51b3b08eb38e0ea`
- `apps/web/src/components/ComparisonTools.tsx`: `2229ccfa151135a66e4b3b08a01c27edef583b299b42c395a757079f746e8e1c`
- `apps/web/src/debug/documentDebugContext.ts`: `992a823cf906ae3c4eec7587fa5fce49a874dfafc89f48934816406828bd7a96`
- `apps/web/src/views/documentSession.tsx`: `ddfa02fc8ea00ae497ca327aee3e535645913d9aedd487f550babc87b9a1f443`
- `apps/web/src/debug/GeneratedDebugTools.tsx`: `a54e068cb2f7c04275a44c7a98d80ecb118fb336723cc2b0d5d3a0ec59c31f90`
- `apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm`: `18f95e4d61bc5e5f3a4c7a7864d7fb63c7744ae4ff7454b23255901870d5e026`
- `apps/web/e2e/literate-editor.spec.ts`: `aedad0f430f2007742adcad8b82d42f2a6d23557b9f2fa31cf27bfb143dbec25`

Launch-profile amendment (2026-10-17): the earlier cumulative-state interpretation
was too permissive. A caller patch inside a scope could be selected with the old
library even though the scope declares their change atomic. The evaluator now
retains only atomic prefixes as debug candidates. Dependency-local editing states
remain separate. `atomic_scope_never_exposes_a_new_caller_with_the_old_library`
is the regression witness. Launch profiles choose the executable within that
selected tree; source preferences and visible comparison routing preserve the
viewed library's breakpoints. Earlier hashes and check counts above describe the
previous implementation and are historical, not verification of this amendment.
See `launch-profiles-choose-targets-within-the-viewed-project.md` for current evidence.
</book:entry>

</book:doc>
