# Two Branches In One Document

**Status: an investigation, not a plan.** Asked for on 2026-08-21 as
something to think about rather than build. Nothing here is implemented, and
one of the conclusions is that a good deal of it should not be.

> "Maybe there's a way to edit two separate branches in the same document by
> using feature flags in hick files that intelligently know which branch that
> flag is for."

## What already exists, exactly

This matters, because the idea is much closer to the current design than it
first sounds.

`hick-feature` is a real feature system: named features, `requires`,
`conflicts_with`, and exclusive groups, validated into a DAG
(`crates/hick-feature/src/lib.rs`). A document declares them with
`<hick:feature>`, and `<hick:when test="…">` gates content on them through
`hick-condition`'s boolean grammar. Which features are on comes from
`--features a,b`, i.e. from *outside* the document
(`crates/hick-literate/src/lib.rs`, `scan_and_register_features`).

So a document can already contain two mutually exclusive versions of a file
and weave either one. What it cannot do is *notice* which one you are working
on.

## The idea, stated precisely

Three quite different things are bundled in the sentence, and they have very
different value:

1. **A feature set that follows the branch.** `--features` is chosen by hand;
   it could instead default to something derived from `git rev-parse
   --abbrev-ref HEAD`.
2. **Editing both variants side by side** in one document, with the editor
   showing which is live.
3. **A branch's changes living as a feature rather than as a commit** — the
   real, radical reading: no branching at all, one linear history, difference
   expressed as `<hick:when>`.

## 1. Feature sets that follow the branch — worth doing

This is small, useful, and fits what is already here. A `[features]` table in
the project config mapping branch patterns to feature sets:

```toml
[features.branches]
"feature/*"      = ["experimental"]
"release/*"      = ["stable-only"]
"master"         = []
```

`hick up` and `hick run` would default `--features` from the current branch,
and `--features` on the command line would override it — the ordinary
precedence.

It is honest because it changes *a default*, and nothing else. The document
still says what it says; a reader who checks out a branch gets the reading
that branch implies, and can always ask for another. The status bar already
shows the branch, so it can show the feature set beside it, which keeps the
mechanism visible rather than magic.

**Risk:** a document that weaves differently depending on which branch you are
standing in makes `hick test` results branch-dependent in a way that could
surprise CI. Mitigated by making the mapping explicit config rather than a
convention, and by having the verification output name the feature set it
used. That is worth doing regardless.

## 2. Editing both variants at once — partly here already, and the gap is real

Today, content inside a `<hick:when test="not experimental">` block is *shown*
(the editor styles `when` blocks; it does not hide them) but the person
editing has no indication which branch of the conditional is currently live.
That is a genuine gap and a small one to close:

- Dim the inactive branch of a `when`, the way an `#if 0` reads in a C editor.
- Put the active feature set in the status bar next to the branch.
- Offer a toggle that re-weaves under a different feature set without
  changing the document — a preview, not an edit.

None of this needs new language. It is presentation over machinery that
already exists, and it would make the existing feature system considerably
more usable on its own merits, whether or not anything below happens.

## 3. Branches as features rather than as commits — the interesting one, and mostly a trap

The radical reading: stop branching. Keep one history. Express "the version
with the new auth flow" as `<hick:when test="new-auth">` and let everyone work
in one file.

**What is genuinely attractive.** Merge conflicts in this product are unusually
expensive, because a `.hick` document is one file that generates many. Two
branches editing two different generated files still collide in the document
that writes both. A representation where the difference is *declared* rather
than *inferred by diff* would sidestep that, and the three-way merge machinery
now in the editor (`lib/merge.ts`) is a workaround for a problem this would
dissolve.

It also has a real precedent in the literature — this is essentially the
"virtual platform" / feature-oriented software development idea, and the
failure modes are documented.

**Why it is mostly a trap, and this is the part worth remembering.**

- **Conditionals compose multiplicatively.** Two features are four readings;
  ten features are a thousand. The C preprocessor learned this the hard way,
  and `#ifdef` hell is the name it got. `conflicts_with` and exclusive groups
  bound it, but they bound it by *forbidding* combinations, which means
  somebody has to know which ones are nonsense.
- **A conditional never gets deleted.** A branch merges and disappears. A
  feature flag stays in the file until somebody decides it is safe to remove,
  and nobody ever decides that. Ten years of dead flags is the observed
  end-state of every long-lived flag system.
- **Git already does this well.** Branches are cheap, tools understand them,
  and `git log` on a branch is a question this app can now answer visually.
  Replacing that with a homegrown mechanism means losing every tool that
  speaks git — including the blame column and the history graph just built.
- **It breaks provenance.** `provenance-and-standing.md` rests on `git blame`
  composed through lineage. A change that is a flag rather than a commit has
  no commit to blame, so "who wrote this, and was it a model" gets a worse
  answer. That is a direct cost to the property this product is *for*.

**Where it might genuinely win**, and worth a narrow experiment rather than an
architecture: **a document deliberately kept in two variants for a long time**
— a tutorial with a beginner and an advanced path, a runbook for two
deployment targets, a paper with a short and long form. There the variants are
permanent by design, so "the flag never gets deleted" stops being a defect and
becomes the point. That is a *documentation* use, not a branching one, and it
needs nothing new: `<hick:feature>` and `<hick:when>` already do it.

## Taken up, 2026-08-23

`machine-scoped-edits.md` implements (1) and (2) of the list below, in a
narrower form than this document imagined: the feature set follows not just the
branch but **where the weave is standing** — OS, arch, branch — as a reserved
namespace of *observed* facts that no author declares.

**(3) is answered by `the-merged-view.md`, and the answer is that the value was
never in the flags.** Editing several branches in one tab, with the
`<hick:when>`-shaped variants existing **on disk nowhere**, takes the whole
benefit this document was reaching for — a shared region is agreed by
construction, so it cannot conflict later, and the merge pain that motivated
the idea is prevented rather than represented. And it pays none of the four
costs: nothing composes multiplicatively in any file, there is no conditional
on disk to go undeleted, the branches stay real branches so git is not
replaced, and every edit lands in a real file on a real branch so blame and
provenance keep working. The measurement this document asked for before
revisiting (3) — how often merging `.hick` documents actually hurts — is still
worth taking, but it now sizes a prize rather than deciding a question.

## What I would actually do

1. **Do (1)**: branch → feature-set mapping in config, with the feature set
   shown beside the branch in the status bar. Small, honest, useful.
2. **Do (2)**: dim the inactive side of a `when`, and add a preview toggle. It
   makes an existing feature usable and costs no language surface.
3. **Do not do (3) as a branching strategy.** Keep branches as branches, where
   git, blame, provenance and the history graph all already work.
4. **Write the long-lived-variants case up as documentation** rather than as a
   feature. It already works; what it lacks is anyone knowing it does.

The one thing I would want before revisiting (3): a real count of how often
merging `.hick` documents actually hurts. The argument for it rests entirely
on that pain being large, and right now that is an assumption rather than a
measurement — and the merge UI that just landed is about to make it visible
for the first time.
