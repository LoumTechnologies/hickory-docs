# The merged view: one tab, several branches, and the merge already done

*Status: design of record for editing across branches, worktrees and machines.
Adopted 2026-08-23. **Nothing here is built.** **Supersedes**
`machine-scoped-edits.md` on its central mechanism — machine names in committed
conditionals — which this replaces. That document remains accurate and worth
reading on two things it settled: why the scoped text lives in the file on every
machine rather than in a per-machine sidecar (its Reading A / Reading B
analysis, whose conclusion this design reaches by another route), and how
`hick_literate::equiv` proves a restructure leaves a given reading alone. It
also closes the question `two-branches-in-one-document.md` left open, without
adopting the idea that document called a trap.*

> "What if we let you edit multiple files across machines and branches in the
> same hick editor tab by using something like these `hick:when` style elements
> — but the actual text that includes them doesn't exist on disk anywhere?"

## `hick:when` as syntax, not as storage

Open one tab. It shows a file as it exists in four places at once — two
worktrees on different branches, a checkout on the desktop, a checkout on the
Windows box — unified into a single editable surface. Regions all four agree on
appear once. Regions that differ appear as variants, in the shape
`<hick:when>` already has.

**None of that text exists on disk.** There is no file containing those
conditionals, nothing to commit, and nothing to merge. The view is synthesized
on open from the sources, and every edit routes back to the source it came from.

### It is a lens, not a document

This is the framing rule the design depends on, and getting it wrong reverses
the product's central claim. Hickory Docs says the document is the source and
the generated file is a working surface onto it. A synthetic document assembled
*from* files points the other way — and that only becomes an inversion if the
thing pretends to be a document you could save.

So it must not. No save-to-disk path, no `.hick` extension, no place in the
folder tree. It is the same category as a diff view or a merge view, and nobody
mistakes `git diff` output for a source artifact.

## The view is the merge, held live

The property worth building this for is not convenience.

**Every shared region is agreed by construction**, because editing it writes to
every target at once. Those regions cannot conflict later — there is nothing to
reconcile, because they never diverged. The only differences that survive to
merge time are the ones somebody deliberately scoped.

And a divergence that already existed when the view opened does not disappear;
it *appears*, as a variant, where it can be resolved by promoting one side to
shared. That is a merge conflict resolution performed early, at leisure, instead
of at merge time under pressure.

So the view is a merge that has not been committed yet — live, editable, and
still separable back into branches, where an ordinary merge is a destructive
one-way event.

### Result and order are different claims

**The result** is what the paragraph above describes, and it holds whatever the
branches are to each other.

**The order** is a further, optional thing, and only applies if the branches are
a **stack** rather than **peers**:

| | What the view is | What "shared" means | Order |
|---|---|---|---|
| **Peers** | N symmetric sources | agreed by all | none — the view removes the question |
| **Stack** | a chain, each landing on the last | agreed by everything at or below | the view encodes the sequence they land in |

Both are coherent and they need different write rules, so **this has to be
decided before the edit routing is written.** In a stack, "shared" is ambiguous —
shared with everything, or inherited from the branch below? — and answering it
late means rewriting the routing.

## Why this matters more here than it would elsewhere

`two-branches-in-one-document.md` established that merge pain is unusually
expensive in this product: a `.hick` document is one file that generates many,
so two branches editing two *different* generated files still collide in the
document that writes both.

That pain was the strongest argument for the radical idea in that document —
stop branching, express difference as feature flags — which it judged a trap for
four reasons. The merged view takes the benefit and pays none of them:

| The objection to branches-as-flags | Why it does not apply here |
|---|---|
| Conditionals compose multiplicatively | They are ephemeral and live in one tab; nothing composes in any file |
| A conditional never gets deleted | There is no conditional on disk to delete — no census, no prune |
| Git already does branches well | The branches stay real branches; git is not replaced by anything |
| It breaks provenance — a flag has no commit to blame | Every edit lands in a real file on a real branch, so blame, history and lineage all keep working |

That effectively closes the investigation — not by adopting branches-as-flags,
but by locating where its value actually was.

## The division of labour with committed conditionals

The view does not make committed `<hick:when>` obsolete, and the line between
them is about **whether the difference deserves to be reviewed**.

