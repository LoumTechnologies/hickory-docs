# An Action Is Asked Of The Element, And Carried Out By The Host

Given a block the app draws, when a client asks for an action on it —
`POST /api/docs/:id/blocks/:at/:action`, `:at` being the byte its tag
starts at — then the element that declared that tag answers, and it
answers with a *request* (`hick_blocks::ActionOutcome`: an answer, a run
of these cells, an edit of this span) which the server carries out with
the machinery its older routes already use. A cell's `run` starts the
same run `POST /run` starts and answers `202` with the same run id; an
`edit` writes the document the way `PUT` writes it and tells the live
room; an `answer` is returned as is. An action the element does not
declare is `404` naming the element and the action; one it refuses is
`422` in its own words; a byte where no element starts is `422`. And
`GET /api/elements` lists every element with its kind, attributes and
actions, read from the same registry `/render` draws from.

The point is the seam. Before this, a verb on a block was a route of its
own, written in the server against the server's state — 92 of them — and
nothing tied a route to the element it was about. Now an element is one
declaration (`an-element-is-declared-once.md`) and its verbs are part of
it; the server has one route for all of them and knows nothing about any
particular element.

Three properties hold it up:

1. **The element never runs anything.** `Element::act` returns an
   `ActionOutcome`, never performs one. `hick-blocks` has no executor, no
   filesystem and no server; the render route can therefore never become
   a way to execute a document, however many actions are added.
2. **Addressed by span, not ordinal.** The byte offset the tag starts at
   is the `span[0]` its block carries, so an action lands on the block a
   person pointed at even after a block is inserted above it — the same
   reason the editor keys rendered blocks by mapped position.
3. **One translation of failure.** `ActionError::Unknown` is `404`,
   `Refused` is `422`, `Failed` is `500`, in `api::block_action` and
   nowhere else.

## Boundary

The routes live in `hickory-cli`'s `serve` module beside the routes they
will replace; the older `POST /run` still exists and the web client still
calls it. A separate `hick-server` crate waits on `LocalState` being split
from the CLI, which is not done. Only `exec` declares an action (`run`);
no element yet returns an `Edit`, though the server carries one out.

---

Last LLM verification:
- Date: 2026-09-05
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hick-blocks/src/lib.rs` — `ActionOutcome`,
  `Registry::act`, `find_tag_at`; `crates/hick-literate/src/render.rs` —
  `ExecElement::actions`/`act`; `crates/hickory-cli/src/lib.rs`
  `block_action`; `crates/hickory-cli/src/serve/api.rs` `elements` and
  `block_action` (status translation, the edit splice checked against char
  boundaries); routes in `serve/mod.rs`;
  `crates/hickory-cli/tests/block_actions.rs` drives all four cases over
  real HTTP; `docs/specs/freeform/api.md` records the contract.
