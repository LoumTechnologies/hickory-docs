# Three provenances, kept apart

*Status: design of record, adopted 2026-08-22, implemented the same day.
Grew out of `receipts-for-a-message.md`: once a message's sentences could be
traced to a meeting, the question became what "traced" means — and it means
three different things that a reader must be able to see separately, together,
or not at all. Pairs with `provenance-and-standing.md` (derived vs declared)
and `agent-cells.md` (the agent's only write path is the tool surface).*

When someone asks *why is this here?* — or *prove that Slack message was not
made up on the spot* — there are three honest answers, and they are not the
same answer:

| | What it says | Who derives it | Can it be forged? | Drawn as |
|---|---|---|---|---|
| **Lineage** | these bytes in the output came from this span of that document | the weave/tangle (`hick lineage`) | no | solid bands/braces in the lineage palette |
| **Context** | when an agent wrote these lines, *this* was in front of the model — this file at this hash and commit, these lines; this prompt; this tool result; this observation | the session record (`hick context`) | no, as long as the tool surface is the only way the model reads — and it is | dotted, amber |
| **Declared** | the author says this rests on those fragments (`cites="#id,.class"`) | nobody — it is asserted (`hick cites`) | yes, by anyone | dashed, grey, worded as an assertion |

Each is useful; each has a caveat the others do not; and the failure this
document exists to prevent is a reader taking one for another — a dashed
citation read as proof, or a context ribbon read as "this sentence was derived
from that turn" when it only says the turn was present.

## Lineage

Unchanged: `Provenance[]` out of the pipeline, byte-exact, recomputed on every
weave, across documents (`docs/guarantees/lineage/a-ribbon-crosses-documents.md`).
It answers *where did these bytes come from* and nothing about why.

## Context

**Derived from the conversation, never from the model.** Every input to a
session already has an element in the session file — the user's words
(`hick:user`), a script's observation (`hick:observation`), a tool's result
(`hick:tool-result`) — and each now carries an `id` (`in0`, `in1`, …). Two
elements are new and written by the harness, not the model:

- `<hick:read file="…" commit="…" sha256="…" lines="a-b"/>` after a read tool
  (`read_doc`, `read_output`, `read_file`) — the file shown, at its content
  hash and, when in a repository, its commit; which lines.
- `<hick:wrote file="…" lines="a-b" hashes="…"/>` after an edit tool — the
  lines the edit left, as they stand after it, and their hashline hashes.

The derivation is one rule: **a write's context is every input before it in
the same session.** `hickory_agent::context` walks a session and, for each
`wrote`, lists the preceding inputs — file reads as (path, commit, sha256,
lines); everything else as a *summary* (element, id, first line, sha256 of
the text, line count, the element's line in the session) that points back at
the session element. The written lines are then found in the document as it
stands now by their hashes; if they have moved on, the write is still a fact
and the context still holds, but nothing in the file today is those bytes.

What this is NOT: it does not say which input "caused" which line. No
algorithm knows that, and the model's opinion on it is declared provenance.
It says *present*, and it says it in a way anyone holding the session can
re-derive without a model.

Two consequences were built into the tools:

- **`read_file`.** The agent's scripts run in a scratch workspace that cannot
  see the project — on purpose, so the document remains the only write path.
  Before `read_file` the agent, told a file existed, looked, found nothing,
  and made one up (`examples/receipts/hick-agent/sessions/20260822-125634-…`).
  Now it reads the real file through a tool that records the read, which is
  both the fix and the provenance.
- **Sessions are the user's own.** They hold every prompt, result, and file
  shown, and they are read locally; `hick init` adds `sessions/` to
  `.gitignore`. The checked-in artifact is the document; the session is the
  user's record of how it got that way. (The sessions under
  `examples/receipts/` are committed as fixtures of this repository — they
  are the evidence the example exists to show — not as a pattern for a notes
  repository.)

Surface: `hick context <doc> [--json]`; `GET /api/docs/:id/context`; the
app's **Context** toggle. Guarantee:
`docs/guarantees/agent/context-provenance-is-derived-from-the-session.md`.

## Declared

`cites="…"` on any element of the document — a `claim`, a `transform`, a
`copy` — names, with the selector grammar that already exists, what the
author says the element rests on. It is resolved (upstreams and transcript
turns included) so it can be drawn and so a dangling citation is reported
rather than hidden; it weaves as a trailing line in words (`*cites: #a, #b*`),
never as a mark; and the app draws it dashed and grey with a title that says
*an assertion, not a derivation*.

`hick refresh` writes `cites=` on a transform from the `#id`s the passage
itself mentions, filtered to the fragments it was shown (the transform input
labels each fragment `[#id]`). That is the model's own claim about what it
leaned on, stamped as such — distinct from `from=`, which is what it was
shown and is fingerprinted, and from `select=`, which is what it was asked to
read.

Surface: `hick cites <doc> [--json]`; `GET /api/docs/:id/cites`; the app's
**Declared** toggle. Guarantee:
`docs/guarantees/lineage/three-provenances-are-drawn-apart.md`.

## The toggles

Three toggles in the status bar — Lineage, Context, Declared — each with a
swatch in its family's stroke and a tooltip saying what it is and what it
cannot claim; any subset, remembered per browser. The overlay draws each
family in its own hue *and* its own stroke pattern, so they stay apart in
monochrome and for colour-blind readers.

## Open edges

- **Context is line-granular and per-write.** Hashline anchors are how edits
  land, so that is the unit. A later edit to the same line by a person
  changes its hash and the write stops resolving — correctly: the bytes are no
  longer the agent's — but the history that it once was is only in the
  session.
- **The summary of a conversation input is a convenience, not a primitive.**
  It is derived on read from the session element; nothing is stored twice.
  If the session file is edited, the derivation follows it — which is why
  sessions being the user's own matters.
- **Ribbons from a verdict to what it cites are declared, not derived.** The
  honest derived alternative — verbatim-span matching between the passage and
  its input — is not built; it would give real ribbons to quoted words and
  none to paraphrase, which is the right asymmetry.
- **Pane-to-pane drawing for context/declared far ends** is not built: they
  terminate on a tab, a tree row, or a port, and a click opens the target.
  Selecting the exact lines in the opened file is the same gap the lineage
  click has across documents.
