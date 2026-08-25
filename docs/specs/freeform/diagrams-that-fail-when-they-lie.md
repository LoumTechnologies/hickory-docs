# Diagrams that fail when they lie

## The problem

You cannot understand a large codebase by reading it. So people draw: three
boxes, some arrows, a paragraph saying which direction calls are allowed to
go. The drawing is the most useful artifact in the repository on the day it is
made and the most misleading one a quarter later, because it is the only
artifact nothing re-checks. Tests fail. Types fail. Diagrams just quietly stop
being true, and the next person builds on a picture that describes a system
that no longer exists.

Hickory's whole argument is that a claim in a document can be executed. A
diagram is a claim — arguably the densest one anybody writes down. So it should
fail like any other.

## The shape

```
<hick:diagram renderer="mermaid" asserts="#no-back-edges">
flowchart TD
  api --> core
  core --> store
</hick:diagram>

<hick:exec id="no-back-edges" container="scip">
cd project && scip-python index . --project-name app
scip print --json index.scip | jq '[.documents[]
  | select(.relative_path | startswith("store/"))
  | .occurrences[] | select(.symbol | contains("api/"))] | length'
<hick:expect match="exact">0
</hick:expect>
</hick:exec>
```

The picture is above; the proof is below; `asserts` binds them. When someone
adds the call the arrows forbid, the cell's output stops being `0`,
`hick test` exits 3 (expectation failed — the outcome no automation may
auto-fix), and the pre-commit hook refuses the commit. Nobody had to remember
the diagram existed.

## Why a tag and not a markdown fence

A ```mermaid fence in prose would have cost no grammar and rendered on GitHub.
It was the first thing tried and it is the wrong answer, for one reason: a
fence cannot be *referred to*. `asserts` needs somewhere to live, and the only
places a fence offers are its info string or a naming convention — stringly
typed configuration, and convention is exactly what rots.

With a tag:

- **The binding is declared, so it can be checked.** A diagram naming an id
  that no longer exists is a warning at weave time
  (`diagram_assertion_warnings`), which is what catches the rename that
  silently unhooked the proof from the picture.
- **An unchecked diagram is visibly unchecked.** No `asserts` is legal — a
  sketch of something outside this repository is a fine thing to draw — but it
  warns once and the notebook says so under the picture. The failure mode this
  whole feature exists to prevent is a drawing that *looks* authoritative.
- **A diagram can be derived rather than drawn.** `<hick:paste>` works inside
  it like anywhere else, so the edge list can come from the cell that computed
  it. A generated picture cannot disagree with its source, which is strictly
  better than one that merely gets checked.

The fence's one real advantage is kept anyway: `hick:diagram` weaves *into* a
fenced block tagged with its renderer, so the woven markdown renders on GitHub,
in an editor preview, and anywhere else a reader opens it — with no hick
installed and no notebook. Tag is the source form; fence is the output form,
the same relationship `hick:file` already has with what it emits.

A bare fence in prose is deliberately NOT rendered as a diagram in the
notebook. Two ways to draw the same thing means the checkable way is optional,
and an optional discipline is not one.

## Renderers, in order

`renderer` is an attribute rather than part of the tag name so that adding one
is a value and not a migration.

1. **mermaid — now.** It renders everywhere a markdown file lands, which keeps
   the woven output honest for a reader who never opens the notebook. It is
   also what a coding agent emits without being asked, which matters when the
   diagrams are generated.
2. **graph — built (2026-08-25).** The interaction role this list staged for
   d3, taken instead by an editor: a JSON scene body (topology apart from
   layout), an interactive canvas in the app that writes every gesture back
   into the document, and a weave that downgrades to a mermaid fence so the
   woven markdown still renders everywhere. See
   `a-diagram-you-can-drag.md` — including why it is a renderer value and not
   a new element, which this section predicted.
3. **d3 — probably never now.** What it was staged for (interrogate the
   picture) is the graph renderer's job; reach for d3 only if a *reading*
   interaction appears that an editor cannot host.
4. **first-party SVG — maybe never.** Only if the above are proven
   inadequate. The cost is a layout engine, and layout engines are a career.

The notebook draws what it can and leaves the rest as source: an unknown
renderer says so, and the document still weaves.

## What is not built yet

- ~~**Live assertion state in the panel.**~~ Built (2026-08-25): the panel
  maps each `asserts` id through the cell that carries it to its last-run
  state, and says "out of date" in the document the moment a named cell
  fails. Derived bodies draw too — the block model resolves this document's
  pastes, so a picture whose edges come from a fragment is no longer a paste
  tag on screen.
- ~~**Deriving a diagram from a cell's output.**~~ Built (2026-08-25):
  `examples/architecture-that-draws-itself.hick` is the worked loop — code,
  an expect-pinned `hick diagram` scan (the new deterministic generator over
  `hick-structure`; `--refresh` re-writes the fragment, layout untouched), a
  fragment, a derived graph diagram, and the SCIP-in-a-user-container
  variant shown emitting the same topology shape.
- **The skill.** A Claude Code skill that reads a repository and writes one of
  these documents — the layers it finds, the picture, and the assertions that
  hold it in place — is the point of the whole feature. It is a separate piece
  of work and belongs in its own spec once the pieces below it are steady.
