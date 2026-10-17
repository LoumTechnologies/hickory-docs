# Stepping prefers the launching literate view

Given a debugging session launched from a literate view, when it pauses at a
line represented by that view, then navigation reveals the line in the view
instead of opening its ordinary source file. This also applies to a callee in
another represented file. The paused-line highlight follows the mapped frame.

When the view is unavailable, its mapping is stale, or the line has no exact
mapping, navigation falls back to the source file. A session launched from an
ordinary file retains ordinary file navigation.

Verification:

- `apps/web/src/debug/literateDebugNavigation.test.tsx` drives the debugger host
  through mapped and unmapped stops and checks which navigation consumes them.
- `apps/web/src/debug/preferredDebugLocation.test.ts` checks exact representation
  provenance, absent lines, unrelated sessions, and surface removal.
- `just test-web` runs type checking and the frontend regression suite.

Last LLM implementation review (2026-10-17, Codex): partially verified.

- Origin: the user's report that stepping opens an ordinary file even when the
  target line is already in the literate file. This change addresses personal
  literate arrangements, which use plain-file debug sessions; native woven
  documents already use the DAP document mapping.
- Implementation: `RepresentationPane` records the launching surface;
  `plainDebugHosts.Host` asks it to reveal a mapped frame before opening a file.
  `representedFrameOffset` checks bounds before using exact provenance.
- Assumptions: source paths use the workspace-relative spelling supplied by the
  debugger bridge. No text matching or filename-only matching is attempted.
- Limits: a closed/unmounted view cannot consume navigation. This does not yet
  reopen a closed literate view when stepping returns from an unrepresented
  callee. Real adapter and interactive browser validation were not performed in
  this change; the host tests inject debugger state.
- Challenges: absent files, out-of-range lines, unrelated owners, removed views,
  and stops lacking provenance must fall back. A mapped stop opening an ordinary
  file while the launching view remains available would falsify the guarantee.
- Executed checks: `just test-debug-navigation` passed all 7 tests;
  `just check-web` and `just check-file-length` passed. `just test-web` passed
  all 2,148 tests across 230 files. The first full run overlapped an edit to the
  new test and failed; the completed rerun above supersedes that result.
- Human judgment: not recorded.

Evidence dependencies (SHA-256; changes invalidate this implementation review):
- `apps/web/src/debug/preferredDebugLocation.ts`: `292683a51998ca5aaac9181382704da6b9909a388f4a64e3433f26e472b0cd7e`
- `apps/web/src/debug/plainDebugHosts.tsx`: `b68e80264563bed6bece7226e28eeff6e7680992954d2527f95e2d82b771a71e`
- `apps/web/src/views/RepresentationPane.tsx`: `87ff952ac72a860b98d71b2e985cddf5f71d0f21392036b9b4f5c4bc2f08d736`
- `apps/web/src/views/RepresentationTools.tsx`: `538409bfa9ccefe84eb74ad74c81ad78f852da41968f4aa6296632d8071568c9`
- `apps/web/src/debug/preferredDebugLocation.test.ts`: `ee120464679a08de5f91332bdcac58ca3ba08a9a2e5c60687769af08ebbd71b4`
- `apps/web/src/debug/literateDebugNavigation.test.tsx`: `35da6c5c4b3bab11c7f22f02f7ae75f9061c1cf0ebd3a26be9d03fa370a6f3b5`
