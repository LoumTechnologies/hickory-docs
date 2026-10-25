<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/audits/2026-10-08-coherent-working-surfaces.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `1b6cd2c215157697f5519478b0fe1699ba953110f04c8f86e6ed750e22722a69`.

<book:entry path="docs/guarantees/audits/2026-10-08-coherent-working-surfaces.md">
# Coherent working surfaces: implementation audit

Date: 2026-10-08. Requirement: the user authorized implementation of all seven
findings from the earlier product coherence evaluation. The adopted rules and
scope are `docs/specs/freeform/coherent-working-surfaces.md`.

## Decisions and challenges

Saved means acknowledged bytes at the write destination. This deliberately
supersedes the earlier explicit-Save checkpoint for document dirty state;
recovery drafts remain separate. Keeping the checkpoint was an alternative,
but it would leave automatic persistence and the dirty indicator describing
different events. Delayed acknowledgments, subsequent typing, refused writes,
and inactive tabs are counterexamples tested against the chosen rule.

The engine remains authoritative for file reads and revision checks. Document
acknowledgment reads the actual file independently of rendering. A successful
render alone would not establish persistence. A syntactically invalid document
can be saved; a successful run cannot establish the truth of its prose.

Historical views declare read-only behavior; history actions still follow the
publication floor. Generated-file edits keep their reverse-edit destination.
Literate and generated buffers survive tab switches in the current window and
participate in close/Save All. They do not gain crash-recovery drafts; prompts
state that limitation. Retained generated edits against a newer baseline are
held, and a remount cannot duplicate an in-flight reverse edit. Discard and
reload are explicit choices, not automatic conflict resolution.

Previews expose proposed bytes/effects and do not execute them. Automatic
keystroke saves remain automatic; requiring a preview for each keystroke would
change the authoring model. Occasional toolbar actions move to overflow; the
existing cell rail and native menu mechanisms remain in use.

Initial browser acceptance exposed overly broad test locators: multiple open
documents, a virtualized agent history, and unrelated audit disclosure elements.
Tests now select the affected document and the actual conversation editor and
reveal the opening turn before inspecting it. These failed witnesses are retained
here as challenges; native fold and editable-history assertions remain enabled.

A full-suite run passed 1968 tests and exposed one timing-dependent scaffolder
test: the initial template options were being reloaded for the selected language
between two assertions. The test now awaits the intended choice and description
rather than assuming the first field label makes every asynchronous field stable.
Its option and description assertions remain enabled.

## Verification statuses

- LLM implementation review: partially verified. The seven contracts have been
  traced through the affected frontend paths and tests. This is an implementation
  review, not formal proof or a claim that all hidden assumptions were found.
- Executed checks: `just test-web` passed 197 test files and 1969 tests
  (final run 05:47:26, 102.92 seconds); `just test-literate-editor` built the
  local engine and passed all 6 browser flows. The final local-notice adjustment
  passed all 8 `workspaceTabs.test.tsx` tests. `just check-web` and
  `just check-file-length` passed; the diff whitespace check passed excluding
  the user’s preexisting `apps/web/src/content/startup.md` changes.
- Human judgment: not recorded. Agent inspection of document and literate-view
  screenshots found the destination, local result, save state, and expandable
  preview visible. That observation does not establish human usability.

Coverage gaps: browser acceptance uses the real local engine in an isolated git
repository. It does not exercise native window dialogs on every OS, cross-machine
writers, crash recovery for synthetic views, every layout size, or semantic truth
of prose. Those gaps do not change existing platform capability boundaries.

## Evidence dependencies

The following implementation and test content was inspected or executed. Any
change to these hashes makes the corresponding evidence stale. The earlier
verification blocks in amended guarantees remain historical evidence.

