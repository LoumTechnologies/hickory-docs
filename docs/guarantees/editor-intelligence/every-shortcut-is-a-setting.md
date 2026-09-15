# Every Shortcut Is A Setting

Given the app's keyboard shortcuts — the menu bar's (new, open, save,
terminal, files, settings, zoom, insert), the editor's (find, format,
rename, code action, add next occurrence, select all occurrences), the
workspace's (find in folder, problems, debug continue) and the tree's
dired and column-selection keys — when a person opens Settings → Keyboard, then every one of
them is listed with the key it is on, a **profile** binds them all the way
one IDE does (Hickory's own keys, which are VS Code's; VS Code; JetBrains;
Visual Studio), and any single action can be changed by pressing its key
and typing a new one (a chord of two strokes included), unbound, or reset
to the profile's. The choice and the overrides persist beside the window
title, so the desktop app and `hick up` agree; the editor's and the
tree's bindings take effect at once for editors opened after the change,
and the menu bar's at the next launch, which the page says.

One catalogue, in `lib/keymap.ts`, is the only place an action's keys are
written down. A binding is spelled the way people spell them
(`Ctrl+Shift+L`, `Mod+K Mod+D`), and three spellings come out of it: the
display string, CodeMirror's, and the shell's accelerator.

Three properties hold it up:

1. **Consumers ask the keymap, never a literal.** `isAction(event, id)`
   and `beginsChord(event)` for window handlers (find in folder, problems,
   debug continue, dired), `cmKeyOf(id)` for CodeMirror keymaps (format,
   rename, code action, the two occurrence commands, placed above the
   search keymap's own). A chord's first stroke is swallowed by the
   workspace so it reaches no editor as a stray Ctrl+K.
2. **The menu bar reads the same answer.** `saveKeymap` writes both the
   settings and `native_accelerators` (menu id → `CmdOrCtrl+…`, null for
   an unbound or chorded item) to ui.json; the desktop shell's `app_menu`
   reads them at launch and keeps an item's built-in key when the file
   does not name it. A chord cannot drive a native menu item, and the
   Settings page records a single stroke for menu actions.
3. **Profiles are complete and clash-free by test.** Every action parses
   under every profile (or is unbound on purpose), and no two actions of
   one scope share keys within a profile; a person's own clash is shown on
   the page and the first wins.

## Boundary

The command bar, the tab and pane keys (`Mod+\`, `Mod+W`), dialog keys
(Escape, Enter, arrows) and the debugger's step keys (F10, F11) are not in
the catalogue: they are the widgets' own or the browser's, not an IDE
convention a person carries. An editor already open keeps the bindings it
was built with until it is reopened.

---

Last LLM verification:
- Date: 2026-09-05
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `apps/web/src/lib/keymap.ts` (`ACTIONS`, `PROFILES`, the
  spellings, `resolve`, `conflicts`, `nativeAccelerators`, the live store,
  chords); `apps/web/src/views/KeyboardSection.tsx`; consumers in
  `shell/dired.ts`, `shell/useTreeNavigation.ts`, `lsp/cmLspFeatures.ts`, `editor/multiCursor.ts`,
  `shell/ShellView.tsx`, `views/WorkspaceView.tsx`, `debug/DebugStrip.tsx`;
  `main.tsx` loads it before the first editor. Server:
  `crates/hickory-cli/src/serve/mod.rs` `UiStore::{keymap,
  native_accelerators}` and the PUT arms in `serve/api.rs`; shell:
  `apps/desktop/src-tauri/src/lib.rs` `Accelerators` and `item`.
- Test coverage: `apps/web/src/lib/keymap.test.ts`,
  `shell/dired.test.ts`, `debug/DebugStrip.test.tsx`;
  `crates/hickory-cli/tests/serve_ui_settings.rs`
  ("the_keymap_and_native_accelerators_round_trip").
