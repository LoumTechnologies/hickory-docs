# Machine-scoped edits: a feature flag whose owner is a box

*Status: **superseded 2026-08-23** by `the-merged-view.md` on its central
mechanism. Machine names in committed conditionals put a personal machine
inventory into a shared repository, and the replacement is a rule: **the
distinction lives in the document, and which machine matches lives on the
machine** — a committed variant names a property (`has:cuda`, `os:windows`),
while transient per-box and per-branch differences are edited in a synthesized
view that exists on disk nowhere. Two things here survive and are why this
document is kept: the **Reading A / Reading B** analysis below, whose conclusion
the successor reaches by another route, and **proving a refactor leaves a
reading alone**, which applies unchanged to capability facts. The invariant —
CI weaves with no facts, so the shared reading must stand alone — survives too.
Read the rest as the reasoning that led there, not as the design.*

*Originally: design of record for machine and worktree scoping. Adopted
2026-08-23. **Nothing here is built.** It resolves the parts of
`two-branches-in-one-document.md` that were left as "worth doing", takes the
part that document called a trap and explains why this reading escapes it, and
depends on `one-engineer-many-machines.md` for what a machine's identity is.
The backwards half — an edit in an output landing in a scoped region of the
source — is the mechanism `scaffolded-files-and-derived-edits.md` asked for,
pointed at a different problem.*

> "There should be an implicitly-added feature flag for *is this only on the
> Windows dev machine?*, and any edits within such a feature flag would only be
> persisted to disk on the Windows dev machine. But any edits outside such a
> flag would be applied to all connected sessions."

## What already exists

`hick-feature` is a real feature system — named features, `requires`,
`conflicts_with`, exclusive groups, validated into a DAG. `<hick:when
test="…">` gates content on them through `hick-condition`'s boolean grammar,
and which features are on comes from outside the document (`--features`,
`scan_and_register_features` in `crates/hick-literate/src/lib.rs`). A document
can already hold two versions of a file and weave either.

What it cannot do is know *where it is standing*. Every feature is declared by
an author and chosen by hand.

## The one design decision this document makes

The request has two readings, and they are not close. Choosing between them is
the whole architecture, so it is made first, in the open.

**Reading A — the text is everywhere, the effect is local.** The
`<hick:when test="machine:windows-dev">` block and its content live in the
document, in git, on every machine. What is machine-local is the *weave*: only
the Windows box tangles those bytes into `appsettings.json`. Edits inside the
region commit and sync like any other text.

**Reading B — the text itself only exists on that machine.** The region's
content is stored in a per-machine sidecar and never enters the shared file.

**Reading A is what this design adopts**, and B is refused. The reasons are the
ones `two-branches-in-one-document.md` already worked out, one step further:

- B makes the document a different file on every machine, so every sync is a
  conflict in the one file that generates all the others — the merge pain that
  document names as the strongest argument *for* the idea, re-created by the
  implementation of it.
- B has no `git blame`, so a machine-scoped line has no author and no commit.
  `provenance-and-standing.md` rests on blame composed through lineage; a
  region outside history is a hole in the property this product exists for.
- A already delivers what was actually asked for. The thing the engineer wants
  is that **the Windows-only lines never appear in the Linux checkout's
  generated files** — not that they be unreadable from Linux. A delivers that
  exactly, for free, from machinery that shipped.

What A does *not* do is hide anything, and that limit must be said plainly
rather than discovered: **a machine-scoped region is in the repository and
anyone with the repository can read it.** Where the content genuinely must not
be shared — a key, an absolute path to somebody's home directory — a feature
flag is the wrong tool and `hick-secrets` or the environment is the right one.
A design that let people believe otherwise would be worse than not having it.

## The implicit facts

Some features are not declared by an author; they are *true of the place the
weave is happening*. They live in a reserved namespace with a colon, which
`hick-condition` already tokenises and which no author-declared feature may
use, so a reader can always tell an asserted feature from an observed one.

| Fact | Value | From |
|---|---|---|
| `machine:<name>` | the enrolled machine's name | the fleet identity in `one-engineer-many-machines.md` — a keypair, not a hostname |
| `os:windows` / `os:macos` / `os:linux` | one of three | the build target |
| `branch:<name>` | the current branch | `git rev-parse --abbrev-ref HEAD` |

*Revised 2026-08-23: `arch:` and `worktree:` were axes here and are cut.
`arch:` is almost never what anyone means, and `worktree:` keyed on a directory
name someone will rename — a defect this document had already flagged. Three
axes carry the real cases.*

Each axis is an **exclusive group** in the sense `hick-feature` already
validates, so `machine:a and machine:b` is a registry error rather than a
condition that is merely never true. `machine:` is the interesting one and
`os:` is the one people will reach for most: *only on the Windows dev machine*
is usually really *only on Windows*, and the narrower flag should be the one
you have to ask for.

