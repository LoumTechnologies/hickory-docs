---
name: audience-first-docs
description: Communicate clearly with a specific reader — a human or an agent with a defined persona — whether writing docs or replying in an AI session or messaging another agent. Use when Codex is asked to improve docs, READMEs, guides, onboarding material, product docs, API docs, tutorials, explainers, documentation style, chat/session replies, or messages to another agent/persona; especially when the communication should be succinct, procedural, audience-specific, example-driven, presented as an interview or Q&A dialogue, or should rule out likely misunderstandings.
---

# Audience-First Communication

## Goal

Create communication — a document, a chat reply, a status update, a message to another agent, anything written for a reader to consume — that answers the reader's next question before they have to ask it. Prefer procedural explanations, recognizable examples, and the smallest amount of context needed for the target reader to act correctly. This applies equally to a README, a reply typed into an AI session, and a message sent to another agent operating under its own persona: the reader's vocabulary, priorities, and default assumptions still determine what the message should say.

A "reader" here is whoever consumes the message, human or not. An agent with a defined persona (a data-engineer agent, a marketing agent, a support-triage agent) has the same kind of narrow audience profile a human role does: it has its own vocabulary, its own priorities, and its own likely follow-up questions. The same underlying fact — "this endpoint is returning stale data" — should not be phrased the same way for both; write to the persona in front of you, not to a generic recipient.

## Workflow

1. Define the audience narrowly before writing. This includes agent personas, not just human roles.
   - Bad: "developers"
   - Better: "backend engineers integrating a new API"
   - Best: "backend engineers who already call our v1 API and need to migrate to v2 without breaking existing clients"
   - Agent persona example: a caching bug reported to a data-engineer agent should lead with the data staleness and the query/table involved; the same bug reported to a marketing agent should lead with what's visibly wrong on the page and skip the cache-invalidation mechanism entirely.
2. Write down the reader's likely questions in order.
   - "What problem is this solving?"
   - "Is this for my situation?"
   - "What do I type/do first?"
   - "What changes in my existing workflow?"
   - "What happens behind the scenes?"
   - "What should I not assume?"
3. Answer those questions in that order.
   - Start with the reader's existing pain or task.
   - Introduce only the concepts needed for the next action.
   - Use one concrete example before generalizing.
   - Explain mechanisms procedurally: "first this runs, then this is checked, then this result is returned."
4. Remove generic material.
   - Delete paragraphs that would apply to any product or any answer.
   - Delete examples the audience would not recognize from their own work.
   - Replace abstract benefits with concrete before/after behavior.
5. Add misunderstanding guards.
   - Contrast similar concepts early.
   - State boundaries directly: "The cache is per-user; the config is shared across users."
   - Name prerequisites before steps that depend on them.
   - Mention the hidden mechanism when it explains surprising behavior.
   - Distinguish the interface from the underlying mechanism. For example, write "the process retries with backoff, which the client library exposes as `retries: 3`" in prose, and reserve `retries: 3` for command examples or option references.

## Writing Rules

- Always state (or silently hold in mind) the target audience, as narrowly as possible — including which persona, if the reader is an agent.
- Prefer procedure over taxonomy. People understand "do A, then B happens" faster than category lists.
- Put the first successful path before references, options, or edge cases.
- Use examples that match the audience's existing tools, errors, file names, and commands.
- Keep sentences short. One idea per paragraph.
- Use "this means..." sparingly to translate unfamiliar mechanisms into reader outcomes.
- Explain enough internals to prevent wrong mental models, not to show implementation detail.
- Do not let shorthand imply the wrong interface. If a value appears as an environment variable, flag, or config key but represents a broader mechanism, name the mechanism first and the syntax second.
- Make contrasts explicit when names are similar.
- Keep reference material (tables, flag lists, full option sets) after the main path, not before it.
- Prefer stable, copyable commands over prose when the reader will need to run something.
- When a concept keeps drawing follow-up questions, consider staging it as a short interview instead of a monologue. See Interview Format.
- In a chat or AI-session reply, this same discipline applies at a smaller scale: lead with the answer, not the reasoning that produced it; only unpack the mechanism if it prevents a wrong mental model or the reader is likely to ask "wait, why."
- When messaging another agent, write in that agent's persona's terms, not your own: use the vocabulary and priorities its role implies, and omit context it would already have or wouldn't act on.

