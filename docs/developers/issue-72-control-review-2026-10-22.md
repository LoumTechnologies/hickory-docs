# Issue 72: delegated-control review, October 22

Audience: engineers checking a declaration against its real controller.

This is a static delegation review, not universal runtime qualification. The
33-file cohort was selected from reachable JSX declarations without a direct
registration recognized by the index. Hook/child delegation is common; zero local
registrations is not evidence of a missing capability. The sweep found the live
useShellKeys gap and led to shared handlers and focused checks. Other newly found
imperative forms/menu/load-state gaps are recorded in the issue-72 handoff.

| Reviewed native declarations | Shared controller and boundary |
| --- | --- |
| ChangePreview; ChangesetDocument | EvidenceDisclosure/PreviewReading; DocumentEditor and captured-project patch controllers. Early changeset failure reading is installed before editor mount. |
| ChatTree; ConversationEditor | ChatDock conversationActions.tip/send; protected-prefix DocumentEditor, session folding and shared inline Send callback. Ordinary session bytes remain separate from draft/reading scope. |
| MarkdownTable; TableCell; TableFillHandle; TableGridControls; TableNumberFormat; elements/table/view | TablePanel/uiTableActions, fill and measured viewport controllers, shared widget engagement and captured block writer; source/readonly checks and UiActor continue through the parent. |
| NewProjectFields | NewProjectDialog field table, options, pending guards, preview/create controller and chooser request/result. Boolean/string option representation follows the actual template schema. |
| GraphInspectorLabel; SceneEdgeView; SceneNodeView; elements/diagram/view | useGraphLabelDraft and GraphEditorPanel graph controller; resize/move/labels pass through the current source-owned block writer. Existing shared graph tests include native gestures and model origin. |
| GeneratedDebugTools; RepresentationTools | DebugLaunchAction, DebugStrip and useDebugger; debugger providers install real inline eval/condition form actions. PromptPanel owns watch input; host setup and capabilities remain authoritative. |
| DocumentEmbed; embed/iframe | EditingSurface/DebuggableDocument and the explicit origin-bound host session. Missing parentOrigin/session is setup failure, not an implicit engine attachment. Unattached hosts must supply the UI session/handshake. |
| WorkspaceProblems | ProblemsPanel reading/selection/close controller; its real onPick retains document/file navigation. Diagnostics are not a current source edit baseline. |
| HoleRow; LineageColumns; StageColumn | useLineageControls for filters, queries, files, ranges, nodes and other-stage disclosure; fold and pointer selection share the existing handlers. Scrolling/measurement alone does not expand source context. |
| GithubTreeNodes | useGithubTreeDisclosure; GithubNotificationRead/GithubTreeLink; TreeRenameEditor and GithubBodyAndComment. Expanded bodies/readers retire with the actual disclosure owner. |
| PaneHeader; ShellView | PaneHeader only stops propagation to preserve header interaction; it is not a separate mutation. The legacy ShellView component has no current JSX caller, but its exported useShellKeys IS called by SpatialShellView and now registers actual split/search/close handlers. |
| TreeContextMenu; fileMenuActions; shell/views | ContextMenu, Dired/TreePrompt/TreeRenameEditor; DivergedBanner, MergeView, OutputEditorPane and recovery controllers. Root/header/tab menu launchers now have their own actual opening controllers. Early generated-pane failures are readable. |
| FleetPane; LandingView; MergedView; RepresentationPane | useFleetControls/pairing; UiLink/media/tour/publication controllers; useMergedViewControls and WorktreeRegionEditor; RepresentationActions/Versions/Tools, DocumentEditor and PromptPanel. Literate-view load failure is separately readable before those children exist. |

The index records potential import reachability, including types and runtime
exports. WorkspaceView aliases SpatialShellView as ShellView; the legacy visual
ShellView has no current JSX caller. Its keyboard export must not be excluded.
Explicit parent/source ownership, disclosure and actual callback outcomes remain
required even where a hook is present. This review does not certify every dynamic
instance, every platform, visual layout, authentic provider behavior or human
judgment. Native OS/credential/activation requests retain their existing typed
requires_input boundaries; this table grants no extra authority.

Focused keyboard evidence: useShellKeys.test.tsx plus ShellView.test.tsx, seven
cases; strict changed-entrypoint TypeScript passed. Native Control-Backslash and
Control-Shift-F share the actual split/search handlers; focused close returns the
normal unsaved-work result. Hidden workspaces cannot act. The first run caught a
test's incorrect layout field name and a return-type inference error; both were
fixed before the passing checks. No broad regression or post-commit monitoring.

Full acceptance remains open. Real funded/authenticated provider, selected
human-reviewed native footage and matching released/deployed evidence are not
supplied by this static audit or by scripted fixtures. See the active handoff.

