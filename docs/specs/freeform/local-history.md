# Local history: the interval below the commit

*Status: proposal, 2026-08-26. Not built. Grew out of a sentence this
repository already writes down as a limitation —
`docs/guarantees/search/find-and-replace-is-exhaustive.md` ends with "a
multi-file undo inside the app is not implemented", and points at git as the
way back. Git is the way back **to the last commit**, which is not the same
thing. Pairs with `expression-and-log.md` (the document is the present, git is
the past), and takes its shape from the store that already exists for drafts
(`crates/hickory-workspace/src/lib.rs`).*

## The gap

Git's resolution is a commit. Everything between two commits is one
undifferentiated jump, and this product spends most of its time in there,
because in a `.hick` folder **you are not the only writer**. Between your last
commit and now, the following may all have written to your files:

| Writer | What it wrote | How you would get back today |
|---|---|---|
| you, typing | prose, structure | CodeMirror undo, until the tab closes |
| `hick run` / the up-loop | every generated output | re-run, if the inputs still say so |
| `hick weave` | the `.md` of every document | re-weave |
| the reverse edit | source, from an edit made in an output | nothing |
| `hick ingest` | a scaffolder's whole tree, as document bytes | nothing |
| the agent's tool surface | source, at hashline anchors | rewind the session, if it is still there |
| find-and-replace | up to 5,000 matches across the folder | nothing |
| the merge driver | a resolved document, at `git merge` time | `git merge --abort`, if you have not moved |
| another editor, on disk | anything | nothing |

Six of those nine have no way back that is shorter than "commit before every
risky thing, forever". That is a discipline, and a discipline is what you ask
of people when the tool has not done its job. **Local history is the tool
doing its job**: a per-user, per-project record of what every file looked like
before each of those writes, kept for a while, revertable, and belonging to
nobody but the person at the machine.

The name is deliberate and borrowed: JetBrains has had exactly this for
twenty years, and it is the feature its users notice missing everywhere else.

## The unit is the act, not the file

This is the one place the design departs from JetBrains', and it departs for a
reason visible in the table above: **almost every writer here is a batch
writer**. A weave touches every document's `.md`. A replace touches forty
files. An ingest lands a scaffolder's entire tree. A run rewrites every output
in the pipeline. Per-file entries would record all of that correctly and make
it useless — undoing a replace would be forty separate reverts, done in the
right order, by hand, from memory.

So the record is a list of **acts**, each holding one or more file versions:

```
act  a7f3   14:02:11  replace   "invoice_id" → "invoice_ref"   41 files
act  a7f2   14:01:55  typed     notes/billing.hick             1 file
act  a7f1   13:58:02  run       notes/billing.hick             9 outputs
act  a7f0   13:57:40  agent     notes/billing.hick             1 file   session 20260826-135102
```

An act is the unit of display, the unit of revert, and the unit of eviction.
A file's own timeline is a *filter* over the acts that touched it, not a
separate structure — which is what keeps "undo that replace" and "what did
this file look like at lunchtime" the same mechanism.

## Every act says who made it

Not as decoration. The kind is what makes the list readable at a glance and
what makes revert safe to offer, because the answer to "is going back here
sane?" is different per kind:

- **`typed`** — a person, through a buffer. Coalesced: a burst of typing with
  no other act between it is one act, cut at a save, a pane change, or a
  quiet gap.
- **`saved`** — the buffer reached disk. Cheap to record and the stop most
  people actually want.
- **`external`** — the file changed on disk with no buffer behind it. The app
  already watches for this (`docs/guarantees/collaboration/the-app-sees-external-edits.md`);
  this is the first thing that writes it down.
- **`run`**, **`weave`** — generated outputs, before the write. Reverting one
  is nearly always wrong (the next run undoes the revert), so the app offers
  *compare*, not *revert*, and says why.
