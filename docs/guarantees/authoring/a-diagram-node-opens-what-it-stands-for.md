# A Diagram Node Opens What It Stands For

Given a `renderer="graph"` diagram whose scene has a node carrying an optional
`path` — a file or a directory, spelled relative to the document's own
directory — and optionally a `doc` — a `.hick` document about that node, spelled
the same way — when a reader **double-clicks that node** on the interactive
canvas, then the app opens what the node stands for: a document opens as a tab,
any other text file opens as a pane, and a **directory** reveals itself in the
folder tree, expanded down to it and scrolled into view. A modifier held during
the same gesture opens the node's `doc` instead. When the same document is
woven, each node with a `path` is wrapped in an `<a href="…">` inside the
`diagram-<hash>.svg` the weave draws, with the href relative to the woven
markdown's own directory, so the picture navigates on GitHub and any static
host too.

Both fields are **topology**, not layout: `hick diagram` emits `path` on every
node it produces — the directory for `--group dir`, the file for `--group
file`, both spelled **relative to the working directory the command runs in**,
which is where the document that pastes the topology lives — the canonical
serialisation writes them one node per line beside the id, the mermaid
downgrade ignores them, and the canvas editor never writes them. That is what
makes a **derived** scene navigable while staying topology read-only: the
scanner states where each box lives, and following it is reading, not editing.

The reason: a box in an architecture diagram is the densest pointer anybody
draws, and until now it pointed nowhere — the reader had to guess which folder
`app` meant and go find it. Putting the destination on the node rather than in
a separate index is what keeps it honest, because the same regeneration that
re-derives the shape re-derives the destinations, and a stale link cannot
outlive the box it belongs to. It sits beside the join key rather than in
`layout` for the same reason `label` and `shape` do: a generator can deduce it,
so a person must not have to maintain it.

---

Last LLM verification:
- Date: 2026-09-06
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence:
  - **Schema.** `crates/hick-literate/src/scene.rs` — `SceneNode.path` and
    `SceneNode.doc`, declared straight after `id` so serde writes them in that
    order; both `skip_serializing_if = "Option::is_none"`, so a node with
    neither carries neither. The TypeScript mirror
    (`apps/web/src/components/graph/scene.ts`) matches field for field, and
    `nodeLine` in the canonical serializer emits them in the same order, one
    node per line. `uncommittableText` checks them for `</hick:` like every
    other string a commit writes.
  - **Scanner.** `crates/hickory-cli/src/diagram.rs` —
    `generate_topology` reads the working directory through `prefix_from_cwd`
    and hands `generate_topology_named_from` the prefix; `relative_prefix`
    computes it (canonicalized, longest shared run dropped, `..` for what is
    left of the base) and `node_path` joins it onto the node's root-relative
    id. The rule is documented on `prefix_from_cwd` and in `hick diagram`'s
    own `--help` (`crates/hickory-cli/src/main.rs`, the `Diagram` variant's
    doc comment).
  - **The woven picture.** `to_svg` in `scene.rs` wraps each node's shape and
    text in `<a href="…">` when the node has a non-empty `path`, escaping the
    attribute through `esc_attr`. Nothing is rebased: the weave writes the
    `.md` and the `diagram-<hash>.svg` into the same directory (`weave.rs`
    `add_file_output` with a bare name), which is the document's own
    directory, so the string the scene holds already resolves.
  - **The app.** `apps/web/src/components/graph/openNode.ts` — `nodeTarget`
    (which of the two a gesture wants), `openTarget` (resolve through
    `lib/mdLinks.ts`'s `resolveTarget`, the same resolver a markdown link in
    the same document uses, then dispatch `hickory-open-path` with a leading
    `/`), `openNode`, `wantsDoc`.
    `GraphEditorPanel.tsx` passes `onNodeDoubleClick` to React Flow and takes
    a `docPath` prop, fed from `cx.path` by
    `apps/web/src/elements/diagram/view.tsx`. `SceneNodeView.tsx` draws the
    corner glyph and declines to open its rename input for a node that leads
    somewhere, so the two meanings of a double-click never fire at once;
    `NodeLink.tsx` is the inspector affordance (the path itself as a button,
    plus the `doc` button), shown for derived scenes too, and the node
    inspector grows a label input for exactly the nodes whose double-click was
    spent on navigating.
  - **Directories.** `apps/web/src/lib/openPath.ts` — `dirForAbsolutePath`,
    the directory sibling of `nodeForAbsolutePath` (which skips directories on
    purpose). `WorkspaceView.tsx`'s `onOpenPath` falls through to it and
    dispatches `REVEAL_PATH_EVENT` (`hickory-reveal-path`), handled in
    `FolderTreePane.tsx`'s `FolderRoot`: `ancestorsOf` expands every ancestor
    and the directory itself, the row is scrolled into view, and
    `folder-tree__dir--revealed` lights it.
- Test coverage:
  - `crates/hick-literate/src/scene.rs::tests` —
    `a_nodes_path_and_doc_survive_a_round_trip_and_lead_the_line` (parse,
    field order, absent-not-null),
    `the_mermaid_downgrade_ignores_where_a_node_lives`,
    `a_node_with_a_path_is_a_link_in_the_woven_svg` (one anchor per linked
    node, wrapping shape and label).
  - `crates/hickory-cli/src/diagram.rs::tests` —
    `every_scanned_node_says_where_it_lives` (dir and file granularity, and
    the in-place case with no prefix),
    `a_scanned_folder_is_named_from_the_working_directory` (the prefix rule,
    including a root outside the working directory).
  - `crates/hickory-cli/tests/diagram_cli.rs` —
    `emits_a_scene_topology_with_no_layout` (paths on stdout),
    `a_scanned_folder_is_named_from_the_working_directory` (end to end,
    `demo/app` from beside the folder),
    `mermaid_output_is_the_same_downgrade_the_weave_uses` (no paths in the
    fence), `refresh_rewrites_the_fragment_and_leaves_the_layout_alone` (the
    woven SVG's `<a href="app">`).
  - `apps/web/src/components/graph/GraphEditorPanel.test.tsx` — "a node that
    stands for something on disk": the double-click dispatch and its resolved
    detail, the modifier taking `doc`, a node with neither opening nothing,
    the derived scene staying navigable, and the inspector's Open button plus
    the moved label edit (with `path` surviving the commit).
  - `apps/web/src/lib/openPath.test.ts` — `dirForAbsolutePath`.
  - `apps/web/src/shell/FolderTreePane.test.tsx` — "revealing a directory":
    `ancestorsOf`, the expansion and the lit row, and a reveal of nothing.
  - Worked example: `examples/architecture-that-draws-itself.hick`, re-run, so
    its pinned `<h:expect>`, its `<h:copy>` fragment, its woven `.md` and its
    `diagram-34f46ab0.svg` all carry `demo/app` and `demo/lib`.
- Caveats — what LLM review could NOT establish:
  - The double-click was exercised through React Flow's `onNodeDoubleClick`
    prop with the canvas library mocked (the mock every test in that file
    uses, because React Flow measures nothing in jsdom). That the real
    library calls that prop on a real double-click is taken on its
    documented contract, not observed here.
  - The reveal's `scrollIntoView` is called but cannot be seen to scroll in
    jsdom; the test checks the expansion and the highlight, which are the
    parts that carry meaning.
  - Nothing validates that a node's `path` names a file that EXISTS. A scene
    can point at a folder that was renamed, and the failure shows only when
    someone follows it — the tree simply does not find it. Whether a
    diagram's paths should be checked at weave time, beside the `asserts`
    warnings, is an open question rather than a settled refusal.
