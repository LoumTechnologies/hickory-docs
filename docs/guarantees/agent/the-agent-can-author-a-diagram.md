# The Agent Can Author A Diagram

Given the built-in agent editing a document, when a diagram is the right form
for what it has to say, then the document language its system prompt teaches
includes `<hick:diagram>` — with the discipline attached: name the proving
cell in `asserts` or be a deliberate sketch; for `renderer="graph"`, emit
semantic node ids, omit `layout` (arranging is the person's half; auto-layout
fills in), and paste an existing topology fragment rather than restating it.

The reason: the prompt says "only these elements exist; do not invent
attributes", which is the right rule for an agent writing into a no-escaping
format — and it makes the element list a whitelist. Before this guarantee,
`diagram` was absent from that list, so the agent literally could not draw:
the feature's second authoring path (spec:
`docs/specs/freeform/a-diagram-you-can-drag.md`, use case "an AI agent
authors a diagram") failed at the prompt, not at the tools. Claude Code and
other outside agents author documents directly and get the same discipline
from the specs; the built-in agent gets it only from this prompt.

---

Last LLM verification:
- Date: 2026-08-25
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence:
  - `crates/hickory-agent/src/protocol.rs` `TOOLS_SYSTEM_PROMPT`, "The
    document language" section: the `<hick:diagram>` entry carries both
    renderers, the asserts discipline, semantic-id and omit-layout rules for
    `graph`, and the paste-not-restate rule (which is the document-chain
    doctrine the same prompt already states for prose).
  - The agent's edits reach the document through `edit_doc`
    (`tools/mod.rs`), which writes raw text — no new tool was needed; only
    the whitelist entry.
- Caveats — what LLM review could NOT establish:
  - No end-to-end run against a live model has been recorded showing the
    agent drawing unprompted; the guarantee is that the language and
    discipline are in front of it, not that any given model uses them well.
    A worked example session belongs beside the deterministic-generation
    example when that lands.
- Test coverage: `crates/hickory-agent/src/protocol.rs::tests::
  the_document_language_names_the_diagram_and_its_discipline` (names this
  guarantee) pins the prompt's element entry and each sentence of the
  discipline, so dropping any of them fails the build.
