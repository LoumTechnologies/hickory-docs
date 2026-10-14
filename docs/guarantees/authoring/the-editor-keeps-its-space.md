# The editor keeps its space

Given the desktop workspace, when editing a document or plain file, then view
identity, editability and write destination use an information control in that
pane’s existing header. Document-wide Run/Test and plain-file Debug live in its
header’s action menu. Cell actions remain on the right-hand action rail;
collisions still push buttons downward. A document with no rail actions reserves
no empty action column; adding a cell restores its actions. Editor instances, source bytes and
selections are not replaced by moving their controls.

Given search or commands, when requested with the configured shortcut or header
search button, then a temporary palette appears. It occupies no layout space
when dismissed. Escape restores the previous focus and editor selection;
reopening immediately does not inherit a pending dismissal.

The palette offers Run document and Test document for the focused document or
its generated file, naming the owning document. It uses that session's existing
actions and omits them while work is pending. Results from a previous query are
not selectable while a new query resolves; keyboard selection scrolls into view
and identifies its result for assistive technology.

An unnamed cell's Run icon offers to save its document through the existing
Save action. It does not claim to run an unknown cell or silently ignore the
click. The starter note explains Save → Run → Test before introducing agents.

Given appearance preferences, Compact is the default. It reduces document and
workspace margins and line spacing without changing the font size. Comfortable
restores generous reading spacing. Files rows use ordinary editor leading;
large 36px touch targets are an independent opt-in. Right-hand line numbers
start on and can be turned off independently of the cell action rail. These
preferences are stored locally and restored after reload, falling back to the
defaults if storage is invalid or unavailable. The ruler remains visible and
keeps its existing wrap-measure behavior in both densities.

Given tab zoom, when the focused document or ordinary code file is zoomed,
then its text changes size while the pane header retains its height. UI zoom
and tab zoom compose. Density and tab zoom do not rewrite source bytes.

---

Last LLM implementation review:

- Date: 2026-10-13
- Reviewer: Codex
- Result: verified for the inspected desktop workspace paths and executed
  Chromium flows; human usability and other native platforms remain unverified.
- Originating requirement: user’s minimalist UI audit request and authorization
  to apply its recommendations, with explicit retention of the ruler and action
  rail collision handling. A later message suggests making right numbers a
  setting and discusses a minimap.
- Interpretation: compact desktop editing is the default; comfortable prose
  spacing and larger Files targets remain available. Right numbers remain on by
  default. The minimap is deferred while the user considers the separate choice;
  no code minimap is claimed. The shell’s existing thin scroll indicator remains.
- Scope and exclusions: header consolidation covers ordinary document/plain-file
  editing. Standalone/embedded surfaces without a pane header retain their own
  controls. Debug sessions, project context selection, failures, explicit Save
  and specialized review tools may retain relevant controls; no execution or
  authorization boundary changed. Native menu behavior is not newly validated.
- Evidence: `shell/PaneHeader.tsx`, `shell/SpatialShellView.tsx`,
  `components/SurfaceInfo.tsx`, `views/workspaceTabs.tsx`,
  `components/PlainFilePane.tsx`, `shell/CommandBar.tsx`,
  `lib/editorAppearance.ts`, `editor/density.css`, `editor/chrome.ts`,
  `styles.css`, and `views/SettingsView.tsx` under `apps/web/src`.
- Test coverage: `PaneHeader.test.tsx` checks the editor’s DOM identity and
  selection while header content changes; `workspaceTabs.test.tsx` checks
  session-specific menu dispatch and disabled run state; `SettingsView.test.tsx`
  checks independent preferences; `CommandBar.test.tsx` checks Escape and rapid
  reopen. `apps/web/e2e/editor-density.spec.ts` checks actual CSS spacing, font
  sizes, headers, optional Files touch targets, unchanged source, right-number
  persistence, empty action-column reclamation and tab zoom against a real native
  engine. Existing spatial browser flows check real editor identity/selection
  through centering, drawer pinning, dragging and resizing.
- Challenges: rapid palette reopen originally inherited a delayed blur dismissal;
  overlay blur now closes immediately and its result buttons retain input focus.
  Browser setup initially selected the introduction because routing alone did not
  center the newly opened tab; the fixture now explicitly selects the tab as a
  person would. File creation uses the filesystem create operation before save.
  Initial browser checks exposed an unnamed-role summary; the information control
  now has an explicit accessible button role.
- Executed checks: `just test-editor-density` passed 9 files / 96 tests after
  the final implementation changes. `just test-editor-density-ui` passed all
  9 native-engine Chromium flows. `just build-web`, `just check-file-length`
  and `git diff --check` passed. Earlier `just test-web` passed 219 files / 2090
  tests before the final palette-reopen fix and empty-rail/margin adjustments;
  affected paths were subsequently covered by the focused checks above.
