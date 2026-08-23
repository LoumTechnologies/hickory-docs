# The document is an expression, the repository is the log

*Status: design of record for what happens when documents emit history.
Adopted 2026-08-23. **Nothing here is built.** It answers the question
`changes-not-commits.md` opened and its first amendment left half-open — if a
session may emit a commit, and a document may span machines, may a document
span commits and repositories too, and is git then redundant? Amends that
document a second time, with the publication boundary. Depends on
`stages-write-forward.md` for what a stage is and on
`sessions-you-run-again.md` for the draft-versus-record principle it reuses.*

> "What if we have hick files for creating the entire codebase and then we
> change an already-implemented requirement? We should not rewrite history
> because it has already been merged to master. But we don't want the document
> forever having to correct itself."

## Places and times

Three axes look alike and are not.

| Axis | What it is | Spanning it is |
|---|---|---|
| machine, worktree | a **place** — several exist at once | coordination |
| repository | a **place** | coordination, with a caveat below |
| commit | a **time** — the past | rewriting |

`machine-scoped-edits.md` routes an edit to one of several readings that all
exist right now. Applying the same mechanism to commits looks like a small step
and is not one: an edit routed into the past is not a live edit to a concurrent
thing, it is a change to the record of what was true. That is the property git
is valuable for making hard, and a design that makes it cheap has taken
something away rather than added something.

So the axis is allowed, bounded: **a document may span time only where time is
still mutable.** Git names that boundary by publication, and jj names it
explicitly with `immutable_heads()` and refuses to rewrite past it. Nothing new
has to be invented, only respected.

## Git is not replaced, and the reason is a category difference

The temptation is arithmetic: a document produces a set of files, a commit is a
set of files, so one could stand in for the other.

**A document is an expression. A repository is the log of its values.** Evaluate
a document and you get a set of files — what is true *now*. A repository stores
the sequence of those values, immutably, with parentage and attribution. An
expression cannot replace a log: change it and the previous value is gone unless
something wrote it down.

Which settles the version-control question in the strong direction. It is not
merely that the document *may* be committed — **the more history a document
emits, the more it needs git**, because the only way to know why last March's
commit says what it says is to have last March's document.

The recursion this seems to open terminates in one step. A document is a text
file; git versions text files; there is no second level.

## The loop is a spiral

The document lives inside the thing it produces — the same shape as a
repository containing its own build script. It resolves by ordering:

> **Each emitted commit contains the document version that emitted it.**

Commit N holds document N. Commit N+1 holds document N+1. No commit's content
depends on a future document, so the circle is a spiral, and going back in time
gets *more* than git alone gives: the code, the document that generated it, and
the lineage between them as it stood.

## Nothing in the document is ever frozen

This is the answer to the worry above, and it is a relief rather than a
mechanism.

The fear is that because commit 12 must never change, the part of the document
that generated commit 12 must never change either — so the document can only
ever append corrections, growing into a chain of *and then we changed this*.

It does not follow, because **the record of what generated commit 12 is inside
commit 12**, not in the document on your disk. The live document is free. It has
one job and it is not archival:

> **The document describes the present. Git holds the past.**

Changing an implemented requirement means editing the document to state the new
requirement plainly, as though it had always said so. The diff of the document
*is* the record of the change; the emitted commit carries a frozen copy; the old
reading is recovered by checking out the old commit, which holds the old code
and the old document together.

This is exactly how ordinary source code already works. Nobody leaves
corrections in a function to preserve history — they change the function, and
history remembers. A document is no different, and the instinct that it is
comes from a model worth naming so it can be refused:

| | The document is… | Reading it means | Failure |
|---|---|---|---|
| **Migration chain** | an append-only script of everything that ever happened | replaying every edit | grows without bound, and the current truth is nowhere stated |
| **Present description** | what the codebase is now | reading it | none — but it is not a record, and must not be mistaken for one |

The migration chain is the same category error as before: an attempt to make the
expression *be* the log. It is also the failure mode of `patch`/quilt series
files, which `scaffolded-files-and-derived-edits.md` already rejects for a
neighbouring reason.

**The discriminator, when it is genuinely unclear:** does the *current codebase*
contain the thing? If yes, the document says it — including both sides of a
deprecation, because both are really there. If no, git holds it.

**The honest exception:** append-only is right where there is external state
that cannot be re-derived. Database migrations are the real case — the deployed
database holds a value no document can regenerate, so the migration list is a
log and legitimately accumulates. A document describing migrations is describing
a log, which is different from being one.

## Verification does not reach backwards

Worth stating because the worry above usually hides a second one: changing a
requirement does not make history fail.

