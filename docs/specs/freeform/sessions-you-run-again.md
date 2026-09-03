# Sessions you run again: the carry, the interview, and nothing from nowhere

*Status: design of record for how a session is refined. Adopted 2026-08-23.
**Sequence steps 1 and 2 are built** (2026-08-23): rewind and re-run named
apart in the dock with what each keeps stated where the choice is made, and
`hick ingest --from carry` as the explicit artifact of ending a session. Steps 3–5 (edited
sessions, self-containment as a check, equivalence as an instrument) are not.
See `docs/guarantees/agent/rewind-and-re-run-are-two-different-acts.md`. It grew out of `changes-not-commits.md`, which asked
whether a session could produce a commit, and amends that document's answer.
It depends on `a-session-is-the-conversation.md` (the guarantee) for what a
session file already is, and on `three-provenances.md` for the rule that keeps
the three kinds of session apart. Pairs with
`one-engineer-many-machines.md`, which is what makes a session run somewhere
other than where you are sitting.*

You run an agent session, you get the result you want, and along the way you
changed your mind twice. So you run it again with a better opening prompt and
different answers to the questions, and you get there without the wandering.

The observation that makes this design possible is that **the second session is
not a reenactment.** Every byte of it was written by the harness, the tools
really ran, the result really came out. It is a complete record of itself, and
it stands without the first session ever having existed. There is no saying you
could not have skipped the first one had you been more prescient — you simply
were not, and the first attempt is a draft.

Which the product already treats correctly by accident: `hick init` gitignores
`sessions/`, so discarding the first attempt is the default rather than a
decision.

## Rewind and re-run are two different acts

The dock already lets you change your mind inside a conversation. The turn tree
is a DAG, `/rewind [N]` makes an earlier turn the tip, and the abandoned branch
stays in the file. That is a real record of having changed course, and it is
sometimes exactly what you want.

A re-run is the other choice, and the difference is only about what is kept:

| | Where it lives | The change of mind is |
|---|---|---|
| **Rewind** | one session file, one conversation, an abandoned branch in the tree | **kept** — visible in the file forever |
| **Re-run** | a second session file; the first is a draft you may discard | **not kept** — nothing in session 2 refers to session 1 |

Neither is more honest. They answer different questions, and the tool should
not quietly pick one.

## Two kinds of session, and the marking is in the bytes

*Revised 2026-08-23: this was three kinds, with an "edited" session carrying a
declared layer over an immutable base. That layer was the most format-invasive
thing in this document and it has been cut — see below.*

| | Written by | Provenance class | What it is good for |
|---|---|---|---|
| **Recorded** | the harness, never a person | derived, checkable | evidence — this was in front of the model, this really ran |
| **Authored** | a person — whether written from scratch or by editing a recorded one | declared | a cheap refinement when re-running is not worth the tokens; a tutorial in interview form |

**Editing is supported**, because re-running costs real tokens and real wall
time and a small tidy is not worth an hour. The rule is simply that **a session
a person edited becomes an authored one, whole**: the file changes class, it is
marked as authored, and there is no per-region bookkeeping.

The earlier design kept the harness-written base immutable and expressed the
edit as a declared layer on top, so untouched regions kept their evidentiary
standing. It was more precise and it is not worth the format. The precision it
bought matters rarely — someone who edited a session for readability is not also
trying to prove which of its lines are pristine — and the layer would have been
the one piece of new syntax in this whole design.

**The marking has to be in the bytes.** `provenance-and-standing.md` requires
that derived and declared never render alike, and rendering is not enough here:
a session file gets `cat`'d, diffed on a code host, and pasted into a chat, and
dashed grey survives none of that. An authored session needs a different element
or an attribute visible in raw text.

**A never-run session needs no new concept.** It is a document whose turns never
ran, and `hick weave` already marks never-run blocks as never-run — the state
exists and the UI already draws it. A written tutorial in interview form is
legitimate and useful; what it may never do is look like something that
happened.

## The carry

What moves from one attempt to the next is not the transcript. It is whatever
you learned, distilled into inputs:

- **a better opening prompt**;
- **tests** — the common case, and the reason it is common is that a session's
  most durable output is often the discovery of what "done" meant;
- **an approach you ruled out**, worth carrying so the next attempt does not
  spend a turn rediscovering it.

Tests are one instance and not the mechanism. Building "carry the tests" as a
primitive would be building the example rather than the thing.

**The trap, when tests are what carries.** They were written by an agent that
had already seen one implementation, so they tend to pin that implementation's
incidental choices — call order, internal names, an error string — rather than
the behaviour you wanted. Session 2 is then bound to session 1's arbitrary
decisions while appearing to have chosen freely. The guard is not mechanical:
the carried tests are the ones **you read and kept**, not everything the first
attempt emitted.

