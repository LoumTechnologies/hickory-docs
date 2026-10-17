# Stepping prefers the launching literate view

Given a debugging session launched from a literate view, when it pauses at a
line represented by that view, then navigation reveals the line in the view
instead of opening its ordinary source file. This also applies to a callee in
another represented file. The paused-line highlight follows the mapped frame.

When the view is unavailable, its mapping is stale, or the line has no exact
mapping, navigation falls back to the source file. A session launched from an
ordinary file retains ordinary file navigation.

Given a captured project launched from a patch, when stepping reaches surviving
bytes in that note, then the note reveals the innermost source block (including
a later patch in the same atomic scope). It uses the selected complete tree's
byte origins, never a search for matching text. A stale note refuses the mapping.
A rendered patch keeps its current layout, highlighting, and code panes when
execution pauses or a frame is revealed. Debug navigation never switches it
to raw patch source; the paused marker belongs in its corresponding code pane.

While paused, Show literate source and Show generated code address the same
frame. Each mounted editor marks its corresponding paused line in the gutter.
Controls remain available in the rendered code panes and when the reader
explicitly chooses raw source.
Returning to a note rebinds its editor to the captured session. Patch panes
with the same complete endpoint share a captured identity, so a breakpoint
in a later patch is installed when debugging the caller. Old and New trees
and genuinely different snapshots keep separate identities.

Verification:

- `just test-debug-navigation`: exact origins, stale/absent lines, context
  identity, navigation preference, paired location actions and debugger rails.
- `just test-unified-debug-ui`: real debugpy launched from a patch, stepping
  into a later welcome definition, generated-code navigation and return.
- `just test-web`: frontend type checking and regression suite.

Implementation review (2026-10-17, Codex): partially verified.

The captured project's complete endpoint supplies byte origins. Patch launches
retain the launching note's source and editor separately from the patch widget;
debug stops scroll to rendered blocks without changing their rendering state
or moving the outer caret into replaced source. Removing a widget by choice
does not remove the session or its controls. Returning
notes rebind by workspace path, provided their bytes still match the capture.
Ordinary file launches keep ordinary file navigation. Unmapped targets fall
back to captured code. Cross-document authored navigation is not claimed by
this added mapping; origins in another contributing note currently fall back.
Human judgment has not been recorded.

Executed evidence: `just test-unified-debug-ui` passed all three real-adapter
flows, including launching from the caller with only the later callee's
breakpoint set and checking both paused gutter arrows. `just
test-debug-navigation` passed 25 focused tests. `just check-web` passed after the rendering correction. The preceding
`just test-web` run passed all 2,160 tests across 234 files. `just check-file-length` and
`git diff --check` passed. The captured-context lifetime regression checks
that no release occurs between starting, running and paused states.