- **`reverse-edit`**, **`agent`**, **`ingest`**, **`merge`**, **`replace`**,
  **`refactor`** — a machine wrote your source. These are the ones the feature
  exists for. Each carries the identifier that explains it: the run
  fingerprint, the session path, the pattern, the merge's base.
- **`revert`** — going back is itself an act, recorded like any other, so the
  way back from a bad revert is the same list. A history you can fall out of
  is a history nobody trusts.

## It is not a provenance family, and nothing may cite it

This is the constraint that keeps the feature from doing damage, and it is
learned rather than invented. `scaffolded-files-and-derived-edits.md` died on
one mistake: `from=` **built a durable claim on a gitignored artifact**, so
the base did not survive a clone. Local history is a gitignored artifact by
construction — stronger, it is not even in the project — so:

- **No element may point at it.** No `from=`, no `cites=`, no attribute
  anywhere in the hick grammar takes a local-history address. If a document
  needs to say where bytes came from, the answer is one of the three
  provenances (`three-provenances.md`), or continuity, or a commit.
- **It is not the fifth provenance.** It answers *what was this a minute ago*,
  for one person, on one machine — a question about a workspace, not about a
  document. `three-provenances.md`'s whole point is that families of
  provenance must not be mistaken for each other, and a family that vanishes
  on a fresh clone would be the worst one yet.
- **It never crosses git.** Not committed, not committable.
- **It never crosses the peer channel.** `one-engineer-many-machines.md` is
  explicit that durable state crosses git and only liveness crosses the peer
  channel. Local history is neither: it is local scratch. Two of your machines
  have two different local histories and that is correct, not a bug to fix
  later.
- **The phone does not have one.** Mobile reads and captures; there is no
  executor, so seven of the nine writers cannot occur there, and the two that
  can are covered by drafts.

Say **"local history"**. Never "version history", "backup", or "snapshots" —
each of those words promises durability across machines, which this
deliberately does not have, and the promise is the harm.

## Where it lives

Under the user's own data directory, keyed by the project's canonical path —
the same place, and for the same four-sentence reason, as drafts and UI state
(`crates/hickory-workspace/src/lib.rs`). Git cannot reach it by construction,
which needs no discipline from anyone and survives a `git add -A` in a folder
where `hick init` never ran.

The shape is the smallest thing that works:

- **Content-addressed blobs.** A file version is `sha256 → bytes`, written
  once. A weave that rewrites forty `.md` files with identical content stores
  one blob. Unchanged files in a batch are recorded as an unchanged hash, not
  a copy.
- **An append-only act log.** Each act is a line: id, wall-clock time, kind,
  the identifier its kind carries, and the list of `(path, before, after)`
  hashes. Append-only means an interrupted write loses the tail and never the
  middle, which matters because this is exactly the store you reach for after
  a crash.
- **Base bytes are already the house style.** Every draft records what it was
  taken from, precisely so reopening is a three-way merge rather than a
  standoff. Local history is that idea with the timestamp made plural.

No new crate. `hickory-workspace` is already "where the app remembers what you
had open and what you had not saved yet", and this is the third thing of that
kind.

## What it can do that git cannot

1. **Undo a replace.** The act holds forty `(path, before, after)` triples;
   revert writes `before` back to each path whose current bytes still equal
   `after`, and reports — never silently skips — the ones that have moved on.
   This closes the sentence at the end of
   `find-and-replace-is-exhaustive.md`.
2. **Show the document before the run.** Not "re-run and hope the inputs are
   the same" — the actual bytes.
3. **Recover from an ingest.** `owning-what-a-scaffolder-wrote.md` makes a
   re-ingest a three-way merge with a coarse base, and says plainly that
   volatile regions are deliberately not built "until the merge has produced
   enough false conflicts to show what they look like". Those false conflicts
   are much cheaper to study, and far less frightening to trigger, when the
   document immediately before the merge is one click away.
4. **Undo an agent's write without rewinding the session.**
   `rewind-and-re-run-are-two-different-acts.md` keeps those two verbs apart;
   there is a third thing a person wants, which is neither — *keep the
   conversation, put the file back* — and it has no home today.