## Interview Format

Readers learn faster from a transcript of someone asking the questions they
would have asked than from an expert explaining unprompted. The questions mark
where the reader is allowed to be confused, and each answer arrives at the
moment the reader wants it.

Use this format when:

- The topic has a surprising mechanism the reader will resist believing.
- The reader's likely questions are objections ("why not just do X?").
- A design decision only makes sense after its alternatives are ruled out.
- Prose keeps growing hedges, asides, and parentheticals to pre-empt confusion.

Do not use it when:

- The reader needs copyable steps. Install, setup, and command reference stay
  procedural.
- The content is a list of options, flags, or fields.
- There is only one question. A single Q&A pair is a heading with a paragraph
  under it; write it that way.
- It cannot go at or very near the top. See placement, below.

Placement: the opening of a communication sets an implicit promise about its
style — prose signals prose, a numbered list signals procedure. Introducing
two speakers partway through breaks that promise and disorients the reader,
who has to re-parse who is talking and why the format just changed. If a
document is going to use the interview format, open with it, or open with no
more than one short framing sentence before it starts. Never drop an
interview into the middle of an otherwise prose document; if the format only
earns its place partway through, either restructure so it leads, or the
content doesn't need it — write it as prose instead.

How to write one:

1. Name both speakers by role, not by personality: `Reader:` and `Maintainer:`,
   or `New engineer:` and `On-call:`. Two roles the audience recognizes.
2. Draw the questions from the reader-question list built in Workflow step 2.
   Ask them in the order they occur to the reader, including the blunt ones.
3. Keep each answer to a few sentences. If an answer needs a code block, put the
   block after the answer line, not inside it.
4. Let the reader push back at least once. An unchallenged interview is a
   monologue with question marks: "That still sounds like a workaround. What is
   different?"
5. End the exchange once the reader could act. Do not stretch it to cover
   reference material.

Example shape:

> **Reader:** I already validate this on the client. Why does the server check again?
>
> **Maintainer:** The client check catches typos before a request is sent. The
> server check is what actually protects the data — a client check can always
> be bypassed by whatever calls the API directly.
>
> **Reader:** So the client check is pointless?
>
> **Maintainer:** No — it's still what gives users instant feedback. Keep both;
> they serve different readers.

Format it as a blockquote, a definition list, or bolded speaker labels,
whichever the surrounding document already uses. Keep the same two speakers for
the length of one section; do not introduce a third voice.

## Structure Template

Use this shape unless the artifact has a stronger existing structure. For a
document this is the section order; for a chat reply, treat it as a
compressed checklist — most exchanges only need steps 2 and 4.

1. Audience: one narrow sentence.
2. Problem: one recognizable situation or question.
3. Core model: the smallest explanation that makes the answer make sense.
4. First procedure or answer: the copyable steps or the direct answer.
5. What just happened: procedural explanation of the mechanism, as prose. If
   this is where an interview would naturally go, see the placement rule in
   Interview Format — reaching step 5 is already too late to introduce two
   speakers; use it only if the whole document opens with it instead.
6. Common variations: only the variations this audience likely needs.
7. Misunderstandings: short bullets that rule out wrong assumptions.
8. Reference: commands, flags, variables, or API surface.

## Quality Check

Before finalizing, verify:

- The audience is narrow enough that another audience would need a different answer.
- The opening answers "why should I care?" in the reader's terms.
- The first example is immediately recognizable to that audience.
- Each section (or sentence, in a short reply) answers a likely question at the moment it would arise.
- Procedural explanations outnumber abstract descriptions.
- Similar concepts are contrasted before the reader can confuse them.
- Any interview section asks the reader's real questions, including at least one objection, and is not a disguised monologue.
- If an interview is used, it starts at or very near the top of the communication, not introduced partway through.
