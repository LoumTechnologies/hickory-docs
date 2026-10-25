<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/debugging/debug-controls-preserve-editor-space.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `1ebf856c59331b3bf8d8592453b04dbe71fab3a0dd2282310d525dd4d7041c47`.

<book:entry path="docs/guarantees/debugging/debug-controls-preserve-editor-space.md">
# Debug controls preserve editor space

Given a developer reading or editing a script in a workspace pane, Debug is one
right-gutter action beside the right line numbers: at the first line of a plain script or the opening declaration of
a generated script. Generated scripts retain their filename annotation without
an additional inline Debug button.

Patch comparisons use this same right-gutter launcher on each runnable side,
including After this patch. Opening a comparison automatically connects its
completion, diagnostics, definition and reference tools to the complete captured
project. References present all results for selection. Debug preserves the displayed source and chooses the latest atomic project
containing its exact bytes. A floating launch-profile picker selects the target
when needed; the gutter ellipsis opens it explicitly.

Generated-file tabs launch through their owning document session and map lines
through provenance. Unsaved output bytes cannot launch an older saved program.
Navigation into captured files preserves the captured native identity; views,
navigation records and active sessions retain it until their references end.

When debugging pauses, Continue, Step over, Step into, Step out and Stop appear
in a vertical group beside the paused line in the right gutter. Adapter-supported
backward stepping is offered there too. Running and starting show Stop. Active
controls stay at the nearest visible edge when their line scrolls out of view;
launch icons scroll with their script declarations. No action inserts text rows
or occupies the pane header. Call stack, watches and exception breakpoints live under Debug
options. Keyboard stepping remains available. Idle debugging adds no toolbar.
Build output floats over the pane while building or after a failed build and
never inserts rows into the editor.

F5 starts the focused script or continues its active session. Step over, into,
out and Stop use the selected keymap (F10, F11, Shift-F11 and Shift-F5 by
default). Definition and References also respect that keymap. A callee opened
while stepping borrows execution controls and values from the owning session,
including while running. Callee breakpoints update that same active session,
with replies and conditions kept under the exact source identity. Removing one
source's breakpoints preserves the others, including sources contributing to
one generated file. Saved source sets are installed before execution on launch.
Line-targeted jump/run commands never send callee coordinates to the launch file.

For a declared literate project, tools default to the completed project when
compatible with the displayed source. There is no global project-version menu.
Literate views keep their revision comparison form in a header menu and launch
through the same gutter action, requiring saved current code.

---

Last LLM verification:
- Date: 2026-10-16
- Reviewer: Codex
- Result: partially verified
- Evidence: `debug/DebugRail.tsx`, `debug/DebugStrip.tsx`,
  `components/ProjectContextSelector.tsx`, `views/workspaceTabs.tsx` and
  `components/PlainFilePane.tsx`.
- Tests: `debug/DebugRail.test.tsx`, `debug/DebugStrip.test.tsx`,
  `components/PlainFilePane.test.tsx`, `components/ProjectContextSelector.test.tsx`.
- Checks: `just test-web` passed 2,144 tests across 228 files on 2026-10-17.
  The frontend tests cover real CodeMirror rail placement,
  click-through launch/step/stop handlers, moving the paused anchor, and viewport
  edge clamping. Source text and its line count remain unchanged by the controls.
- Additional witnesses: `ComparisonTools.test.tsx` checks direct patch launch,
  full-code navigation without recapturing the project, completion and reference
  selection. `GeneratedDebugTools.test.tsx` checks Unicode provenance and owner
  launch. `plainDebugHosts.test.tsx` checks callee control continuity.
- Live browser evidence (2026-10-17): `just test-unified-debug-ui` passed against
  a disposable engine and Git project. It set a breakpoint in the side-by-side
  patch, launched directly from its right gutter, stepped into `helpers.py`,
  verified one captured editor with the paused line and live controls, stepped
  within the callee and stopped from there. Idle, paused-patch and paused-callee
  screenshots were inspected. Both diff sides retain their change highlights;
  the active debugger adds no editor rows.
- Source-breakpoint continuation (2026-10-17): the same browser flow sets a
  breakpoint in `helpers.py`, continues to it, selects the caller frame, stops,
  removes the caller breakpoint and launches again with only the saved callee
  breakpoint. It stops at that breakpoint, then removes it and continues to
  completion. Native socket coverage also checks conditions and preservation
  of the caller's breakpoint after clearing another source's set.
  `just test-source-breakpoints` passed 58 DAP unit tests, three project mapping
  tests, nine bridge tests, the live socket test and 142 focused frontend tests.
  The full frontend run passed 2,145 tests before the final clearing regression
  was added; the focused continuation also checks source failures and prevents
  a breakpoint cleared while idle from reappearing on a later launch.
- Revision witness: `just test-literate-editor 'edit ordinary source'` passed
  editing and saving current code, inspecting deleted bytes through the gutter,
  selecting a historical read-only revision and returning to editable current
  code. The harness now selects the opened spatial tab explicitly.
- Browser-suite limit: the wider literate-editor suite still contains stale
  pre-spatial-layout selectors and fixtures; its full run is not passing.
- Limits: standalone embedded editors without a workspace rail retain their
  own debugger toolbar. This frontend change does not add language support or
  establish live debugger installation on every platform.

Launch-profile continuation (2026-10-17): the former global version-menu and
replacement-editor paragraphs are superseded by exact-source selection and the
floating picker. Historical checks above remain evidence for their original
revision. Current witnesses and limits are in
`launch-profiles-choose-targets-within-the-viewed-project.md`.
</book:entry>

</book:doc>