`branch:` is the branch→feature-set mapping `two-branches-in-one-document.md`
recommends, arriving as a fact rather than as a config table. The config table
in that document still stands for mapping a branch to *author-declared*
features; this is the raw fact underneath it.

## The invariant that keeps it honest

**The shared reading must stand alone. A machine-scoped region may only add.**

Concretely:

- `hick test` and `hick weave` in CI run with **no machine facts set at all**.
  Every `machine:`, `worktree:` and `branch:` fact is false; every scoped region
  is off. The document must still weave, still pass its expectations, and still
  produce a working set of files.
- A `<hick:when test="machine:…">` may therefore not be the only definition of
  anything load-bearing. If the Linux checkout needs a value the Windows box
  overrides, the shared reading holds the default and the scoped region
  replaces it — never "Windows has it and nobody else does".
- The verification output names the fact set it used, so a green run always
  says which reading was checked.

This is the answer to the risk `two-branches-in-one-document.md` flags — that a
document weaving differently per branch makes `hick test` results
branch-dependent in a way that surprises CI. The machine axis makes that risk
much worse (a reading nobody else can even reproduce), so the axis is
constrained rather than merely documented: the canonical reading is the one
with no machine.

## The census, and why these flags can die

The strongest objection to flags in that document is that **a conditional never
gets deleted**: a branch merges and disappears, a flag stays until somebody
decides it is safe to remove, and nobody ever decides that.

Machine flags escape this, and it is the reason the mechanism is worth having
at all: **their owner is a physical object with a retirement date.** A feature
called `new-auth` has no natural end. A region that says `machine:old-laptop`
is provably dead the day that laptop leaves the fleet, and the fleet knows the
day it leaves.

So the mechanism ships with the thing flag systems never have:

```
$ hick machines
  machine:windows-dev   6 regions in 3 documents      enrolled
  machine:mac-mini      1 region  in 1 document       enrolled
  machine:old-laptop    4 regions in 2 documents      NOT ENROLLED — retired 2026-05-02
  os:windows           11 regions in 5 documents      —
$ hick machines --prune old-laptop
```

`--prune` does not delete silently. It shows each region, and offers the two
honest resolutions — **inline it** (the machine is gone, the change was always
right, promote it to the shared reading) or **remove it** (it was a local
workaround and it dies with the box) — as a diff the engineer approves. A
region whose machine is not enrolled is a warning in `hick check`, not an
error, because a machine can be unplugged for a month.

## The backwards half: an edit in a generated file

This is the part that is new machinery rather than presentation over existing
machinery.

Today, saving a generated file diffs it against the bytes the loop wrote
(`up::reverse::diff_to_edits`), maps each edit through `Provenance` onto a span
in the `.hick` source, and either applies it or refuses with a reason
(`hickory_lineage::map_edits`). The refusal case is generated text with no
editable origin.

Machine scoping adds one choice at the moment of the save, and it is a choice,
never an inference:

| What you do | Where the edit lands |
|---|---|
| Save (default) | the source span, shared, exactly as today — every machine gets it |
| Save, marked **just here** | the source span, wrapped: the original text becomes the unscoped reading, the edit becomes a `<hick:when test="machine:this-one">` variant beside it |

**The default is shared and the scoping is a gesture.** A silent machine-scope
is a silent divergence, and a divergence you did not ask for is the exact
failure mode that makes flag systems hated. In the app the gesture is a
lineage-gutter action on the edited region; from `hick up` it is a marker line
the loop understands, or nothing — the headless loop may reasonably only ever
do the shared thing and say so.

Four rules the transformation has to obey, each learned from
`scaffolded-files-and-derived-edits.md`:

1. **It is a source transformation, and it is reviewable.** What lands is a
   diff of the document, which the engineer reads. Rewriting somebody's
   document into a conditional is not something to do invisibly.
2. **It never nests.** If the span is already inside a region scoped to *this*
   machine, the edit lands inside it and nothing is wrapped.
3. **Editing another machine's region is refused**, and the refusal has an
   unusually precise reason available: those bytes were not in the file you
   edited. The output you saved was woven without them, so an edit to it cannot
   be about them. The message says so and offers the two things you might have
   meant — scope this to your machine instead, or edit the document directly.
4. **An unresolvable wrap fails the save**, restoring the file, as a refused
   save already does. A generated program silently missing a change is worse
   than a save that did not happen.

The pleasing part is that this is not a new provenance story. The scoped bytes
carry `SourceOrigin::Literal` spans like any other document text, the ribbons
draw them like any other, and `hick lineage` on the Windows box shows the
region live while the same command on the laptop shows it absent — which is the
truth, drawn.

## Proving a refactor leaves a machine alone

An agent proposes a restructure. The question you actually have is not "is this
tidier" but **"does the Windows box still get the same files?"** — and that
question needs no new proof machinery, which was not obvious until the existing
one was read.

