<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/literate-projects/comparison-integration-evidence.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `7e7be2bd849059f436f6532b38c83ea7c30cedfb1272bcf47400129f0bab972d`.

<book:entry path="docs/guarantees/literate-projects/comparison-integration-evidence.md">
# Comparison and agent integration evidence

The originating requirement is `docs/specs/freeform/literate-changesets.md`.
This continues commit `52a2912`; the earlier implementation-evidence file records
historical checks and hashes, which are stale for files changed in this continuation.

The earlier observations below are retained as historical evidence. The
continuation section records subsequent checks and supersedes their pending
statuses where explicitly named. Human judgment is not recorded. This file
does not declare the entire plan complete.

## Decisions and assumptions

- Comparisons edit drafts; immutable revisions require an authored fork.
  Each edit invalidates source-equality evidence. The native engine serializes
  diffs and regenerates patches through WASM; the browser adds no patch parser.
- Agent capture uses actual directory bytes and the declared project inventory.
  Every changed source-output path needs attribution resolution when the host
  cannot establish an exclusive writer. This is conservative: confirmation is a
  user declaration, not proof of who wrote the bytes. Room changes are separately
  named. A failed/incomplete capture cannot be finalized.
- Incorporation uses the current maintained-source closure as its destination
  baseline, preserving pre-existing work. AI returns a complete candidate
  document map. Empty-tree-to-B equality is checked before review and again after
  review revisions. Publication rechecks destination documents and completed code.
  Code already equal to B is never applied a second time.
- Reviewed source batches are independently guarded writes. A partial failure
  must name what landed; they are not a filesystem transaction. Deleted source
  rooms are retired so delayed persistence cannot recreate their files.
- Native document names may be absolute, but portable construction names are
  relative. The pipeline rebases supplied identities without borrowing bytes;
  the agent loads the declared closure rather than only the primary Markdown.
- Explicit nested scopes inherit declared inputs, finish every contained act,
  and expose Old/New endpoints independently of advanced local states. They do
  not order conflicting siblings. No implicit scope is created per generated file.
- Cross-document native mappings carry source identity and convert UTF-8 byte
  spans to UTF-16 at the protocol boundary. Missing or ambiguous positions do
  not become an unrelated source line.
- Session logs are language-defined non-source metadata under `sessions/`, beside
  `.git/`, `.hg/`, `.hick-cache/` and `.hick-journal/`. This aligns the inventory
  with the existing private conversation directory, rather than silently adding
  agent logs to the constructed program. Other exclusions remain explicit.
- Recording fingerprints and transcript rendering move below the host layer
  and are reused by both portable and native callers. Input-length fields use
  little-endian u64, preserving the existing 64-bit fingerprints while removing
  a native/WASM pointer-width difference. Existing 32-bit fingerprints may stale.
  Unknown mounted/capability inputs require execution or resolution; no cache,
  working source file or process supplies bytes during pure generation.

## Executed witnesses so far

- `just test-literate-comparison-editor`: nine focused browser tests passed
  before the latest scope/recording changes. These tests are now pending rerun.
- `just test-literate-agent-projects`: three real HTTP flows passed, followed by
  ten existing agent-edit-session tests and seven collaboration tests. The flows
  cover dirty A, AI incorporation through the existing review queue, revised
  explanations, missing outputs, concurrent completed-code edits, and a real ACP
  child process with abandoned and completed native file writes. Later changes
  to native mappings and recordings are pending rerun.
- Focused scope and reverse-edit tests passed in `hick-lang` (17 tests), and
  cross-document/UTF-16 and endpoint tests passed in `hick-lsp` (4 tests). The
  following pipeline build found a static attribute-table error, which was fixed.
  The full focused recipe is pending rerun.

## Outstanding verification and work

Native selector and comparison-tool integration, full contributor/recording
composition, staged-hook isolation, multi-target routing/undo, CI parity and
regression checks are still being implemented or verified. No discoverable
adapter or UI label alone establishes a working workflow. Live paid model
quality and explanatory accuracy require human judgment; scripted providers
exercise the mechanics only.

## Continuation review and executed checks

LLM implementation review: the inspected registration, selector, comparison,
publication and candidate-index paths support the tested integration subset.
The remaining limitations below still apply. Human judgment is not recorded.

The originating requirements are the user's empty-tree reproduction rule,
in-memory verification requirement, atomic project tools, independent editable
sides, and verified incorporation in `docs/specs/freeform/literate-changesets.md`.
The source-backed working continuation is based on commit `52a2912`; it is not
yet a committed release. Changing any dependency below stales its evidence.

Corrections discovered during verification:

- A literal command wrapper in a kept cell was rejected by portable replay while
  the native walker reads its text. `recorded.rs::command` now accepts a literal
  `cmd` body and continues to reject unresolved nested operations. The pipeline
  witness checks wrapper/bare parity, stale commands, expectations and origins.
- Main-worktree plain-file publication tried to create a Markdown collaboration
  room for `shared.txt`, so only the linked worktree was written. The failed
  shared-write/undo witnesses exposed this. `merged_write.rs::publish` now routes
  Markdown through the live-room guard and plain files through guarded atomic
  saves. Twelve real-worktree tests passed after the correction.
