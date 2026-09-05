# The minimal core: line-numbered documents, React components, Rust backends

*Status: adopted 2026-09-05. Steps 1 and 2 built the same day
(`the-editor-reads-with-the-parser-the-server-uses.md`,
`an-element-is-declared-once.md`), step 3's routes
(`an-action-is-asked-of-the-element.md`), and step 4's registry
(`an-element-is-drawn-by-its-view.md`); the rest is in progress; each is shippable on its own and each deletes more than it
adds. Nothing here changes what a document is, what it means, or what any
`.hick` file on disk does.*

> "I want the core to be minimal — mostly about a way of defining
> line-numbered documents with react components and rust backends for those
> react components."

## What the codebase was on 2026-09-05

Thirty days old, 420 commits, about 150k lines of Rust in 35 crates and
72k of TypeScript. The core the sentence above describes already existed
and was small and good: `hick-lang` (3.4k lines, two dependencies) parses
text plus namespaced tags; `hick-literate`'s block model turns a run into
`{kind, span, props}` the React side draws; the `Executor` trait is a clean
boundary. But the *architecture* was not that skeleton. It was
`hickory-cli`: 60k lines, 141 files, 92 HTTP routes, a `main.rs` of 4.4k
lines, depending on 21 sibling crates and owning git, fleet, broker,
scaffolding, debugging, LSP bridging, terminals, the agent, ingest, lenses
and the up-loop. Nothing below it decided anything.

What that cost, measured rather than assumed:

- **Three definitions of "what is a document".** `hick-lang` in Rust, a
  651-line hand-rolled parser in the editor, and a mock server for the
  demo. The two parsers disagreed on 22 of this repository's 29 documents.
- **A vocabulary declared nowhere.** About fifty tag names and sixty
  attribute names matched as string literals across the crates. The block
  model exposed four kinds, the card rail knew six, the editor's parser
  about twenty. Adding one element touched a Rust enum, a TS union, the TS
  parser, the card list, the editor's hardcoded panel chain, and the mock.
- **Dead weight.** `hick-grove`, `hick-xml` and `hick-sink` had no callers
  outside themselves; `hick-store` served one enum; six more crates had one
  or two callers.
- **Rules set and not enforced.** No file-length or complexity lint despite
  the CI instructions requiring one; a workspace licence of
  `GPL-3.0-or-later` beside a no-copyleft policy and an architecture doc
  saying MIT.

## The five steps

1. **One parser.** `hick-lang` compiled to WebAssembly is the editor's
   parser; the TypeScript one is deleted. The marketing demo keeps working
   and now runs the real parser. **Byte spans stay** — provenance is
   byte-precise, and the one place bytes meet UTF-16 indices is one
   conversion per parse. *Built.*
2. **An element registry** (`hick-blocks`). An element is one Rust value
   declaring its tag name, its attribute schema, how it renders to block
   props, and the actions it owns. The registry replaces the string matches,
   and the block model becomes uniform: `{kind, span, props}` for every
   element, `kind` resolved by the registry. *Built:* `hick-blocks`, with
   the four elements the app already drew registered from `hick-literate`
   and the wire shape unchanged.
3. **A generic server** (`hick-server`). Read and write a document, render
   its blocks, subscribe to changes, and one action route dispatched to the
   element's backend. `hickory-cli` shrinks to argument parsing and the
   commands. Routes an element does not own migrate or stay as extensions.
   *Built so far:* `GET /api/elements` and
   `POST /api/docs/:id/blocks/:at/:action`, with `exec`'s `run` as the
   first action through it, beside the older routes. *Not yet:* the crate
   split, which waits on `LocalState` being separable from the CLI.
4. **The frontend mirror.** A component registry keyed by the same `kind`,
   one generic document editor that draws lines and mounts components, and
   each element a folder pair — Rust beside TSX. The hardcoded panel chain
   goes away. *Built:* `apps/web/src/elements`, one folder per kind, the
   editor's branch chain replaced by a lookup, a cell's Run through the
   action route. *Kept on purpose:* the demo's mock server — the site's
   demos stay and now run the real parser, which was the requirement.
   *Not yet:* the Insert menu read from `GET /api/elements`, and the two
   registries as one list (see the guarantee's boundary).
5. **Three bins for everything else.** *Element*: exec, file, diagram,
   table, math, picture, ingested, sample, session turns. *Extension outside
   the core*: the git pane and lenses, the LSP and DAP bridges, terminals,
   the agent, ingest, scaffolding, the index, fleet, broker, peer. *Parked*:
   `hick-grove`, `hick-xml`, `hick-sink`, `hick-store` and the one- or
   two-caller crates, until an element needs them — parked, not deleted,
   until told otherwise.

Alongside: a file-length lint (*built* as a ratchet,
`scripts/check-file-length.sh`: no source file passes a thousand lines and
the twenty-five already past it may only shrink), the licence
contradiction resolved (*not done* — the product's licence is the owner's
decision, and it is flagged rather than changed), and `AGENTS.md` given a
one-screen entry point that describes the core and points here (*done*;
the record beneath it is kept, because each paragraph is a decision).

## What changes for a person, and what does not

About nine parts in ten of this is where code lives, not what it does. The
grammar, every document on disk, the CLI verbs, the cards, run and weave
and test, lineage, the git pane, debugging, LSP, terminals, the agent and
ingest are unchanged. What does change, on purpose: the editor and the
server can no longer disagree about structure (step 1 surfaced cases the
old parser silently tolerated); an element is drawn by one component
everywhere it appears, so the history lens and the merged view lose
quirks of their own; and adding an element becomes one Rust file and one
TSX file, which is the only new capability in the design and the one the
opening sentence is really asking for.

## What is refused

- **No line-based coordinates.** "Line-numbered" is what the person sees in
  the gutter; the record is bytes. A block is addressed by its byte span,
  and lines are derived at the edge.
- **No second parser, ever again, for any reason** — not for speed, not for
  a demo, not for a lens. If a reader needs structure it loads the parser.
- **No registry that runs anything.** An element's backend renders props
  and answers actions; execution stays behind `Executor`, and the rule
  against a wasm *container runtime* stands. The parser as a library is not
  a runtime.
- **No rewrite.** Extract in place, one step at a time, with the old route
  and the new route side by side until the old one has no callers.