- Visual inspection: `/tmp/hickory-compact-editor.png` at 1200×900 shows the
  compact document, merged header, retained ruler, both number gutters and
  unoccupied source area; this is evidence from the current run, not a portable
  repository artifact.
- Human judgment: requested design preferences recorded; final layout has not
  been reviewed by a person. Browser geometry checks cannot establish taste or
  readability on every display/font/platform.
- Dependencies: SHA-256 hashes below identify the inspected content. Evidence
  becomes stale when these files or their relevant dependencies change.

Commit isolation (2026-10-13): the UI density/header implementation was
staged independently of pre-existing drawer, review-navigation and agent edits.
An isolated checkout of HEAD plus the staged patch passed `just
test-editor-density` (93 tests) and all five `editor-density.spec.ts` native
browser flows. The native browser runner reused the existing compiled fixture:
no Rust source changed in this task. The earlier 96-test / nine-flow results
above include the pending drawer work and are supplementary worktree evidence.
The Files fixture opens a drawer if available and otherwise uses the existing
visible Files pane, so these new checks do not require the pending drawer work.
The production implementation and settings were otherwise unchanged by this
separation. A snapshot mismatch, missing action, font-size change from density,
or loss of editor selection would falsify this guarantee.

Content witnesses for the staged implementation (2026-10-13):

```text
1865746167127c14bd618bdd56aca2d19b01b5cfba57c3974345d4e165590c24  apps/web/src/shell/PaneHeader.tsx
b82e22985369611a482266e9703576eb952be07cf78c4c1dd2dca4d6d786984d  apps/web/src/shell/paneHeader.css
fdba74d7f8dd9d6603d11af97b4b6d19b64a26c5e2c60406f06f7ee8e376088b  apps/web/src/shell/SpatialShellView.tsx
b06c6908d5e111aa65c4ab16ad5ae6f92018b380553b7db600be5f40a7d9b3f6  apps/web/src/components/SurfaceInfo.tsx
6d4b9455b921e31e84ba68aeaa7ceaeddee2a1be93aca0376be6d0c639cdc469  apps/web/src/views/workspaceTabs.tsx
fd7e5079fe28c4cd7be681bd64eb3bdb28e4e918a470049a2a89e65aa26b8729  apps/web/src/components/PlainFilePane.tsx
1c60880aaecd1300036e0db0496dbfc3745eca6263623c148d41f82e6a13ce22  apps/web/src/shell/CommandBar.tsx
7edb7372322bee777072972a1f2c82275defbf834fa90b26630e9feb7493faeb  apps/web/src/shell/commandPalette.css
8c88b1653483b91919f9110084587294a777134cdb8217aa35f53d2d31f8086e  apps/web/src/lib/keymap.ts
024be7f8ecfa68d05515271500e8879334675bbb0e826db291ae1425f09fd6e9  apps/web/src/lib/editorAppearance.ts
35224dc500c9cedf70fa7116772627d4aece23a1ea32d6931a1fbcf22c2bdb1d  apps/web/src/editor/density.css
68fc994a979fce753a64c725c4a43d93c1cc3884e21e28ea8340386856641e5d  apps/web/src/editor/chrome.ts
85e9a9dcdfddae25eaa97f825bc384fd6f5c76ac55b5f4d170afc3c1c4bf9e96  apps/web/src/editor/CardRail.tsx
61cdbb72c1d6de2f4ad8495028ee4d5d236cca0d5f1af5e2fbea219bf8b30f31  apps/web/src/lib/cardRail.ts
74529f28987edd87caae20fddc1f898f22a138d5f808727a375d9e094031500e  apps/web/src/styles.css
4e6788d7931870e66a58129e2620b7efdf18285d2fca303ebea6254bb9600499  apps/web/src/views/SettingsView.tsx
de29df0bb5168ce75d23a88657c3149eca937c8d602caf96089e3d0bfe78eef6  apps/web/e2e/editor-density.spec.ts
06a17ed21303c06ab9114b37c339b433195a1be5a89dacd8cc9e4a62ecc3dc96  apps/web/src/shell/CommandBar.test.tsx
4a9185b0547ccf0c7b78a1ca5fe6f07040d8de224035b629aeb46470cbd5abc2  apps/web/src/shell/PaneHeader.test.tsx
dbd8fddfc27d44e918b723de66b2ca0fdd7a68c2d4b4eb65b510eca76d638144  apps/web/src/views/workspaceTabs.test.tsx
43dba4833dc282b448ea0c03ee4aad149d88d4d49e4102a1ccf6781485a5cb2d  apps/web/src/views/SettingsView.test.tsx
```
