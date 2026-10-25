<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `9271bb949c5afe1e1bf141288fc5c6dad13ffd4c04c92a36e319ca67bd93d70d`.

<book:entry path="docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md">
# Launch profiles choose targets within the viewed project

Given a Debug gesture beside displayed source, Hickory resolves the latest
compatible atomic project before discovering launch profiles. The viewed file
remains the breakpoint coordinate space; a selected application's entry point
may be another file. Definition-only library modules are not automatically
applications. Discovery reads project inputs without running them.

Built-in providers discover common application syntax, npm scripts, and
standalone Python test functions. Providers share one normalized profile model
with version-1 `.hick-launch.json` definitions. Manual IDs override discovered
IDs. Invalid configuration, duplicate manual IDs, multiple defaults, absent
programs and changed project fingerprints refuse launch with an explanation.
Captured projects use only configuration in their selected tree.

A single debuggable application or configured default can start directly.
Ambiguous choices and the gutter ellipsis open a floating picker without adding
editor rows. An explicitly chosen ID is remembered for that source during the
app session; an unavailable remembered ID asks for another choice. Commands and
discovered tests/scripts offer Run; source programs offer Debug when supported.
Argument boundaries and environment values are preserved. A failed explicit
build prevents launch. No semantic reachability claim is made by discovery.

Explicit breakpoint preferences follow the same project, path and exact file
bytes across compatible captures, including clearing a source's breakpoints.
Adapter binding status remains local to its live session. A stop in a visible
matching comparison reveals that pane, including a library originating in a
separate contributing document.

---

Last LLM verification:

- Date: 2026-10-17
- Reviewer: Codex
- Result: partially verified; final executed checks recorded below.
- Originating requirement: user approved auto-discovered and manually configured
  launch profiles after library debug actions merely executed definitions. The
  separately approved function-invocation/REPL design remains unimplemented.
- Interpretation: target selection and project selection are independent.
  Compatibility means whole displayed-file bytes at atomic project endpoints.
  Conservative syntax discovery is shippable without claiming LSP call-graph
  completeness. The provider trait is the extension seam; dynamic plugin loading
  and arbitrary command debugging are outside this implementation.
- Evidence: `launch_profiles.rs` providers/validation, `debug_profiles.rs`
  capture/fingerprint/launch, `serve/launch_profiles.rs` terminal runs,
  `debug/launchProfiles.tsx` picker, `breakpointPreferences.ts`,
  `plainDebugHosts.tsx` and `projectDebugSurface.tsx` source routing.
- Witnesses: native `tests/launch_profiles.rs`, `hick-lang/tests/project_debug.rs`,
  frontend `launchProfiles.test.tsx`, `plainDebugHosts.test.tsx`,
  `projectDebugSurface.test.tsx`, and real-adapter `literate-editor.spec.ts`
  launch-profile cases cover both versions of caller and library, literal args,
  environment, build output, and unchanged editor geometry.
- Challenges: the initial browser run hit the old library breakpoint but opened
  a generated tab because its origin was in another document. Visible captured
  comparison routing now checks exact source before falling back to origin.
- Gaps: Windows terminal execution and non-Python adapters have not been exercised
  by this change. Syntax discovery can miss entry points or offer unreachable
  targets; a successful launch does not promise every breakpoint will execute.
  Build recipes may fail. Automatic compiled builders do not receive profile env;
  explicit builds do. Configuration editing is file-based. Preferences are
  app-session memory. Builds and programs execute user code, without a new sandbox.
- Human judgment: approach approved; finished behavior not yet accepted by user.
- Dependencies: working-tree evidence hashes and observed checks appended after
  validation. Changes to those inputs invalidate that evidence.

Executed continuation (working tree, 2026-10-17):

- Full frontend: `just test-web` passed 235 files / 2,174 tests.
- Live browser: the command in `just test-launch-profiles-ui` passed all 11
  real-engine/Python-adapter flows. The first retry exited during disposable-engine
  startup before Playwright; the subsequent identical script passed.
- Additional frontend typecheck and 24 plain-file/workspace-tab tests passed after
  routing retry/install actions through fresh profile resolution.
- Failed test fixtures: a newly added test allocated a session on every render,
  causing React update recursion; stabilizing its mock fixed the test. The full
  frontend initially exposed a plain-file test missing the new discovery response;
  the fixture now supplies and checks the chosen profile on the debugger wire.
- Visual review: Codex inspected old-caller and manual-Run screenshots. Human
  acceptance remains unrecorded.

Evidence dependency SHA-256 values:

- `crates/hick-lang/src/project/evaluate.rs`: `fe1405af85265ce60e65bc3300e52620af6635879d040d2b3c56b11701bd01a4`
- `crates/hickory-cli/src/launch_profiles.rs`: `bb2cadf4708f349b9cf4e0f93e9da44e42089a0e0e2613e4a49781485c3b2616`
- `crates/hickory-cli/src/debug_profiles.rs`: `841c7ac2f94a3958f26a3517997016e7a3ce62810de3f12c02ef992995f591ac`
- `crates/hickory-cli/src/serve/launch_profiles.rs`: `0a8e72704163a4c4395c1dee247b401932c5866274835cb14e779f40a0c295e7`
- `apps/web/src/debug/launchProfiles.tsx`: `105feb523ef021657cbe72940f277d5fafc0ff0c35192eccc9b45c0dd6bca31e`
- `apps/web/src/debug/plainDebugHosts.tsx`: `fbd0ce1f92f8ad4cc0d4afeb24ea6dd865c7fa5bcc7f2957fb00609213ce6412`
- `apps/web/src/debug/projectDebugSurface.tsx`: `6ac8a3ebf1a69320f79b715d4c45ffc95d6d60c485227544841ef4d4b5134350`
- `apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm`: `c1e46cdc36323307dce58eaf91f9eb9b268f77464ebbf6bc030662010dda5162`
- `apps/web/e2e/literate-editor.spec.ts`: `ae1b28f8720d8e13f6f71c3cbe2eba2c02b53ba6851aa836654cc7a5a4f677b8`

Validation checks: `just lint-literate-projects` (all selected crates and targets,
warnings denied), `just check-codegen`, `just check-file-length`, and
`git diff --check` passed. The earlier source-breakpoint import failure after
extracting debug wire types was corrected with an explicit `Breakpoint` import.

Native witnesses passed: 19 project cases, seven exact-source/atomic-selection
cases, six launch-profile cases (including independent provider/manual override),
and four existing test-gutter command cases.

## Shared UI continuation — 2026-10-20

The additive shared-controller and captured-owner behavior, focused check results,
source fingerprints and remaining gaps are recorded in
[the scoped reading-surface contract](../agent/scoped-reading-surfaces-share-native-actions.md).
Earlier verification above is historical; it does not establish current universal
UI parity or native/provider/release acceptance.
</book:entry>

</book:doc>
