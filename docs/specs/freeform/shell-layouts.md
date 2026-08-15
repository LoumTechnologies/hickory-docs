# The shell: panes, tabs, and layouts a document declares

*Status: design of record for the desktop app's arrangement. Adopted
2026-08-15. **Supersedes** the `Document | Split | Output` mode switcher in
`DocumentView`, which is to be removed once the pieces below can do everything
it could.*

A note on the word: this file lives in `docs/specs/freeform/` because that is
where freeform specs live, and it also describes a layout called **freeform**.
The collision is unfortunate and the term is the one in use, so it stays.

## The decision

The app opens a **file** or a **folder**, the way VS Code and Zed do, and the
two answer different questions.

* **A file** opens into the **freeform layout**: new files arrive as tabs, and
  a pane can be split horizontally or vertically or given more tabs — an
  ordinary tiling editor with no opinion about your codebase, because opening
  one file gives it no grounds for one.
* **A folder** may contain **layouts**: documents that declare regions of the
  codebase and how they relate. The app lists what it found and offers them.
  Freeform remains on that list and remains the fallback.

Modes are gone. `Document | Split | Output` asked a person to choose between
three arrangements someone else designed, each hiding something they might
want. What replaces it is one arrangement they build, or one a document
declares.

**Columns are not a feature.** The lineage browser's one-column-per-stage view
is what a layout declaring one region per stage looks like. It stops being a
destination with its own route and becomes a layout you can pick — which is
also why nothing here adds a "columns mode".

## The three layers, and which one is a document

| Layer | Holds | Lives | Changes when |
|---|---|---|---|
| **Declaration** | Regions, their membership globs, how they relate | A `.hick` document, in git | The architecture changes |
| **Derivation** | Region graph + viewport → an arrangement of tiles | Code. Deterministic; nothing stored | Never: it is a function |
| **Session** | What is open, focused, split, resized, which tab is in front | Untracked, per machine, beside scroll position | Constantly, and nobody reviews it |

A document declares **structure**, never geometry. The moment a `.hick` file
carries pane widths and the front tab it becomes a config file with a weave
step, and it changes every time somebody drags a splitter: layout is
high-frequency, low-value churn, and architecture is low-frequency,
high-value. They do not share a file.

The discipline that follows: **every declared layout corresponds to a claim
about the code that a run can check.** Wanting a different arrangement means
writing a different true statement — layers, pipeline, crates — not a different
layout config. Wanting the debugger bottom-right at 300px is session state
asking to be session state.

## The model

* A **pane** is a leaf: an ordered list of tabs and one active tab.
* A **tab** is a view of something — a document, a generated file, or a tool.
* A **split** is a node: a direction, children, and their sizes.
* A **layout** is the tree plus, when declared, the rule that says which pane a
  newly opened file belongs in.

Freeform is the same model with an empty rule: a new file opens where the focus
is. A declared layout supplies membership globs, and those globs answer the
question a tiling shell otherwise has to invent an answer to — *where does this
file open?* That is the whole reason the declaration is worth having.

## Two kinds of tile

**Regions** come from the declaration. **Tools** come from the app — the
debugger, a transcript, an agent. Calling the debugger an architectural layer
to get it on screen is the category error that would rot this: a tool is not a
region, and a layout that pretends otherwise stops being checkable.

Tools follow one rule: **a tool that describes one region lives in that
region's tile**, as a tab or a strip. Violations for this region, tests for
this region, what this region depends on. A tool that describes the whole
session — the debugger's controls — belongs in the editor it is driving, not in
a dock (see `literate-debugging.md`, and the principle below).

## Tools belong in the editor

A list of symbols in a side panel asks you to look away from the code to learn
something about the code. A control above the editor does not. A panel earns
its place only when the information cannot be put on, in, or above the text:
the stack is a breadcrumb and gutter arrows, values are inline and in hovers,
evaluation happens on the line it is about.

The costs are real and accepted: a panel advertises itself and a keystroke does
not; sixty locals do not fit at the ends of lines, so something must still hold
"all of it"; and combo boxes reward people who know what they want over people
who are browsing.

## Zoom, if it happens

Scale is a real axis here — architecture holds regions hold files hold lines —
and the innermost level already exists as the lineage browser's holes.
**Discrete levels on a keystroke** (architecture → region → file) with a short
transition, showing different content at each level rather than the same
content smaller. Not a canvas: continuous zoom fights text rendering, spends
goodwill on motion, and makes focus ambiguous mid-gesture. This is optional and
last.

## Order of work

1. ~~**The model, as pure functions.**~~ **Done** — `apps/web/src/shell/layout.ts`:
   tile tree, splits, tabs, focus, glob routing, with tests.
2. ~~**The freeform shell.**~~ **Done** — `ShellView.tsx`. Panes, tab strips,
   draggable dividers, `Cmd/Ctrl-\` to split and `Cmd/Ctrl-W` to close.
3. ~~**A pane hosts the real editor.**~~ **Done** — a `document` tab is the
   full `DocumentEditor`: CRDT, LSP, gutter, debugger. A `generated` tab is
   the live output buffer with its provenance under it.
4. ~~**Folder open: discover and pick.**~~ **Done** — `layouts.ts` reads
   region declarations out of the folder's documents and lists them beside
   Freeform. A folder that declares exactly one opens into it.
5. ~~**Declared layouts derive a tree** and route opens by glob.~~ **Done**,
   and verified in the app: a `.md` opened from a layout with a `generated`
   region lands in that region without being told.
6. **Port what exists** — still open. The ribbons survive whole as the
   "Document & outputs" layout, which is a composite rather than a set of
   regions; the lineage columns are still their own route. Both should become
   ordinary layouts, and then "where did this come from" needs ONE answer
   rather than two.
7. ~~**Delete the mode switcher.**~~ **Done.**

## Also true now

* **Opening a single document costs nothing.** The desktop picker offers a
  document or a folder, the engine already accepted either, and a session with
  exactly one document opens it rather than listing it.
* **A layout picker appears only when there is a choice.** One option is not a
  choice, and a control that offers one is asking a question it knows the
  answer to.

## Open questions

* **Where do layout documents live?** A convention (`*.layout.hick`, or any
  document declaring regions) decides whether "open a folder" has to weave
  everything to find them, which decides how fast the picker is.
* **Ribbons or links.** Both draw provenance. Keeping both is defensible;
  keeping both by accident is not.
* **How many regions fit.** A twelve-region declaration is a true statement and
  an unusable screen. Collapse, zoom, and "a region with nothing open is a
  strip" are the candidate answers.
* **Session persistence.** Per folder, per layout, per machine — and what
  happens to a saved session when the layout it belonged to changes.