In this repository's existing vocabulary the carry is `promote` aimed at a
session's *input* rather than its output.

## The person is the judge

There is no automatic gate on the second session's result, and this is a
decision rather than an omission.

Your understanding of the problem may have moved between the two sessions, and
may move again during the second one. That is the normal case and it is fine.
Requiring session 2 to be provably equivalent to session 1 would freeze the
first attempt's understanding as the specification — which is precisely the
thing you re-ran to escape.

So **equivalence is an instrument, not a gate.** When you *do* want to know
whether two attempts land in the same place, the machinery exists
(`hick_literate::equiv`, `hick test`), and the useful comparison is behavioural
rather than byte-exact: two independent sessions will phrase the same program
differently, and comparing bytes would report a difference on every run.

## Nothing from nowhere

This is the constraint that does real work, and it is a **legibility** property
rather than a provenance one.

A second session that opens with *"use the retry policy, and make sure the 429
case is handled"* is a complete and honest record. To anyone who does not have
the first session, it reads as requirements pulled out of the air. Nothing is
forged; it is simply unexplainable — and since the first session is gitignored
and discardable, the citation that would explain it dangles by construction.

> **A session must be self-contained: followable by a reader who has only the
> repository.**

Two things fall out. Discarding the first attempt becomes safe by construction,
which is what the gitignore already assumes. And a carried requirement whose
only support is a session nobody has becomes a detectable condition, of the
same shape `hick cites` already reports for a dangling citation.

## Why the interview form, for a better reason than taste

An interview teaches more thoroughly than a monologue, and a session is already
an interview — a badly edited one. But the argument for keeping the form is
stronger than pedagogy, and it is the previous section:

**A prescient opening prompt is worse for the reader precisely because it is
prescient.** It concentrates everything you learned into one unexplained
monolith — the maximum ex nihilo. A dialogue distributes the same knowledge
across exchanges where every requirement arrives attached to the question that
provoked it, so the earned knowledge enters the record with its motivation
still on it.

Two consequences worth stating because they simplify the design:

- **The interview is an input technique, not an output format.** You let the
  agent ask two clarifying questions because dialogue reaches clarity faster
  than a prompt trying to preempt every misunderstanding. The readable artifact
  falls out. There is no post-hoc authoring step, and nobody edits a transcript
  into shape.
- **It handles the recursion.** Understanding that evolves *during* session 2
  is ex nihilo relative to session 2's own opening prompt — the same problem one
  level down. In interview form it is not, because the evolution arrives as
  further exchanges in the file rather than as an invisible revision of intent.
  The form does not merely tolerate changing your mind mid-session; it is what
  keeps that legible without yet another re-run.

## Sequence

1. ~~**Name the two acts.**~~ **Built 2026-08-23.** `/rewind` and `/rerun` in
   the dock, each affordance carrying the sentence about what it keeps.
2. ~~**The carry**~~ **Built 2026-08-23.** `hick ingest --from carry <session>` writes the
   opening prompt and leaves the other two slots empty and named — it does not
   distil for you, because the tests worth carrying are the ones you read and
   kept. One thing the design left open is now settled: the carry **invents no
   fourth artifact**. It is an ordinary `.hick` document, written outside
   `sessions/` so it survives the session being discarded.
3. **Edited sessions** — the declared layer over the frozen base, and the
   in-the-bytes marking. This is the one piece of new format.
4. **Self-containment as a check** — a carried requirement with no reachable
   support, reported the way a dangling citation is.
5. **Equivalence as an instrument** — surfaced where you would want it (two
   attempts, side by side), never as a gate.

## Open edges

- **`a-session-is-the-conversation.md` says one conversation, one file.** A
  re-run is a second file rather than a branch of the first, which is
  consistent — but the dock hydrates a document's turn tree from `sessions/`,
  and it will now find several conversations that produced the same result.
  Which one it opens by default, and how it shows that the others exist, is not
  designed here.
- ~~**The carry has no home.**~~ **Answered 2026-08-23** by building it: a
  carry is an ordinary `.hick` document — no new file type, no new store, no
  new gitignore rule — written outside `sessions/` and therefore committed by
  default, which it must be, since a carry whose only support is a gitignored
  session dangles by construction. The resistance the design asked for is what
  produced the answer: the minimum invention was no new artifact at all.
- **How an edited session's base stays immutable in practice.** Nothing stops
  someone opening the file and typing. The layer is a convention until something
  enforces it, and the enforcement most likely lives in the editor rather than
  the format — which is a weaker guarantee than the format-level ones this
  product prefers.
- **Staged sessions and never-run marking were built for a different reason.**
  Reusing that marking is right, and it means a bug in never-run rendering is
  now a provenance bug. Worth knowing before it happens.