| | Use for | Lives |
|---|---|---|
| **The merged view** | transient or local differences — this box, this branch, this checkout | nowhere; synthesized on open |
| **Committed `<hick:when>`** | permanent variants worth declaring — a runbook with two deployment targets, a tutorial with two paths | in the document, in git, reviewed like anything else |

**The loss to be honest about:** a difference held only in the view is invisible
in the repository. Two files simply differ, with nothing recorded saying the
difference is deliberate, so drift and intent look identical. That is acceptable
for local and transient differences — which is most of them — and it is exactly
why the permanent ones stay committed.

### And when a variant is committed, it names a property, not a machine

`machine-scoped-edits.md` keyed committed conditionals on `machine:<name>`,
which put a personal machine inventory into a shared repository. That is the
part this document supersedes, and the replacement is a rule rather than a
rename:

> **The distinction lives in the document; which machine matches lives on the
> machine.**

A document says `<hick:when test="has:cuda">` — meaningful to any reader, on any
team, indefinitely. A local, untracked fact set says *this machine has CUDA*.
Nothing personal is committed, and fleet identity
(`one-engineer-many-machines.md`) goes back to being purely a runtime and
networking concern that never reaches the repository.

Detect where it is trivially reliable (`os:`, `arch:`); **declare locally
otherwise.** Auto-detecting a GPU sounds helpful and produces surprises; a line
in an untracked local config is predictable. The invariant from the superseded
document survives unchanged and matters more here: **CI weaves with no facts at
all, so the shared reading must stand alone and a variant may only add.**

## Three hard parts

**N-way diff is a real step up.** `hick-merge` does three-way. Aligning four
sources so shared regions genuinely correspond — rather than fusing two regions
that merely look alike — is the main technical risk in the whole idea, and a bad
alignment routes an edit silently into the wrong file. This is where a
first version should be deliberately conservative: fewer regions declared
shared, more shown as variants, because the failure of over-sharing is an edit
in the wrong place and the failure of under-sharing is a little redundant
typing.

**A multi-target edit is not atomic.** You type in a shared region; three
machines accept the write and the fourth is asleep. Refusing the edit is
intolerable and pretending it landed is worse, so the write queues and **the tab
shows per-target status** rather than implying success. Designing this up front
is the difference between a feature that feels finished and one that does not.