`hick test` compares the current document against the current tree. Drift is a
property of the working tree, never of the past. Commit 12 was verified when it
was made and its verification is recorded in it; a requirement changing today
cannot retroactively fail it, and nothing in this design should ever be able to
report that it did.

## The frontier

The feature-branch case is the other half of the question, and it is the same
boundary read from the other side.

- **The floor is publication**, computed and not felt: `merge-base(HEAD,
  origin/master)`, or whatever the repository's published ref is. Below it,
  commits are records.
- **Above it is the frontier**, and the frontier is *derived*. Edit the
  document, re-emit, and the frontier is replaced — commits rebuilt, not
  patched. This is the useful reading of "a document that edits across
  commits", and it is `jj squash --into` with the destinations computed instead
  of chosen.
- **Commit boundaries come from stages.** `stages-write-forward.md` already
  establishes the stage as a real boundary that never writes upstream, which is
  the same shape as a stack of dependent changes. So one stage is one commit by
  default, the shape is structural rather than an artifact of the order you
  typed, and re-emission is deterministic given the document.
- **Merging moves the floor**, and yesterday's rewritable commits become
  permanent. Nothing about the document changes at that moment — which is the
  point, and the reason no document region is ever frozen. **The
  mutable/immutable boundary lives in the commit graph, never in the document
  text.**

So `changes-not-commits.md`'s rule needs one qualifier, and this document is the
second amendment to it:

> **Nothing may re-produce a *published* commit.** Publication is what makes
> emission one-way. Before it, a commit is a draft.

Which is the principle `sessions-you-run-again.md` already applies one artifact
down: the first session is a draft you may discard, a frontier commit is a draft
you may re-emit, and in both cases publication — pushing, or someone else
holding it — is what converts a draft into a record. Two guards follow, because
cheap emission plus cheap re-running makes accidental rewriting cheap:
**emission appends and never amends below the floor**, and a refusal names the
published commit and where the floor is.

## Where the change gets explained

The document says what is true now, so a reader six months later sees the
current requirement and not that it changed. That is correct for the document
and insufficient for the reader, and the missing half already has a home: the
commit — its message, and the session that emitted it. *We tried X and moved to
Y, because…* belongs there. Document is present state; commit is the change and
its reason.

## Repositories: read across, write local

The remaining axis, and the one where the analogy to machines genuinely fails.

A document writing into two repositories means two commits that cannot be
atomic, so partial failure is a real state someone has to design for. Worse, the
spiral rule breaks: either both repositories carry the document and it drifts,
or one does and the other's history references an explanation its readers do not
have — the ex nihilo failure from `sessions-you-run-again.md`, at the scale of a
repository.

So: **read across, write local.** A document may read other repositories, pinned
by commit hash, and writes into one. That
is `stages-write-forward.md`'s asymmetry — read anything upstream, write nothing
there — generalized one level out.

## Sequence

1. **The floor**, as a computed fact surfaced in the UI: which commits on this
   branch are still drafts. Useful immediately, and it needs no emission at all.
2. **Each emitted commit carries its document version** — a property to build
   into emission from the first day, because retrofitting it means a generation
   of commits that cannot explain themselves.
3. **Stage-shaped emission**, one stage one commit, read-only first: show the
   commits a re-emission *would* produce.
4. **Frontier re-emission**, with the refusal below the floor.
5. **Cross-repository reads**, pinned by hash.

## Open edges

- ~~**Provenance has nothing to say across versions.**~~ **Answered
  2026-08-23** by `provenance-across-versions.md`: the data is not missing, the
  *identity* is. `(commit, path, line)` addresses published bytes exactly and
  replay recomputes exact lineage at any commit; what neither can do is
  **correlate** two versions. That is what a recorded correspondence does —
  written at a moment when both sides were in hand, or, where nothing recorded
  one, guessed by an agent and confirmed by a person. There are deliberately no
  element ids. Drawn only when a setting is on.
- **What a re-emission does to a branch someone already fetched.** The floor is
  computed from a published ref, which assumes the only way a commit escapes is
  a push. A peer that attached over the fleet channel and pulled has a copy the
  floor does not know about.
- **Stage-shaped commits may be the wrong grain.** One stage is often several
  commits' worth of intent, and sometimes three stages are one. The override
  (choose the destination, as jj does) is named here and not designed.
- **Whether the frontier should be visible as the git history tab or as the
  document.** Both are true and drawing them as one surface is the collapse
  `changes-not-commits.md` warns about. Linked, not merged — but which one is
  the primary view of a frontier commit is undecided.
