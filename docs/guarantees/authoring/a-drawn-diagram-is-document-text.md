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
  resets, two panels independent).