Exact reviewed files (SHA-256; a changed dependency stales this review):
- `apps/web/src/components/ChangePreview.tsx`: `2400d202996f3f42aee011fed2584c01bcfb6c57f83c97d82c1f4c6dedfff44e`.
- `apps/web/src/components/ChangesetDocument.tsx`: `86d46e98d6616a0666f3a64db623fe848d855edab81197f4e14511682c1ee86a`.
- `apps/web/src/components/ChatTree.tsx`: `01e508ec060b5f68f2ca8d07406572baa6c4905f3014aa284d5183b7e71a0309`.
- `apps/web/src/components/MarkdownTable.tsx`: `05a39a4dbca31dd4a3d9693b501ebe55d1c2a91d87b234ad3e449892da538af9`.
- `apps/web/src/components/NewProjectFields.tsx`: `b365c2beb36eed44671727120d2f727f769d1282c45629cc3300c55e7468b77b`.
- `apps/web/src/components/TableCell.tsx`: `54ca2493be70316f4c893ab681ced1b6e628125c0aa7a0d8903fe8f4d5209f0b`.
- `apps/web/src/components/TableFillHandle.tsx`: `39d663e6efbc62252adac2431caaeee9f16ec6299ddc447716c8c78298f9432f`.
- `apps/web/src/components/TableGridControls.tsx`: `06e5387ea52a272248d998cfbf4dfb55f8dddf5644b4072eace0fb7fcd9e6480`.
- `apps/web/src/components/TableNumberFormat.tsx`: `5edcc81388b329072b6db181e5a2840624ac590c2932acc1b75132440a2f0c84`.
- `apps/web/src/components/graph/GraphInspectorLabel.tsx`: `429ae73b6d7876a42ec23b6dea0630734affac33fc8fce0ed61261f47588b48f`.
- `apps/web/src/components/graph/SceneEdgeView.tsx`: `c42cd9d312805054dee7b02d38a419afb380f56e0c42e656554eb266118d93bf`.
- `apps/web/src/components/graph/SceneNodeView.tsx`: `d3df7d5c3f9c0a8e737c75d5faefb875adf7d1c2960e36352fcfc1cec2c30cc3`.
- `apps/web/src/debug/GeneratedDebugTools.tsx`: `6997bd00602a3fea59f94a4883af8d404098e41ac9bec71a175458c815ff6835`.
- `apps/web/src/elements/diagram/view.tsx`: `4f3111d1c25070e20594ef9254ce05e78e0ce5c6d786981d0d581abee2be80ca`.
- `apps/web/src/elements/table/view.tsx`: `0e0c17bf0f75a379952f4f11e746f8c347815348141c9b8f3b1174f59c537bb7`.
- `apps/web/src/embed/DocumentEmbed.tsx`: `e17b32b88335240607b1002dbbce76087a58913c205f00881b6045e653d57c83`.
- `apps/web/src/embed/iframe.tsx`: `ad6e46f073d0ba50299fbc939940233367290084271ea4d59899790d02f7cc99`.
- `apps/web/src/environments/WorkspaceProblems.tsx`: `9fbadc45427cfe60522661953d75146f3262ca85ead8fa19be5b2cc39b4417ad`.
- `apps/web/src/lineage/HoleRow.tsx`: `6e8d14c09c1b208b522bb0d194e75ac37d78d397f4726a4b692b372e72f6dc3f`.
- `apps/web/src/lineage/LineageColumns.tsx`: `5c98b1743ca79bc9c6cf4bebae284f15cdbb91f186affb41ceb5fa52eb51ebd6`.
- `apps/web/src/lineage/StageColumn.tsx`: `69f0cb2a291df1e75a6210d4ec4a3907f8ddabeb0cc48162e6721bc7515236ae`.
- `apps/web/src/shell/GithubTreeNodes.tsx`: `5ffec3322208e76357e48eb236dceb87dba0cad5b6a7687cd0bf0397dd5b4514`.
- `apps/web/src/shell/PaneHeader.tsx`: `1865746167127c14bd618bdd56aca2d19b01b5cfba57c3974345d4e165590c24`.
- `apps/web/src/shell/ShellView.tsx`: `4edf31409bc9ab64f58f4cdab5c768472f0da8a2bbb4f2fba490043c2092e2d0`.
- `apps/web/src/shell/TreeContextMenu.tsx`: `7114b7db3a6014a4d320134f7f7639dfc61352f6c176e3cab541f8f3312d687c`.
- `apps/web/src/shell/fileMenuActions.tsx`: `6cfe0a9d23b6fae98d091d665eaa289f8651756ee66ee18ca854f672cec6613e`.
- `apps/web/src/shell/views.tsx`: `84f23763d9a3490365263b1a18badc1fd57193b5a1bc663b5591c4329e89b2f1`.
- `apps/web/src/views/ConversationEditor.tsx`: `1399bfa292bf6c788c1d97bfeb80798cc02417cfa9fc10f87262a89c0ff64d35`.
- `apps/web/src/views/FleetPane.tsx`: `0a9f5dbcb1a3e0e473378c3242c08e350b5827ee1e78d3123f8b044af02d0bed`.
- `apps/web/src/views/LandingView.tsx`: `e6c57cc192d4d820cc64e5f81cf77bcea482e3434df63ba076fbd3f3d6243b78`.
- `apps/web/src/views/MergedView.tsx`: `dad9dcf4f670c9642c006968d07bcfbfbfdf810d00d8c423ab85b23bae017b12`.
- `apps/web/src/views/RepresentationPane.tsx`: `3b874242d47a9c5d45dbd687637a21508026ab0e507d4ea41ae3879d32acda42`.
- `apps/web/src/views/RepresentationTools.tsx`: `f897b7a737b38ac91f3af05350b0201a68eb73d69a34f3680f535af23d506f1d`.

Declaration index SHA-256: `b054c5ea0186c00a700fa879380e4d9af9465f3f6e9fe8dac13c3e1c1530e9ba`.

The later native screenshot found a fixed-position child occluding coach chrome;
static delegation/focus evidence had not established visual operability. Its CSS
fix and actual-component pointer/geometry checks are in the active handoff.
