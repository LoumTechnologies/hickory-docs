# A diagram you can drag

## The problem

`diagrams-that-fail-when-they-lie.md` made a picture checkable, and its price
was mermaid: text in, layout out, and no way to put the store *beside* the
service it belongs to. That is fine for a diagram a generator emits and wrong
for the diagram someone is thinking with — people arrange boxes to say things
("these three are one subsystem") that no auto-layout can be told. The
interaction staged for "d3 later" in that spec turns out not to want d3 at
all: it wants an editor.

So `renderer="graph"` is a second renderer, not a second element: same
`<hick:diagram>`, same `asserts`, same warnings, same weave discipline. Its
body is a JSON **scene**, and the app puts an interactive canvas where the
mermaid panel would be — drag nodes, connect edges, rename, delete,
auto-layout — with every finished gesture written straight back into the
document text.

## The format

```
<hick:diagram renderer="graph" asserts="#no-back-edges">
{
  "nodes": [
    {"id": "api", "label": "API server"},
    {"id": "store", "label": "Postgres", "shape": "cylinder"}
  ],
  "edges": [
    {"from": "api", "to": "store", "label": "SQL"}
  ],
  "layout": {
    "api": {"x": 40, "y": 30, "w": 160, "h": 64},
    "store": {"x": 40, "y": 190}
  }
}
</hick:diagram>
```

One split does all the work: **topology is what a generator can deduce,
layout is what only a person decides.** `nodes` and `edges` carry ids, labels,
shapes (`rect`, `round`, `pill`, `circle`, `diamond`, `hexagon`, `cylinder`),
optional colours, edge styles (`solid`/`dashed`/`dotted`) and arrowheads
(`end`/`none`/`both`). `layout` maps node id → position, in integer pixels.
An edge may also record `fromSide`/`toSide` — which side of the box each end
meets (`top`/`right`/`bottom`/`left`), with an optional slot suffix
(`left.1`) when several lines share a side. Sides sit on the person's half of the split
in spirit (a generator never sets them; the editor records them when a line
is drawn or re-plugged) but live on the edge, because an edge with no nodes
to sit between is nothing; the mermaid downgrade ignores them, since mermaid
routes its own lines.

Ids are **semantic** (`"api"`, never a timestamp) on purpose twice over: they
are the join key that lets a regenerated topology keep a hand-made layout, and
they keep the file diffable — the serialization is canonical, stable key
order, **one node, edge, or layout entry per line**, so a moved box is a
one-line diff and a CRDT merge of two people editing one diagram usually
touches different lines.

Provenance of the schema: its shape is informed by the pure-data format grafly
documents in `GRAFLY_DIAGRAM_FORMAT.md`. Grafly itself is AGPL-3.0 and this
workspace is MIT-only, so **no grafly source was read or used** — the format
doc was treated as a description of data, the types here were written fresh,
and the editor is built directly on `@xyflow/react` (MIT), which is the
substrate grafly is built on too.

## The editor is not a store

The implementation rule that keeps this honest: **the document is the only
store.** The canvas commits on gesture *end* — drag ended, edge connected,
node added, renamed, deleted, auto-layout applied — through the same
`replaceBlockContent` path the table grid uses, as a user event, so undo, the
CRDT, and blame all see typing. The panel remembers only the last bytes it
wrote: a body that comes back equal is its own echo; anything else (undo, a
collaborator, the agent) resets the canvas from the document, which wins.
Each mounted panel is its own React Flow provider — N diagrams in one
document are N independent editors.

The canvas library is loaded lazily, exactly as mermaid is, and for the same
reason: the marketing site builds from this tree, and a diagramming engine
must not reach a page that shows no diagrams.

Two interaction rules, stated because they are choices and not defaults:

- **A line is re-pluggable, and selection is what tells the two grabs
  apart.** A line ends exactly where a node's connection dot sits, and both
  gestures want those pixels — so a bare drag from the dot draws a NEW
  line, while clicking the line first (anywhere along its 24px band) raises
  it above the nodes and the same drag then RE-PLUGS its end onto another
  node, or another side of the same node. A selected line shows a soft halo
  at each end saying "grab here". Every side of a node is one connection
  point, usable in either direction (loose connection mode): a person
  choosing where a line meets a box is choosing a side, not a polarity. On
  a derived scene re-plugging is off, because it changes topology and the
  topology belongs to the fragment.