- The staged-hook fixture included an ordinary note without declaring it as an
  input, output or exclusion. Complete inventory correctly refused it. The
  fixture now explicitly excludes that note; the implementation retains its
  refusal of unexplained files rather than adding a Markdown-extension filter.
- Rename readings used the New filename for Old tools. Comparison sources now
  retain revision-local paths; a UI witness checks tool availability on both
  distinct names. Immutable inputs remain immutable.
- Older toolbar fixtures lacked the new live-source/project-context fields.
  Their four failures were corrected in the fixture without weakening parsing.

Executed checks on this continuation:

- `just test-literate-comparison-editor`: 22 tests passed, plus TypeScript checking.
  New selector tests use the real WASM evaluator. Tool UI tests check no request
  on render, complete later-file state after an explicit action, empty Old, and
  revision-local paths across renames. The native tool transport is mocked in
  these UI tests; native memory capture is separately exercised over real HTTP.
- `just test-literate-projects-web-regressions`: all 216 files / 2,067 tests passed
  after the toolbar correction. The later rename-path change was checked by the
  focused recipe above; the broad result predates that small change.
- `just test-literate-projects`: 19 language, 5 LSP, 1 DAP, 5 pipeline and 6
  revision tests passed; the opt-in real Mercurial test is excluded from this
  count. Fresh-folder and frozen-index witnesses do not materialize during pure
  generation/comparison.
- `just test-literate-worktrees`: 12 real HTTP/worktree tests and 6 browser tests
  passed, including partial publication, explicit permission and guarded undo.
- The latest real HTTP agent-project run passed all four flows: dirty A,
  reviewed/reverified incorporation, concurrent B refusal, native ACP abandoned
  writes, and immutable in-memory native-context registration/read/removal.
  One preceding ACP run timed out during initialization; an earlier run and the
  subsequent rerun passed. Its cause is not established and it remains an
  observed intermittent transport/startup failure, not erased success evidence.
- `just test-literate-projects-csharp`: the real EFCore migration, persistence,
  and netcoredbg breakpoint/watch acceptance passed on macOS arm64.
- `just lint-literate-projects`, `just check-codegen`, `just check-file-length`
  and `git diff --check` passed. Generated WASM was rebuilt after the literal
  command-wrapper change.
- `just test-literate-projects-native-regressions`: 91 parser, 103 LSP, 58 DAP,
  and one refactor-baseline tests passed. `just verify-literate-project-fixtures`
  passed all four source entry/contributor lint and verification paths. The
  separate `just test-literate-projects-hg` passed the actual Mercurial provider.
- A final review found that partial-write undo omitted unreached targets despite
  the older guarantee. Tokens now cover every attempted target with its expected
  current bytes; already-restored targets are no-ops and newer edits still refuse.
  Permission changes are prepared before replacement so a permission failure
  cannot leave a reported failure with unrecorded changed bytes. Worktree tests
  and lint passed after this correction.

Selected dependency SHA-256 hashes:

```text
crates/hick-lang/src/project/recorded.rs dd5c665c584e992688dce11015214d36dec1c4d96a31f902ed6575f88081ab9d
crates/hickory-cli/src/serve/merged_write.rs 4486b726bd89785913ee1bac211bd2f91762731450c7ce2d4517a2b323bd619f
crates/hickory-cli/src/serve/native_projects.rs 6642911b48f5f1975fea8616710e96294b85e70b1be3272375424d65ff968406
crates/hickory-cli/tests/literate_agent_projects.rs a188a61c84e422ddd75aff7b85359b14c90aa6abd84a44fd92b2c0a3b1d76601
crates/hickory-cli/tests/project_revision.rs 140187542d40dfe474305507355b68ab5d8b891d5aee95fd7ded479fb190f7d8
apps/web/src/components/ComparisonTools.tsx bece771cd0505564b3a58140174e8c0fc1d379872e6f8e451c8728849157f330
apps/web/src/components/ProjectContextSelector.tsx 8aa354b4d5a1a58adccc0f1bad6118e8cd8ddbf7567125c7827c559246161e97
apps/web/src/lib/projectComparison.ts 1bd71977465b9912227d82dc7e8bdf2f55a60de1c410042033234aee98957904
apps/web/src/components/ComparisonTools.test.tsx 0145f3a88164e6deaccad390b54895c7fbc7d4d649076d441cdcb00d3d0864ec
apps/web/src/components/ProjectContextSelector.test.tsx 8d971d442ae962b69118b98ad14e701f2103ade6274d9bc2267a66065a25b237
```

Current gaps: changed binary/metadata transition responses remain named refusals;
recorded cells with unresolved mounts/capabilities require execution or resolution;
arbitrary runtime-dependent construction, live paid-model explanation quality,
full visual automation of every native toolbar action, and additional native
platforms are not established by these checks. Multi-file and multi-worktree
publication report guarded per-target outcomes, not filesystem transactions.

The selected hashes above precede the final undo correction. Its inspected
post-format dependency hash is:

```text
crates/hickory-cli/src/serve/merged_write.rs aa387f36df1cf4263dbc92b2bf86bc1b06913d94cbd3bd2c7ccd40e6e271936f
```
</book:entry>

</book:doc>
