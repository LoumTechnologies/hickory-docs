# Captured reading surfaces share owned native actions

Given the lineage browser, launch-profile picker, editor ruler, recorded session
cards, function-profile evidence or constructed patch editor in an attached UI,
when the agent observes and invokes a registered action, it uses the corresponding
native controller and its current owner, revision, enabled state and disclosure.
Replacing the document, project, picker or captured run retires prior handles.
Reading a collapsed surface does not disclose its folded body. The command result
separates a requested view/selection/pin from completed replay, process startup,
provider delivery, verification or persistence.

Specific safety properties:

- Replay labels unavailable selected revisions and retains working-tree recovery.
  Historical replay excludes today's structural navigation. Project replacement
  cannot adopt a previous project's late response.
- Lineage fold gestures belong to one column and retire with their captured model,
  file or ranges. A fully folded file has one recovery hole. Column reading reports
  displayed source lines; revealing folded bytes requires a real reveal action.
- Profile choices bind their captured picker; command-only profiles refuse Debug.
  A picker selection does not attest successful execution.
- Ruler measure changes bind the document/view, stay within supported bounds and
  do not write source. Pointer cancellation discards its temporary position.
- Session work uses explicit disclosure. Source chips open captured historical
  passages when supplied, and relationship reviews require exact historical bytes.
  A retired session response cannot replace another session's cards.
- Profile rows are disclosed before aggregate reading/navigation. Context pinning
  selects existing evidence; it does not claim delivery. Editor selection changes
  only if the event's captured source bytes match the current buffer. A replacement
  run resets this surface's pin/disclosure state.
- Patch file/side choices live inside Patch options. A changed project capture
  refuses an old draft save. Retired save replies cannot update the replacement
  local editor. Agent comparison input markers survive regeneration into editable
  document transactions; this alone is not durable authorship certification.

## Verification record — 2026-10-20

Origin: issue #72's shared handlers, progressive reading/interaction and explicit
outcomes. This is a retrospective contract/evidence checkpoint for implementation
commits `dc68bb06`, `bbc691f6`, `d5cf3ebc`, `dae329f5`, `7a81d545`,
`3ec82e02` and `a84f43d0`; their behavior was previously recorded in the developer
parity log without corresponding strict-guarantee amendments. Earlier guarantee
reviews retain their historical scope and do not certify these changes.

LLM implementation review: partially verified. Inspected the named controllers,
shared dispatcher hooks and focused tests. The late cross-document patch-save
adoption guard was inspected; the new patch lifecycle test exercises late
inspection rejection, not an in-flight cross-process project write.

Executed checks (focused only): eight TimeSlider cases; three LineageView cases;
twelve LineageColumns cases; seven launchProfiles cases; ten existing ruler cases
plus two new ruler ownership/cancellation cases; five session chip/disclosure cases
and four SessionLens cases; two ProfileEvidence cases; four existing patch editor
cases plus two new patch disclosure/ownership cases. Targeted TypeScript checks,
file-length checks and whitespace checks passed. Source/body replacements, stale
handles, unrelated-buffer navigation, wholly folded recovery and failed-read retry
provide counterexamples to simplistic successful-click coverage.

Assumptions and gaps: registry scope/actor are host supplied; source identities and
captured profile records use their existing validation boundaries. Test geometry,
API replies and launch/navigation handlers are fixtures where noted in
`docs/developers/ui-action-parity.md`; actual native/provider/released/deployed
witnesses are absent. Static standalone pages still lack an attached host broker.
SVG docking geometry, host filesystem races, all platform-native callbacks and
unrelated controls are not certified. No new OS limitation is asserted. Full #72
parity remains incomplete. Changed dependency bytes make this evidence stale.

Human judgment: not recorded. No deployment, recording publication, provider run
or acceptance declaration was performed for this checkpoint.

Evidence dependencies (SHA-256):
- `apps/web/src/lineage/TimeSlider.tsx`: `1c77c1aa92b0fe18d62e06e73ea0e3508337855ad754bbfc098d4e5fc5a7a7d8`
- `apps/web/src/views/LineageView.tsx`: `7997b365cf497b9079ed3d01e3dbebb20d170c8c9b7b3daac3d0dbc305cfb2de`
- `apps/web/src/lineage/LineageColumns.tsx`: `5c98b1743ca79bc9c6cf4bebae284f15cdbb91f186affb41ceb5fa52eb51ebd6`
- `apps/web/src/lineage/StageColumn.tsx`: `69f0cb2a291df1e75a6210d4ec4a3907f8ddabeb0cc48162e6721bc7515236ae`
- `apps/web/src/lineage/HoleRow.tsx`: `6e8d14c09c1b208b522bb0d194e75ac37d78d397f4726a4b692b372e72f6dc3f`
- `apps/web/src/lineage/useLineageControls.ts`: `3259b8ff1f884efab90eb7ca34266faf0f765fb3aff4d47539f7373c1a9beb4a`
- `apps/web/src/debug/launchProfiles.tsx`: `33854f3f20470e84ab20e943bab73c4fe8f55a7ec868029ebbe3ba3a2302e3db`
- `apps/web/src/editor/EditorRuler.tsx`: `651d8185e79641eb598edba76f3c70869643444422defdcb6073cf3a08b1d46e`
- `apps/web/src/components/EvidenceDisclosure.tsx`: `0133589da1499dd1cdaa98e99a70fd64c1a352ad75d946d678e453ff22ba6e15`
- `apps/web/src/elements/session/view.tsx`: `c48f7f5be7b053ebd0597cac092d51b13b791c40cb3254461dd8edeb3d87e32c`
- `apps/web/src/views/SessionLens.tsx`: `4c19f5ebe0cfa6b41b2484580cdd33b1978e9002c606b5331d6b85fd7b845ec4`
- `apps/web/src/debug/ProfileEvidence.tsx`: `f60eb10520685206fa32323daf2c66f56b1f71dde87f2b31eca73383b5e2d3a1`
- `apps/web/src/elements/patch/Editor.tsx`: `39edf9e24f2918274164941f2beeb7c739649f8fbf640040b5a7b55a224ab48a`
- `apps/web/src/components/ComparisonEditor.tsx`: `d6e12c85bda04a9664a6575938dbc9e47860f9c1a2ab0ac8c49d5812f4d34ce4`

## Source launch rail continuation

Source and projection launch rails now invoke the same callback from model,
click and F5 controls. Exact source and callback/lifetime changes retire handles;
callbacks cannot overlap within their controller. A completed callback does not
establish a live debugger/process; actual failure remains visible. This extends
the contract to `DebugRail`/`useDebugLaunchActions` without certifying downstream
adapter startup. Five existing rail cases and three new focused registry/native
cases passed, with real CodeMirror and mocked launch callbacks. LLM review is
partial; human judgment and native/provider witnesses remain unrecorded.

Source dependencies (SHA-256):
- `apps/web/src/debug/DebugRail.tsx`: `224cbdfb8dde2e348728beb9295da89239ebd1a7bda781fb4c5b7130481b3ae6`
- `apps/web/src/debug/useDebugLaunchActions.ts`: `a4e2f3af054e2de713bdb64ec48816d5dc40f81dc22cebf0250e86217f57ae5d`