`hick_literate::equiv` already enumerates the **power set of feature
combinations** and compares the outputs of two documents across all of them
(`generate_combinations`, `check_equivalence`,
`crates/hick-literate/src/equiv.rs`). The app's refactor badge is the weaker
one-reading version of the same check: `POST /docs/:id/refactor/begin` pins the
current woven outputs in process memory and `…/refactor/status` re-weaves the
live text against them with the same `compare_outputs`
(`crates/hickory-cli/src/serve/refactor.rs`).

So once the facts above are features, "provably unchanged for
`machine:windows-dev` and `branch:master`, changed for `machine:mac-mini`" is a
result the existing checker can produce, and it is the right thing to put on a
proposed refactor.

Two things have to be true for the answer to be worth trusting:

- **The exclusive groups are what make it affordable.** One machine at a time,
  one OS, one branch, so the readings are not `2^n` — they are `n+1` per axis,
  and the machine axis costs a weave per machine rather than a weave per
  subset.
- **A truncated proof must say it was truncated.** `generate_combinations`
  stops at `max_combinations` by taking the first subsets in order, which for
  machine facts would silently skip readings. A badge that says "unchanged"
  while having checked eleven of fourteen machines is worse than one that
  refuses to answer; it must name what it did not check.

This also changes the shape of the combinatorial worry below. Having many
readings is a **comprehension** problem — nobody can hold fourteen variants in
their head — and not a **verification** one, because nobody has to: the checker
reads them and reports the ones that moved.

## What the editor must show

Dimming the inactive side of a `when`, which `two-branches-in-one-document.md`
already recommends, stops being a nicety here and becomes required. A region
that is *live because of where you are standing* is invisible in a plain text
editor, and the failure it causes — editing a Windows-only block on the laptop
and wondering why nothing happened — is guaranteed to happen on day one.

- Inactive regions dimmed, the way `#if 0` reads in a C editor.
- The active fact set beside the branch in the status bar: `master ·
  windows-dev · os:windows`.
- A preview toggle that re-weaves under a different fact set without changing
  the document — including **as another machine**, which is how you check what
  the desktop is about to get before you push.
- In the fleet pane (`one-engineer-many-machines.md`), a document open in two
  attached sessions shows each machine's own reading. One document, two
  renderings, both correct, and the strip says which box you are typing into.

## Sequence

1. **Implicit facts, read-only.** `os:`, `arch:`, `worktree:`, `branch:` as
   observed features; the fact set in the status bar and in verification
   output; CI weaving with none of them. Useful immediately, no new syntax, and
   it needs nothing from the fleet.
2. **The invariant enforced** — `hick check` failing a document whose shared
   reading does not stand alone.
3. **`machine:`**, once machine identity exists.
4. **The census** — `hick machines`, and the unenrolled warning.
5. **The backwards half** — "just here" on a save, the wrap transformation, the
   three refusals.
6. **`--prune`**, with its inline-or-remove diff.

Steps 1–2 are worth doing whether or not anything after them happens, which is
the test of the decomposition.

## Open edges

- **How many facts is too many.** Conditionals compose multiplicatively and
  three axes is already eight readings on paper. The exclusive groups bound
  it — one machine, one OS, one branch at a time — so the *live* readings are
  few, but a document with regions for four machines is still hard to *read*.
  Verification scales (above); comprehension does not, and the census plus the
  dimming are the only pressure valves proposed. That is probably not enough.
- **The in-memory document.** The idea that started this design was stronger
  than what it became: a literate document *generated from files across
  machines*, existing only in memory, as a surface for editing them together.
  `hick adopt` already wraps a plain file byte-exactly and the reverse-edit path
  already carries an edit home, so a virtual document gathering regions from
  several real files is closer than it sounds. What stops it being written down
  as a design is that this product's central claim runs the other way — the
  document is the source and the file is the working surface onto it — and an
  ephemeral document inverts that for the duration of an edit. Whether those
  two can coexist without the weaker one teaching people the wrong model is not
  answered here.
- ~~**A worktree is not a stable name.**~~ **Resolved 2026-08-23 by removing
  the axis.** Machines have keys; worktrees do not, and giving them one means
  writing a file into the worktree. If the need returns, that is the decision to
  make first.
- **Two machines that are the same in every way that matters.** Two Linux boxes
  that both need the same override want `os:linux`, and will get two machine
  regions that drift apart. A named group of machines — `group:linux-builders`
  — is the obvious answer and is deliberately not designed here, because a
  group is a declared thing and everything on this list is an observed one.
- **What `hick up` should do with "just here".** A headless loop has nobody to
  ask, and asking is the whole gesture. Refusing to scope from `hick up` is
  defensible and small; a marker line in the saved file is more capable and
  invents syntax in somebody else's C# file. Not decided.
