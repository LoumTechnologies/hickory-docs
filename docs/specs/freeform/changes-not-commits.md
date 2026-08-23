# Moving a change to where it belongs: what jj is right about

**Status: an investigation, not a plan.** Asked for on 2026-08-23 alongside
the sharing designs, as something to think about. One small piece of it should
be built now and is named at the end; the large piece should not be built, for
a reason this document exists to record.

**Amended the same day**, after the question was pushed on: the objection to
generated commits was aimed at the wrong boundary. See *the line is
re-derivable history, not emission* below, and `sessions-you-run-again.md` for
what a session that emits one looks like. **Amended again**, same day, by
`expression-and-log.md`: emission is one-way only for *published* commits, and
whether documents replace the repository is settled there — they do not, and a
document that emits history needs git more rather than less.

> "I'm curious about the similarities of this development mode with jujutsu's
> ability to move changes between commits; and how agent sessions can span
> multiple commits; and how maybe the `.hick` file should actually produce
> different commits instead of just producing different files."

## The observation underneath the question is correct

Jujutsu's central move is that **a change is a first-class thing with an
identity, addressable independently of where it currently sits**. `jj squash
--into` takes work you did in the wrong place and puts it in the right one; a
change id survives every rewrite of the commit that carries it; a conflict is
stored rather than blocking.

This repository already does that operation. It just does it on a different
axis, and has never said so.

| | jujutsu | Hickory Docs |
|---|---|---|
| The axis | history: this change belongs in *that revision* | the tangle: these bytes belong in *that source span* |
| The move | `jj squash --into <rev>` | `up::reverse` — an edit saved in a generated file, mapped through `Provenance` onto a span of the `.hick` document |
| Identity of the thing moved | a change id, stable across rewrites | a `Provenance` range, recomputed exactly on every weave |
| Refusal | conflicts are stored, not blocked | a span with no editable origin refuses the save and says why |

`machine-scoped-edits.md` extends this by adding a *destination choice* to the
move — shared span, or a machine-scoped variant beside it — which is exactly
`squash --into` with a different set of targets. So the resemblance the
question notices is real and structural, not a mood.

**What follows from naming it:** the vocabulary should be shared. "Move this
change to…" is a better description of the reverse-edit gesture than "save",
and a destination picker that offers *the shared source span* or *just this
machine* is one instance of a general operation rather than a one-off button.

## Should a `.hick` document produce commits?

Three readings, increasingly ambitious, and they separate cleanly.

### 1. The document produces files, and a commit is what you make of them

What happens today. Git history is authored by a person; the document is one of
the files in it. Nothing to build.

### 2. The document proposes a change — a set of files plus a message and a shape

A weave already knows things a commit wants: which documents changed, which
outputs each produced, and — through `hick:upstream` — which stage each belongs
to. `stages-write-forward.md` establishes that a stage is a real boundary and
that a stage never writes upstream. That is the same shape as a stack of
dependent changes, and jj's stacked workflow is the tool people reach for to
manage exactly that.

So a `hick change` that shows the working tree grouped by stage, with a proposed
message per group, and offers to commit them as a stack, is a **read-only view
that becomes an action**. It adds nothing to the document format, it derives
everything from machinery that exists, and it is wrong in a recoverable way —
you decline the proposal and commit by hand.

This is the reading worth having, and it is not urgent.

### 3. The document *is* the history — commits are generated, not authored

The radical reading, and it fails for the reason
`two-branches-in-one-document.md` gives about branches-as-flags, sharpened:

**A generated commit has no author to blame.** `provenance-and-standing.md`
rests on `git blame` composed through lineage (`agent_lineage.rs`), and
`three-provenances.md` builds the context provenance on `<hick:read
file=… commit=… sha256=…>` — a *real* commit, recorded because the model was
shown a file at it. If history is derived from the document, then the commit a
read points at is an artifact of the last regeneration rather than a record of
what existed, and the strongest provenance in the product degrades to the
weakest.

There is a second, more practical objection. Git history is the one artifact in
this system that other tools read. Generating it means every one of them —
blame, bisect, the history graph in the app, a code host's review UI — is now
looking at something that can be rewritten by a re-run. The product's whole
claim is that the derived artifact stays honest about the source; making the
*history* derived puts the audit trail on the wrong side of that line.

So: **no.** Documents produce files. History is authored.

### Amended 2026-08-23: the line is re-derivable history, not emission

The refusal above is right about the wrong boundary, and the correction
matters because it changes what may be built.

What is fatal is history that can be **re-derived**: a document which, re-run
next year, produces different commits — so the commit a `<hick:read>` points at
becomes an artifact of the last regeneration rather than a record of what
existed. Every objection above is an objection to that.