| Workspace-relative file | SHA-256 |
| --- | --- |
| `apps/web/e2e/literate-editor.spec.ts` | `8cade3ebaba7e5c451312699075038b0e844a76732052f286e1401a5adb7c6cd` |
| `apps/web/src/components/ActionNotice.tsx` | `0e28ae805288d26dd448f84d0396c5e9b515454fc9ea991b770f1d2d77d2db4f` |
| `apps/web/src/components/CellPanel.test.tsx` | `2674110ce527dd6cf363fa4097c7304d08d32c18f2234946259704514e71ad59` |
| `apps/web/src/components/CellPanel.tsx` | `2f9f333fb61b56c988b18dd2d5f8f46f6dcc0142c7bef5e99d46cef34c08cdf6` |
| `apps/web/src/components/ChangePreview.tsx` | `c9eaa5f1cf9ea1f192b16cae2a58043ef1f644aa0fb6baa43ca2b1a721143a3e` |
| `apps/web/src/components/DiagramPanel.test.tsx` | `a903a137406053a6122661a2c6f7f29b906f4f88b065032de62fd8b00a9919fd` |
| `apps/web/src/components/DiagramPanel.tsx` | `58c2db7a77684491c8b1999cc9f34dbfd862567a583de405c80d3de9f8bdff8a` |
| `apps/web/src/components/DivergedBanner.test.tsx` | `b308300a058a45ff4d95c59c937e61578b4dbea6c575ab8dc9a9a3a10f0f6cbf` |
| `apps/web/src/components/DivergedBanner.tsx` | `e77f475f5c57cab1d7391b6333e6f5d54f407e8f2e507fd88e2eb30fc6f22217` |
| `apps/web/src/components/InsertMenu.test.tsx` | `1eb8ed0082530fddaf99dc5912f5760fa6c05f282e9bc7d9b780da4a74f99e91` |
| `apps/web/src/components/InsertMenu.tsx` | `4f5d5ba8f1745a97e92e639252df8894d44b34592990253ecd00b0bead97f406` |
| `apps/web/src/components/MoreActions.tsx` | `ae904e592359cc9ca9fc2f8b665d173548a4a84b4584777a5abc97ef33d9430a` |
| `apps/web/src/components/NewProjectDialog.test.tsx` | `1ece206c920f1ea7e746d7c3664bea569ff6ace57d429551fc81178ea228c045` |
| `apps/web/src/components/NewProjectDialog.tsx` | `60652e9e598f2f1e2ae0f21cb346be1daafbbc16542d9b80610e36668dc8acbb` |
| `apps/web/src/components/PlainFilePane.tsx` | `d6ca04ee7e680741316ac240cd97731acf886991dde315d1e1866c4214cff92d` |
| `apps/web/src/components/StatusChip.tsx` | `5da30da59b3dda4ae75c1b2228608790cd15857d5d5c1c322a32636f896557f5` |
| `apps/web/src/components/SurfaceInfo.tsx` | `fa390b0a2feb877ec62b16a48fb13130e8c21184f6545f5af68a77c78d95f3aa` |
| `apps/web/src/editor/CardRail.tsx` | `3ec6d066f32fe9d4d3712c238f3ee0ad7185fd902979e6f0d2e34851f1fdfea9` |
| `apps/web/src/editor/DocumentEditor.test.tsx` | `d7992ea4808acdf49a9af2210ab00ec529bd446d0375c1b8968720b5923cc206` |
| `apps/web/src/editor/DocumentEditor.tsx` | `c842f047fa73d044de50a8998cb0fefe08a31055a61b66bcdf9738ee621c46d0` |
| `apps/web/src/environments/EnvironmentPanel.test.tsx` | `2c23eecc1e094c9a74ceacff5c1b4d6f8cb30ccdf93af5f7f0b2821a2ce89da7` |
| `apps/web/src/environments/EnvironmentPanel.tsx` | `f54cdc509796694d21d020213db65e6cbd5d89bb9949f4ccefd4639356b902f9` |
| `apps/web/src/lib/executionEvidence.test.ts` | `b816c12c112ead58615af6bb998d464fce740a5bde51b1dac057f32525ccf064` |
| `apps/web/src/lib/executionEvidence.ts` | `2a7794496a7f4a6877dc6737723ff41dbc3739d9e1c0f2b0f077eb55c105d8b6` |
| `apps/web/src/lib/flushSaves.ts` | `f983e73079d2c2ab9f8cbdb1bb9639ee7192438b887292f918aa4beaf7c0ab14` |
| `apps/web/src/lib/outputSave.test.ts` | `e138ba4d8000c13bba01bcb03c93bcf5af52cf823f677e331e3799a287fb01ec` |
| `apps/web/src/lib/outputSave.ts` | `6b0418dd611b17baa7fbedb559cd34d6246b0b961919667a6edd6bfc2b227e07` |
| `apps/web/src/lib/plainFileSave.test.ts` | `a009395adf8e49123046621fd3bdc2561707e3f5951c476260aa1c68b2f571b8` |
| `apps/web/src/lib/plainFileSave.ts` | `064939d3ad97a397439c2af0805b6375ad535b21ee36fc12fa61c0215b8eafb8` |
| `apps/web/src/lib/railActions.test.ts` | `48f23b3d1bd54b6ba3751b5e735199d4378f9991ee185830bc1281a1b74ceda7` |
| `apps/web/src/lib/railActions.ts` | `98133a0213264cf6e21bcde9c0fea104ffe96f6ea7f15172cb02969cad5c49e8` |
| `apps/web/src/lib/viewBuffers.ts` | `c64b2e1726ae5b41967bd79a98cda329e70f39b932c732c728ef2183663b7632` |
| `apps/web/src/shell/FilesystemTreeEditor.tsx` | `8aa95231d1bf3542a7c9caff10b3feb247af49cc698cecaf6ffe29ea1feac999` |
| `apps/web/src/shell/views.test.tsx` | `eacdb96d9ffbbbdfd963b2aa4ca26e7ca1c86c2a39fbbf2d71776a3489b80000` |
| `apps/web/src/shell/views.tsx` | `91d0079ed77f5814dae3e7e6e2fab9b1df2c18434964f6dfd63a0e97119e82fe` |
| `apps/web/src/styles.css` | `c11a58f7985ad207bdf8930ec1c11154773e780750ad1cc0484046a032660a8f` |
| `apps/web/src/views/CommitReading.tsx` | `d067a1d72abb078dc0057ffccb0ff44ffcb206061751e8d2805b8e360d162c5a` |
| `apps/web/src/views/DocumentReading.tsx` | `5740eeb13bfec26621fdccc07a1003290b5cf07c767011e0ae8906a430962e1d` |
| `apps/web/src/views/HistoryLens.tsx` | `20a8c6ac1274375e27cec8116560f854912e149f9cb7ea29f3450a2755708361` |
| `apps/web/src/views/MergedView.tsx` | `4af07c38d5722225a81601195cf74c2b2f8be0f3a2fb611e2452cf7f7d407cc6` |
| `apps/web/src/views/RepresentationPane.test.tsx` | `1cc1c8373d87ab428f94d9d513e6d3d31b95f9eb761fe5e0acc75efc02ebccfb` |
| `apps/web/src/views/RepresentationPane.tsx` | `15d937a948db3460a7406c028e8542021e1b9cc21c1e9db72d6abb164a2ad49c` |
| `apps/web/src/views/SessionLens.tsx` | `e359ad3ac12d7977fa2c477d1f4bd7175f6df795417f5f34cc4503ef2d7d7b96` |
| `apps/web/src/views/WorkspaceView.tsx` | `9ccbea7a83ab963127f895481aa4377a208fdde80509c63ee3131710b1a1cecd` |
| `apps/web/src/views/documentSession.tsx` | `7f98d04099678df362fe8dce1feebbd9719106aa26c91c10e9f81f79c6d04ec5` |
| `apps/web/src/views/representationTabs.tsx` | `8553df40cee20a5c2cc7a003c5834831b8458f1bf3bcb87c0beed5761fbee9ab` |
| `apps/web/src/views/useDocumentPersistence.test.tsx` | `2c602e2956cd31e4dd639e2c07bf43227418db41f2eac05682d10f63b8b55f70` |
| `apps/web/src/views/useDocumentPersistence.ts` | `ffdb9debe0bf9c525b1ede7e694c1442350d60234c80f70848aeb5b1c4469c1c` |
| `apps/web/src/views/useUnsavedLifecycle.test.tsx` | `2575c1ad804a799e16b9f94dece2b3e4e36a5fee9364788df2758407525ea3b1` |
| `apps/web/src/views/useUnsavedLifecycle.ts` | `6814d357791134ca6b0431137c7bafb870ac49f79781764fb801f9985648a529` |
| `apps/web/src/views/workspaceTabs.test.tsx` | `0cb20d07212adc7dde364c28d669c0ef9820d3cd0c59b0c07cc4c05001a51a9a` |
| `apps/web/src/views/workspaceTabs.tsx` | `08f61d49ebe55f73a0253ad44b7de9b5a90ca8bd82abc3d93724956e14946b4c` |
| `docs/specs/freeform/coherent-working-surfaces.md` | `995461a987b939a54e48165ff9c8b0e54bd6dc9210841826f19e4482ae8e8347` |
| `docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md` | `5506df1923016c920945f2ee6b8ac1e020f509d7c8c441f88f0999647c84983f` |
| `docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md` | `b23eb6bc43a1e798f0195f922a9a3c7a7017ab3ca1fb67da892b1cf15b3f62a2` |
</book:entry>

</book:doc>