- **A side offers as many points as it needs, and only sides offer them.**
  Connection bubbles are round, always faintly visible, and live at the
  sides' midlines — never at corners, which belong to the resizer's square
  grips, so what connects and what resizes never look alike. Each side draws
  one bubble per line already attached plus one free bubble, the group
  centred with a narrow gap: a side holding a line still has an open point
  right beside it, and the slot a line lands on is recorded (`left.1`).
- **The cursor should be near the thing, not exactly on it.** A drag snaps
  to a connection point from ~36px, an edge end is grabbable for re-plugging
  from ~24px, an edge is clickable along a 24px band rather than its
  one-pixel stroke, and each handle's hit target is larger than its visible
  dot; the resize grips are 16px to hit for a small square of ink, and the
  box's sides are a wide invisible resize band. Precision is for the layout,
  not for the acquiring of targets.

## Derived scenes: paste the topology, own the layout

A generator's diagram enters the document the doctrinal way — a fragment,
pinned by the cell that wrote it:

```
<hick:exec id="arch-scan" container="scip">
…emits the topology JSON…
<hick:expect match="exact">…pins it…</hick:expect>
</hick:exec>

<hick:copy id="arch-topology">{"nodes": […], "edges": […]}</hick:copy>

<hick:diagram renderer="graph" asserts="#arch-scan">
{
  "topology": <hick:paste select="#arch-topology" />,
  "layout": {"api": {"x": 40, "y": 30}}
}
</hick:diagram>
```

`topology` and inline `nodes`/`edges` are mutually exclusive — both would be
two sources of truth, and the parser refuses it.

In the editor a derived scene is **topology read-only**: drag, arrange,
auto-layout freely — commits rewrite only `layout`, and the paste tag is
written back byte-for-byte — but nodes and edges belong to the generator, and
an edit to them here would be silently destroyed by the next re-run. That
lock, plus layout keyed by semantic id, is the whole answer to "a re-run must
not destroy my arrangement": the regenerated fragment changes, ids that
survive keep their places, new ids appear beside the layout (visibly
unarranged; auto-layout is a button, never something a re-run does to you),
and layout entries for departed ids are dropped on the next commit.

## The weave: downgrade, and say so

A scene weaves to a **mermaid fence**: deterministic `flowchart TD`, labels,
shapes, edge styles — and no positions, because markdown has nowhere honest to
keep an x coordinate. The woven file still renders on GitHub with no hick
installed, which is the same bargain every renderer makes. A body that does
not parse weaves as a ```json fence — the reader sees what is there, never
nothing — and the warning about it appears at validation time beside the
asserts warnings, naming the line. A later upgrade, feasible because positions
are authored (no layout engine needed), is weaving a static SVG asset; it is
named here and not built.

## The one string a scene cannot hold

No raw-content parsing was added for this (`is_raw_content_tag` would kill the
`<hick:paste>` that derived scenes are made of). JSON never collides with the
no-escaping invariant except in one case: a string containing the literal
`</hick:` would close the diagram from inside its own body. The editor
**refuses to commit** such a label and says why, the same rule the session
payload format already has for `</hick:input>`.

## Assertions, unchanged

`asserts` works on a scene exactly as on mermaid — the warnings match the tag,
not the renderer. A hand-drawn scene with no `asserts` gets the same "nothing
checks this diagram" line; a derived scene cannot disagree with its fragment
*and* should still assert the cell that pins it, because the fragment can rot
against the code even though the picture cannot rot against the fragment.

## Styling, sizing, and the grid (built 2026-08-25)

Selection is the styling surface: select a node and an inspector row offers
the shape (all seven), and a **palette** — eight named hues stored as plain
hex — for fill (the same hues at low alpha, so text stays legible), outline,
and text. Select a line and the row offers its colour (arrowheads included),
an arrowhead cycle (one end / both / none), a dash cycle, and the line's own
label, drawn on the line. A palette rather than a picker on purpose: eight
named choices keep two diagrams in one project looking like siblings, and
keep the diff of "made the store red" one readable word. Derived scenes show
no inspector — styling lives on nodes and edges, which are the fragment's.

Nodes resize by their selection handles, and everything **snaps to one
16px grid** — drags, sizes, and the dot background all use the same number,
so boxes line up without anyone squinting. Resizing is layout and therefore
allowed on derived scenes.

Connectors keep their rounded elbows across open space and go **straight
between close neighbours**: a smoothstep between two adjacent boxes folds
its corner radii into an S in the little gap, so below ~96px the line is
drawn straight.

## What is not built yet

- **Edge waypoints** (a line you bend by hand).
- **The SVG-asset weave** named above.
- **Grafly import.** The format is close enough that a converter is a small,
  separable piece; nothing depends on it.