**Emitting a commit once is not that.** A commit that is produced by a process a
person directed, and then frozen, is authored in every sense blame cares about.
Nothing re-produces it; no tool that reads git history is looking at something
a re-run can change. And the product already has this verb: `hick promote`
turns a session into a clean pipeline, one way, and nothing regenerates the
result afterwards.

So the rule is not "documents do not produce commits". It is:

> **Emission is one-way. A document or a session may produce a commit; nothing
> may re-produce a commit that already exists.**

**Qualified 2026-08-23 by `expression-and-log.md`:** the word that was missing
is *published*. Nothing may re-produce a **published** commit; below that floor
— an unmerged feature branch, computed as `merge-base(HEAD, origin/master)` —
a commit is a draft, and re-emitting the frontier from an edited document is
the useful form of "a document that edits across commits". Publication is what
converts a draft into a record, exactly as it does for a first session in
`sessions-you-run-again.md`.

Under that rule `hick agent` → session → commit is clean, and
`sessions-you-run-again.md` is the design for the session half of it. Three
classes of edit still have to be refused, because each would make an emitted
commit lie about itself:

1. **Reattribution** — an edit that lets model-written bytes read as
   human-written. `provenance-and-standing.md` already forbids the claim; this
   would have to forbid the edit.
2. **Retroactive context** — `<hick:read file=… commit=… sha256=…>` is a record
   of what was in front of the model. It is a fact, not a draft, and an edited
   session must express changes as a declared layer over it rather than as a
   rewrite of it.
3. **Result drift** — an edit that changes what the session produced while
   presenting as a change in how it reads. This one is mechanically checkable,
   which is what the equivalence machinery is for.

## Agent sessions spanning commits

This is the part where something is missing today, and it is small.

A session file is one conversation with a turn tree
(`docs/guarantees/agent/a-session-is-the-conversation.md`). A turn reads files
and writes lines. The session already records, per read, the file's **commit**
and content hash — the harness writes `<hick:read file=… commit=… sha256=…
lines=…/>` — and, per write, `<hick:wrote file=… lines=… hashes=…/>`.

The asymmetry is the gap: **a read knows which commit it saw; a write does not
know which commit it became.** So a session can answer *what was in front of
the model* and cannot answer *which commits contain this conversation's work* —
which is the question you actually have three weeks later, and the question
"can agent sessions span multiple commits" is really asking.

Closing it is one field and one hook:

- when a commit lands in a repository the session's document is in, stamp the
  turns whose written hashlines are in that commit — `<hick:landed turn=…
  commit=…/>`, written by the harness, never by the model, exactly as `read`
  and `wrote` are;
- `hick context` then answers both directions: this line came from that turn,
  and that turn's work is in these commits.

What this makes possible without committing to any of it: *revert this turn*
(the commits are known), *what did this conversation cost* across a week of
commits, and a session that is legible after the branch it was on is merged.
What it deliberately does not do is make a turn *be* a commit — a turn is a
unit of conversation and a commit is a unit of intent, they align often and not
always, and a design that forces them to align gets both wrong.

Two honest limits, both inherited from `three-provenances.md`: hashline anchors
mean a later human edit to the same line stops the write resolving, correctly;
and `sessions/` is gitignored by `hick init`, so the record lives on the machine
that ran the turn. That second one collides directly with
`one-engineer-many-machines.md` — a turn driven from the laptop onto the sealed
box leaves its session on the sealed box — and `hick:landed` is what would let
those sessions be reassembled from the commits when they are eventually
gathered.

## Interoperating with jujutsu

Worth one paragraph, because the honest answer is "do nothing, and do not break
it". `jj` colocates with a git repository; an engineer who uses it gets the
working copy as a change, `jj squash`, and stored conflicts, and nothing in
this product needs to know. What *would* break it is generating history
(reading 3) or reimplementing change movement over git plumbing. Neither is
proposed. The one thing to check when the reverse-edit path grows the
destination picker is that it never assumes the working copy is clean in the
git sense, because under jj it routinely is not.

## What I would actually do

1. **`<hick:landed turn=… commit=…/>`.** Small, harness-written, symmetric with
   `read`'s existing `commit=`, and it closes the one real gap. Do this when
   the sharing designs make sessions cross machines, because that is when it
   starts hurting.
2. **Name the move.** Call the reverse edit what it is — moving a change to
   where it belongs — in the UI and in the docs, and let
   `machine-scoped-edits.md`'s "just here" be one destination among a general
   set.
3. **`hick change`, read-only first.** Group the working tree by stage, propose
   messages, commit as a stack only once the grouping has proved itself.
4. **Do not generate *re-derivable* history**, per the amendment above.
   One-way emission is allowed and is how a session produces a commit; a
   history that a re-run can rewrite is not, and no provenance story that
   survives it has been found.