**"Across branches" means across worktrees.** A branch that is not checked out
cannot be written to without going behind the working tree into the object
database — which bypasses hooks, surprises people, and produces commits nobody
watched. So each branch in the view is a real worktree on disk. That grounds the
feature in something that already exists (`hick-term`'s `add_worktree`, the
terminal's `open_in_worktree`) and connects it to multiple-worktree support
directly.

## Three things it forces

**A shared edit writes N branches in one keystroke**, which is a very sharp
tool. The "just here" gesture is the only guard, and **undo across N worktrees
has to be designed rather than assumed** — a half-applied edit that is then
undone in three of four places is a state somebody will reach on the first day.

**Some targets must not converge.** A long-lived release branch in the view
would silently receive shared edits. Targets need a read-only mode, and it
should be the **default** for anything not explicitly opened for writing.

**The gesture survives and gets simpler.** "Just here" no longer wraps text into
a conditional in a file; it means *write this to one target only*, which is both
easier to implement and easier to explain.

## What the view remembers, and where

The view is synthesized on open, so by default it remembers nothing: reopen it
tomorrow and the shared/variant partition is re-derived from whatever the
sources say then. A region that was shared yesterday can be a variant today with
nothing recording that it changed — and worse, a difference you made
deliberately is indistinguishable from one that drifted in.

Three things could be kept, and only one earns it:

| | What it is | Keep it? |
|---|---|---|
| **Intent state** | which divergences are deliberate | **yes** — small, bounded, and the thing whose absence is the problem |
| **An edit log** | every keystroke routed through the view | no — grows without bound, answers questions nobody asks |
| **An undo stack** | enough to reverse a shared write across targets | yes, bounded — it is needed anyway |

So this is **state**, not history, and it is uncommitted.

### The precedent already exists

`hickory-workspace` persists drafts — unsaved buffer contents — plus UI state,
in the platform's per-user state directory, outside the project, keyed by the
project's canonical path. And it records **the base bytes with every draft**,
precisely so that reopening is a three-way merge rather than a two-way *these
differ, you sort it out*. That is this problem, already solved, including the
subtle half.

### The rule that makes it safe

Stored view state is the kind of thing that becomes a correctness bug rather
than a UI wrinkle. If the view trusts its stored partition after the files move
underneath — a pull, a rebase, an edit in vim — it will show *shared* for
regions that have diverged, and route the next edit to N targets believing they
agree.

> **The diff decides what *is*. The stored state decides what you *meant*.**

The partition is always recomputed from the sources on open. Stored state may
only contribute intent — *this divergence is deliberate, stop offering to
reconcile it* — and may **never** assert agreement. It is the same rule
`provenance-across-versions.md` applies to its journal: derivable state is never
authoritative.

### Where it lives: a local, unpushed ref

Two homes are plausible and the second fits this feature better.

**Per-user state**, following `hickory-workspace`'s reasoning that git
eventually commits in-project state through a `git add -A` on a machine where
`hick init` never ran. Safe — but keyed by path, so it is *per worktree*, which
is awkward for a feature whose whole job is spanning worktrees.

**A local, unpushed ref** — `refs/hickory/view-state`. Refs are not touched by
`git add`, so the accidental-commit risk goes away, and **worktrees of one
repository share refs and objects**, so one view's state is visible from every
worktree in it. That is exactly the span this feature needs.

Neither spans machines. Nothing local does, and that is the honest limit: a view
opened from the desktop and the same view opened from the laptop do not share
what you meant.

### What it still cannot do

Uncommitted state gives *you* continuity of intent and gives nobody else
anything. A reviewer still sees two files that differ, with nothing recorded
saying the difference is deliberate. **So the boundary above does not move** —
permanent, shareable variants are still committed as `<hick:when>`. This makes
the transient side less forgetful, not more reviewable.

### Not the correspondence journal

Both are span-to-span correspondences with a labelled kind, so sharing an
artifact is tempting. The reason not to is **lifetime**: a cross-*version*
correspondence is permanent once recorded, while a cross-*target* one is
invalidated the instant either side changes. Putting a perishable record inside
an append-only permanent one leaves most of the journal stale, and forces the
never-authoritative rule to be enforced per entry rather than per artifact.

## What it does not replace

**The merge tab** (`provenance-across-versions.md`) does a different job.

| | Job |
|---|---|
| **The merged view** | *planned* co-development — you own all the branches and are steering them toward each other. Prevents conflicts |
| **The merge tab** | *unplanned* merges — someone else's work arrives and you reconcile after the fact. Resolves the ones that happen anyway |

One nice fallout: each region of the view maps to `(source, span)`, which is
exactly the lineage relation. So ribbons work on a synthetic document, and they
show something genuinely new — a ribbon terminating in a file on another
machine.

## Sequence

1. **Two local worktrees, read-only.** The N-way diff, the shared/variant
   rendering, no writing at all. This is where the alignment risk is proved or
   disproved, and it is useful on its own as a comparison view.
2. **Writing, one target at a time** — "just here" only, with read-only the
   default for every other target. Nothing can be broken in four places yet.
3. **Shared writes**, with per-target status and undo across targets.
4. **Peers or stack**, decided, and the routing written to match.
5. **Remote targets**, over the fleet channel's `edit` grant, with queueing for
   a machine that is asleep.
6. **Intent state**, in the local ref, once shared writes exist and there is
   something worth remembering. Not before — a view that remembers nothing is
   correct, just forgetful.
7. **Capability facts** for the committed case — `has:`, declared locally,
   detected where reliable.

## Open edges

- **Peers versus stack is undecided**, and it is the decision everything else
  waits on.
- **A fresh clone has no local fact set**, so it weaves the unscoped reading
  until someone declares one. Same shape as the merge-driver onboarding step,
  and it should be checked and reported at project open rather than discovered
  when a cell runs slowly.
- ~~**The view has no history of its own.**~~ **Answered 2026-08-23**: it keeps
  *intent state*, not history — uncommitted, in a local unpushed ref, never
  authoritative over the diff. See above. What remains open is that the state is
  per-machine, so the same view opened from two machines does not share what you
  meant.
- **N is not bounded.** Four targets is legible; twelve is not, and nothing here
  says what happens when someone opens a view over every branch in the
  repository.
