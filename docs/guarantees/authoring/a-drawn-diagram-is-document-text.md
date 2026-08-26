# A Drawn Diagram Is Document Text

Given a `<hick:diagram renderer="graph">` whose body is a JSON scene, when the
reader drags a node, connects or re-plugs an edge, adds, renames, resizes, or
deletes a node, restyles a selection from the palette, or applies auto-layout
in the app's graph editor, then the finished gesture is
written back into the document's own bytes as the scene's canonical
serialization (stable key order, one node/edge/layout entry per line, integer
positions) as a user edit — so undo undoes it, collaborators receive it, and a
diff of a diagram edit is a readable diff. When the document is woven, the
scene is drawn as an SVG output file (content-named, referenced as an image
from the woven markdown) carrying the author's positions, sizes, shapes and
colours, a body that does not parse weaves as its raw JSON rather than
nothing, and a scene string containing the literal `</hick:` is refused at
commit with a message saying why.

Three rules govern the gestures themselves, because each had a reading that
looked reasonable and said the wrong thing:

1. **A drag from an OCCUPIED connector moves the line already there**; a drag
   from an empty one draws a new line. A slot holds one line, and a second
   line from a taken slot is what the flank handles (`before`/`after`) are
   for — so the drag with no other meaning gets the meaning people expect:
   pick a line up by its end and put it somewhere else. **The end that was
   picked up is the end that moves**, and the other keeps its node and its
   slot, so an arrow points the way it pointed. It applies only when exactly
   one line is attached: two lines sharing a slot is not something the editor
   produces, but a hand-written document can say it, and silently moving one
   of two is worse than drawing a new one.
2. **A drag dropped over nothing changes nothing.** The move is committed on
   connection, never on release, so a cancelled gesture needs no undo.
3. **A line being labelled gets out of its own way**: while the label editor
   is open the line stops at the box and picks up on the other side. The
   editor is opaque, so this is not about seeing through it — a line crossing
   the middle of a text box reads as a strikethrough, which says "deleted"
   about the words being typed. It is a mask on the path rather than a filled
   rectangle, because the canvas has its own pattern behind it and a patch of
   background colour would sit on that as a visible hole.

The reason: an editor with its own store is a second copy of the diagram, and
a second copy is where the lie starts — the reviewed file and the edited
picture drift, and provenance stops at the store's edge. Committing the
gesture into the text keeps the file the single source of truth, which is the
same contract the table grid already honours. The `</hick:` refusal is the
no-escaping invariant's one collision with JSON: written into the body, that
string would close the element from inside it.

---

Last LLM verification:
- Date: 2026-08-25
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence:
  - Format + canonical serialization: `apps/web/src/components/graph/scene.ts`
    (`parseSceneSource`, `serializeScene`, `uncommittableText`) mirrored in
    Rust by `crates/hick-literate/src/scene.rs` (`parse_scene`,
    `scene_warnings`, `to_mermaid`).
  - Commit path: `apps/web/src/components/graph/GraphEditorPanel.tsx`
    (`commitScene` — every gesture handler funnels through it) →
    `DocumentEditor.tsx::replaceBlockContent(slot, text, "input.diagram")`,
    a user event, so CRDT/undo/blame treat it as typing. The panel keeps
    `lastCommitted` and resets from the document on any external change.
  - Weave: `crates/hick-literate/src/weave.rs` `"diagram"` arm branches on
    `renderer == "graph"`, resolves the body synchronously
    (`resolved_scene_body`, pastes answered from run state), and emits an
    SVG output file (`scene.rs::to_svg` — shapes, palette colours, slot-
    anchored endpoints, whole-corner elbows) plus an image reference, or a
    ```json fence on parse failure. Validation warning
    beside the asserts warnings: `hickory-cli/src/lib.rs`
    (`diagram_assertion_warnings`, the graph branch).
  - Isolation and laziness: each panel wraps its own `ReactFlowProvider`; the
    panel is `React.lazy` so `@xyflow/react`/`@dagrejs/dagre` (both MIT) stay
    out of the marketing site bundle — verified by building
    `vite.site.config.ts` and grepping `dist-site/assets` for
    `xyflow|@dagrejs` (no hits).
- Caveats — what LLM review could NOT establish:
  - The canvas library is mocked in tests (it measures real DOM); what is
    proven is the panel's contract with the document, not that React Flow
    draws. No live drag on a real browser has been exercised by automation.
- Evidence for the three gesture rules: `onConnectStart`/`onConnectEnd` and
  the exported `moveEdgeEnd` in
  `apps/web/src/components/graph/GraphEditorPanel.tsx` (the grab is recorded
  in a ref and cleared on every drag end, so a drop over nothing commits
  nothing); the label gap is the `<mask>` in
  `apps/web/src/components/graph/SceneEdgeView.tsx`, referenced by `BaseEdge`
  through the `mask` prop it spreads onto the path.
- Test coverage: `crates/hickory-cli/tests/diagrams.rs`
  (`a_graph_scene_weaves_to_a_mermaid_fence_and_its_positions_stay_home`,
  `a_scene_that_does_not_parse_weaves_as_its_json_and_is_warned_about`,
  `a_well_formed_scene_with_a_broken_reference_is_warned_about`);
  `crates/hick-literate/src/scene.rs::tests` (downgrade determinism, dual
  topology refused, warnings, id sanitizing);
  `apps/web/src/components/graph/scene.test.ts` (canonical fixed point,
  orphan dropping, `</hick:` refusal);
  `apps/web/src/components/graph/GraphEditorPanel.test.tsx` (drag/connect
  commit canonical text, refusal shown and nothing written, external edit
  resets, two panels independent, and `moveEdgeEnd` — arrowhead moves with
  the tail anchored, tail moves with the arrowhead anchored, a move to another
  side of the same shape, other lines untouched, and nothing committed for a
  line that is gone);
  `apps/web/src/components/graph/SceneEdgeView.test.tsx` (the label editor's
  gap is a mask the path actually references, and no mask when nothing is
  being edited).
