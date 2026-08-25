# A Derived Diagram Keeps Your Layout

Given a `<hick:diagram renderer="graph">` whose body takes its topology
through a `<hick:paste>` (`{"topology": <hick:paste select="#…"/>, "layout":
{…}}`), when the reader arranges it in the graph editor, then commits rewrite
**only** the `layout` object and write the paste tag back byte-for-byte —
adding or removing nodes and edges is refused, and the panel says the topology
is derived — and when the fragment is regenerated, nodes whose ids survive
keep their recorded places, new ids appear beside the layout rather than over
it, and layout entries for departed ids are dropped on the next commit.

The reason: the topology belongs to the generator and the layout belongs to
the person, and the only way both survive a re-run is to never store them in
each other's half. An editable derived topology would be an edit the next
re-run silently destroys; a regenerated layout would be an arrangement the
machine destroys. Keying layout by semantic node id — not by position, not by
ordinal — is what makes "the same box" survive the fragment changing around
it. Auto-layout stays a button: a verb the person applies, never something a
re-run does to their arrangement.

---

Last LLM verification:
- Date: 2026-08-25
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence:
  - Derived parse/serialize: `apps/web/src/components/graph/scene.ts` —
    `parseSceneSource` lifts the paste tag out verbatim (`scene.paste`),
    `withResolvedTopology` folds the server-resolved body in, and
    `serializeScene` writes `{"topology": <paste>, "layout": …}` with the tag
    byte-for-byte; inline nodes/edges beside a paste are refused (mirrored in
    Rust: `crates/hick-literate/src/scene.rs::parse_scene`, "never both").
  - The lock: `GraphEditorPanel.tsx` — `onConnect`, `onNodesDelete`,
    `onEdgesDelete`, `addNode`, and `rename` all no-op when `scene.paste` is
    set; `nodesConnectable={false}`, `deleteKeyCode={null}`, and the banner
    says the topology is derived. `onNodeDragStop` commits layout only.
  - Survival across re-runs: layout is keyed by node id
    (`scene.ts::placeMissing` places only ids with no entry, to the right of
    the placed extent), and `serializeScene` emits layout entries only for
    ids the topology still has, which is where orphans die.
  - Resolution: the app reads the resolved body from the render block model
    (`crates/hick-literate/src/render.rs::resolve_diagram_children` →
    `DocumentEditor.tsx`), and the weave resolves through run state
    (`weave.rs::resolved_scene_body`), so the same document drives both.
- Caveats — what LLM review could NOT establish:
  - The block model resolves pastes document-locally; a fragment living in
    another document draws nothing in the panel (the weave still resolves it).
  - "The next re-run keeps the layout" is proven at the serialization level
    (layout never holds topology and vice versa), not by an end-to-end test
    that actually re-runs a generator cell and reopens the editor.
- Test coverage:
  `apps/web/src/components/graph/scene.test.ts` (paste kept byte-for-byte,
  layout-only rewrite, orphan dropping, unplaced ids beside the layout);
  `apps/web/src/components/graph/GraphEditorPanel.test.tsx` ("locks a derived
  scene's topology");
  `crates/hickory-cli/tests/diagrams.rs`
  (`a_derived_scene_takes_its_topology_from_a_fragment_and_keeps_its_own_layout`);
  `crates/hick-literate/src/scene.rs::tests`
  (`derived_topology_reads_like_inline`, `two_topologies_are_refused`).