5. **Work in a folder that is not a repository.** The time slider says, with
   the right amount of grace, that a folder of notes under no version control
   is entirely normal and there is simply no history to slide through. Local
   history is the answer for those folders, and for the interval before the
   first commit in every other one.

## Retention

Bounded, because an unbounded local store is a slow disk leak that shows up as
a bug report about startup time.

- Acts are evicted **oldest first**, under a per-project byte budget and an
  age limit, whichever bites first. Defaults belong in `Config`
  (`HICKORY_HISTORY_BYTES`, `HICKORY_HISTORY_DAYS`) per
  `config-and-environments`, typed and validated at boot like everything else.
- Blobs are reference-counted against the act log and swept when the last act
  naming them is evicted.
- **An act below the last commit is evicted freely** — git holds that. The
  budget therefore mostly protects the interval it exists for.
- `hick history forget --path <p>` and `--all` exist and are honest verbs,
  because this store holds bytes their author never chose to keep. A file that
  briefly contained a secret is the obvious case, and the answer is a purge
  the person can run, not a redactor we pretend to have.

## Surfaces

- **CLI.** `hick history` (recent acts), `hick history <path>` (filtered to
  one file), `hick history show <act> [--path p]`, `hick history revert <act>
  [--path p]`, `hick history forget`. Every one of them works offline and in a
  folder that is not a repository — which is most of the point.
- **App.** A panel in the same family as the git pane, and the same `⌘Z`
  sensation one level up: the act list on the left, a diff on the right,
  revert per act or per file within it. Generated outputs are shown and
  offered *compare* rather than *revert*, named the same way find-and-replace
  greys out a generated file — the row is real and worth reading, the write is
  refused with a reason.
- **The time slider stays where it is.** `hick lineage --at/--history` slides
  through *commits* and recomputes lineage by weaving them. That is a
  different question with a different answer, and merging the two controls
  would put a citable thing and an uncitable one on the same track.

## What this is NOT

- **Not a versioning system.** It has no branches, no names, no merge, no
  push. Anything you want to keep, you commit; that is what commits are for,
  and `expression-and-log.md` is unamended by this document.
- **Not a backup.** It is on the same disk, in the same account, and it is
  evicted on a timer. Say so in the panel.
- **Not undo.** CodeMirror's undo stack is finer, is per-buffer, and dies with
  the tab. Local history is coarser, is per-project, and survives everything.
  They overlap in the middle and that is fine; what would not be fine is
  wiring one into the other and getting a `⌘Z` that sometimes reverts a
  scaffolder.
- **Not a record.** `provenance-across-versions.md` is careful that the
  correspondence journal is *a record, not a cache*, which is what makes it
  committable. This is the opposite by design: a cache, not a record. The two
  must not grow into each other.

## Open edges

- **Coalescing `typed` acts is a guess.** Too eager and the list is a
  keystroke log; too lazy and an hour of writing is one stop. The cuts that
  are certainly right are a save, a pane change, and any other kind of act;
  the quiet-gap threshold is the guess, and it should be one constant with the
  reasoning next to it rather than a setting.
- **Collaboration.** With `hick-grove` live, "who typed this" inside one
  `typed` act is not answerable from bytes. The honest first version records
  the act as `typed` with no author and does not pretend otherwise; the
  session is where a name lives.
- **Reverting a document while an output edit is in flight** needs an order,
  and the safe one is to refuse rather than interleave. A revert during a run
  is the same question.
- **Whether an act should be able to name a commit it preceded.** Tempting —
  it would make "everything since the last commit" a cheap query — but a
  commit id inside an uncitable store is the first step toward somebody citing
  it. Probably: record the id, expose it only as a filter, never as an
  address.
- **Retention defaults are unmeasured.** Nothing here should ship a number
  that was not watched on a real folder for a week first.
