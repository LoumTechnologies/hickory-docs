# Lenses: the viewer is a block editor, and a document is one thing it views

*Status: adopted 2026-09-03. **All six steps built the same day**: the
history lens; File → New Project as a recipe commit through a temporary
index, `Hick-Output` a tree hash the lens checks; replay into a sibling
above the floor or a child below it (`a-recipe-commit-can-be-replayed.md`);
the tail as the next commit, prose or recipe
(`the-tail-of-the-story-is-the-next-commit.md`); reword, move and drop as
git's own rebase above the floor
(`the-past-is-edited-by-rebase-above-the-floor.md`). The re-ingest merge is
retired with it. It
names what two built things already are — the generated-file tab and the
merged view (`the-merged-view.md`) — and adds a third, the **history lens**,
which takes over scaffolding from `owning-what-a-scaffolder-wrote.md`. That
document stays the record of the two refusals (CRDT edits and line-offset
patches) and of why a durable claim must not point into a cache; it is
superseded here on where a scaffold lives. `changes-not-commits.md`,
`expression-and-log.md` and `three-axes.md` are assumed and not restated.*

> "`.hick` files are still a thing, but the hick viewer views more than just
> files. It can view git logs or multiple files across branches, and serve as
> a way to edit git commits, create new commits, alter files in unison —
> almost like block-level editing."

> "This is kind of like Emacs, isn't it?"

## The complaint that started it

Scaffold a project with `dotnet new`. The document shows the command and,
beneath it, the files it wrote. Edit one of those files and the command's
output disappears, because the cell's inputs have moved and the recording no
longer matches. The document was saying two true things — *this command
produced these files* and *these files are not what the command produced* —
and had one place to say them.

The design behind that cell treats a scaffold as a living expression: re-run
it with a newer SDK and three-way merge the result over your edits. But a
scaffold is not an expression. It is an act that happened once, on one day,
with one SDK, and `expression-and-log.md` already says where acts go: **the
document describes the present, git holds the past.** The cell was duplicating
a job git does, and doing it worse — the spec's own open edge admits the
re-run is "close enough to the re-derivable history `changes-not-commits.md`
refuses that the boundary deserves saying out loud."

The fix is not a better cell. It is to notice that the app already views
things that are not documents, and to say what the rule is.

## What a lens is

A **lens** is a synthesized document: blocks drawn with the same cards a
`.hick` document is drawn with, over something that is not a `.hick` file.
Every block in a lens declares three things:

1. **What it is a view of.** A span of a file. A span of a generated file. A
   variant that differs across N sources. A commit's message. A commit's
   tree. The working tree.
2. **How a change to it gets home.** A byte edit. The reverse edit through
   lineage. A write to the one source that matches. A reword. A rebase. The
   next commit.
3. **Whether a change is allowed right now**, and by what rule. Always. Not
   for synthetic bytes. Only additively. Only above the publication floor.
   Never.

That third column is the whole design, and nothing in it is new. Every entry
is a rule the repository has already decided somewhere else — the reverse
edit's `422` for synthetic bytes, the merged view's "a variant may only add",
the floor `hick emit` and the Git pane's amend already refuse below. What is
new is that the question is asked **per block instead of per tab**.

The viewer is therefore a **block editor**. A `.hick` document is the one
thing it views whose blocks are also its storage. Everything else is a lens.

### The four lenses

| Lens | A block is a view of | An edit gets home by | Allowed when |
|---|---|---|---|
| Plain file | a span of the file | a byte edit | always |
| Generated file | a span of the file, with lineage | the reverse edit, to the document's span | the bytes are not synthetic |
| Merged view | a region agreed by N sources, or a variant | a write to each matching source, reported per target | a variant may only add |
| **History** | a commit's message; a commit's tree; the working tree | reword; rebase; the next commit | above the publication floor; the tail always; below the floor never |

The first two exist and are not called lenses. The third exists and is. The
fourth is this document.

### What a lens is not

**A lens is never saved.** It has no path and no `.hick` extension, so nobody
can commit a view and mistake it for a source. The merged view drew that line
— "the view is the merge, held live" — and it holds for all of them. Dired
cannot be saved, and that is the model rather than the exception.

**A lens's blocks come from the substrate, not from tags.** A commit boundary
is a fact git reports; a file boundary is a fact the filesystem reports. No
lens is parsed out of text it drew, so the parser's byte-for-byte invariant is
untouched and there is no second grammar.

**A `.hick` file stays the only thing that is authored.** Lenses read many
things and write back to each through its own verb. The risk this document
exists to prevent is that "the viewer views more than files" quietly becomes
a second document format with its own save path.

## Emacs, and where this leaves it

Emacs's one idea is that everything is a buffer: text, plus a mode that says
what the text means and how edits get home. Dired is a buffer over a
directory. Magit's status buffer is a buffer over a repository, with commits
and hunks drawn as text and staging, rewording and rebasing as edits to that
text — it is the history lens, built twelve years ago in Lisp. Org is the
literate document. The repository already leans on this once: the persistent
terminal is comint, and `a-terminal-that-writes-the-document.md` names the
line Emacs never crossed (line-oriented interaction unifies with editing,
screen-oriented does not).

Four things differ, all on purpose:

- **Blocks, not text.** Magit spends real effort keeping its buffer parseable
  by itself. A block editor's blocks come from the substrate, so a section is
  a fact and not a regex.
- **A floor, not trust.** Magit will rebase published history and let you
  force-push. The floor is what makes the history lens safe to hand to
  someone who is not an Emacs user.
- **No save.** Any Emacs buffer can be written to a file. A lens cannot.
- **Provenance.** A hunk that came from a replayed recipe and a hunk you typed
  are not the same colour. Emacs has no such axis.

What is worth taking whole is Magit's insight that the **section** is the
unit of interaction and the same keys act on a section wherever it lives.
Run a cell and replay a commit are one gesture. Fold is fold everywhere. The
commit card's verbs and the cell card's verbs are the same verbs where they
mean the same thing.

## The history lens

The commit history, drawn top to bottom with document cards. The message is
prose. A commit that carries a **recipe** is a cell: its command, and the
diff as the cell's output. The working tree is the last card. It reads
**oldest first**, because a narrative reads forward and a document already
does; the past folds by default and the view opens at the tail.

### A replayable commit

A commit whose message carries its recipe as trailers:

```
Scaffold a web API

Hick-Recipe: dotnet new webapi -o . --no-restore
Hick-Image: mcr.microsoft.com/dotnet/sdk:9.0
Hick-Output: 9f2c…c4e1 app
```

`Hick-Output` is the **git tree hash** of the scaffolded folder as the
scaffolder wrote it, and the path it sits at. Not an opaque fingerprint:
a tree hash is something git can check against the commit's own tree with
one `rev-parse`, no replay needed — which is what gives a recipe card its
first state below.

This is the shape `hick emit` already has — each emitted commit carries the
document version that emitted it — with the recipe in place of the version.
File → New Project becomes: run the scaffolder, commit the result with its
recipe, stop. The four lines you change are the next commit. The scaffold's
output is visible forever, because it is the commit's tree, and the scaffold
card can say *edited since, two commits down*. That is the fact the cell
could not show.

### The in-between moment is designed away

A recipe commit is honest only if its tree is exactly what the recipe
produced. Run the scaffolder, edit a file, *then* commit, and the edit is
fused into the recipe commit where no replay can separate it — the commit
cannot be upgraded. So the product never produces that moment: File → New
Project runs the scaffolder into a scratch directory and commits what it
wrote **as one act**, with nothing a person can do in between. There is no
uncommitted scaffold, and so nothing to stash, stage or mark as pending.

Git makes this safe to do with other work in flight. The commit is built
through a **temporary index** — `read-tree HEAD`, add exactly the files the
run wrote, `write-tree`, `commit-tree`, `update-ref` — so the person's own
staged and unstaged changes are neither swept into the recipe commit nor
touched. Afterwards the scaffold's paths are added to the real index so
`git status` agrees with HEAD about them. The one refusal is a target folder
that already holds anything: a scaffold is committed exactly as the
scaffolder wrote it, so it needs an empty folder of its own — and never the
repository's root, since `Hick-Output` would then name a tree that is not
the scaffolder's.

Someone can still make the in-between moment by hand — `dotnet new` in a
terminal, an edit, a commit with hand-written trailers — and the lens
catches it, because the tree hash is checkable by git alone. A recipe card
therefore has **three states, drawn apart**:

- **Matches its recorded output.** Derived: git holds exactly the tree the
  trailer names at that path. Nothing was edited before it was committed,
  so replay and rebase can upgrade it. Still a claim about what the
  scaffolder wrote — a matching hash can be hand-written — so the word is
  "matches", never "verified".
- **Edited before it was committed.** Derived mismatch. The edits cannot be
  separated from the scaffold without re-running the old recipe exactly, so
  the card offers no replay and says why.
- **Replayed: same, or differs.** Evidence, from an actual run. The only
  state that speaks to whether the recipe still produces this today.

### Replay makes a new commit, never remakes the old one

`expression-and-log.md`: a session may produce a commit, and nothing may
re-produce one that exists. Replaying scaffold commit S1 with a newer SDK
produces **S2, a sibling from the same parent**. Your edit commits then move
onto S2, and git does the three-way merge with S1 as the base. That is
exactly the merge the scaffold cell wanted, with the base being a commit —
so it survives a clone for free — and git's merge doing the work instead of
a document-level mechanism over bytes that share no history.

The floor decides how the move happens. Above it, moving your edits onto S2
is a rebase, which is allowed there. Below it, nothing is rewritten and the
replay lands as a merge commit. The Git pane already computes the floor per
row (`merge-base(HEAD, @{upstream})`, then `origin/HEAD`, then
`origin/master`), and the lens reads the same fact.

Replay is a real run, possibly in a container. It is a verb on a card, never
something the view does on its own while drawing, and it refuses a dirty
working tree the way the pane's other verbs do.

### What the recipe can prove

The trailer is a **declared** claim, in the commit's own words. Replay is
what verifies it: run the recipe, compare the output fingerprint through the
one comparison `hick test`, `equiv` and the pin already share, and the card
says *same as S1* or *differs*. On the evidence axis a recipe commit is
therefore recorded, stale or unrecorded like any cell. Say **"no evidence of
drift"**, never "reproducible", about a commit nobody has replayed.

The three provenance kinds stay visibly apart on every card: the message is
declared and unverifiable, the diff is derived, a replay result is evidence.

### Where writing is allowed

Two places, and the floor separates them.

- **The tail is the working tree**, drawn as an unrecorded cell. Prose typed
  there is the next commit's message. A command typed there runs, and its
  result is committed with its recipe. This is `hick emit` with a place to
  type it — and it is the *only* place a lens creates something new.
- **Above the floor, editing the past is a rebase.** Reword is editing a
  card's prose. Reorder is moving cards. Drop is deleting one. Each is git's
  own verb, run as itself, with git's own words shown when it refuses. The
  floor row is where cards go read-only.

A change across several cards is **one act**, whichever commits it touches —
rewording three messages, reordering four cards — which is what
`local-history.md` means by "the unit is the act, not the file". The lens is
where a batch act happens on screen; local history is what lets it be undone.

### Merges

A merge commit is a card with two parents. Drawn as prose with a diff it
loses what happened. It gets the diverged surface that already exists —
base, ours, theirs, three ways out — read-only below the floor, and above it
the same three ways out as a redo of the merge.

## What this retires

- **The scaffold cell.** `exec > ingested > file` for scaffolds, the
  re-ingest three-way merge, and the `hick:ingested` chip on an exec card.
  Scaffolding writes a commit, not a document.
- **The open edge** in `owning-what-a-scaffolder-wrote.md` on whether an
  ingest should be re-runnable in place. Answered by construction: it should
  not, because a commit is the thing that is re-runnable, and only into a
  new commit.

What stays: `hick ingest --from '#cell'`, for the case it was really about —
bringing an exec's output volume into a document you are writing — and the
ingested origin, for bytes that are present in a document and are not yours.
A scaffolded project can still become literate afterwards, through the
existing *make literate* path, **by choice and never by default**.

## What it does not replace

Not the `.hick` document. The document describes the present and is
runnable; the history describes how the present got here and is replayable
only into new commits. Same cards, opposite arrow of time, and a card in the
history never claims it can regenerate itself in place.

Not the Git pane. The pane does the daily loop — stage, commit, push, pull,
branch — in a list. The lens is the same repository read as a story, and the
two share every verb and every refusal. When both exist, the pane is the
index and the lens is the page.

Not the phone's business to execute. The phone reads and never runs
(`notes-ide.md`), but it can read a weave: a history woven to markdown gives
it the story without the verbs.

## Three hard parts

1. **Scale.** A thousand commits is not a document anyone reads top to
   bottom. The past folds the way a session folds the agent's work, the view
   opens at the tail, and folding is by the same mechanism (`editor/folding.ts`)
   rather than a second one.
2. **A rebase that stops.** Reordering cards can conflict. The lens must
   leave the repository in the state git leaves it — mid-rebase, with git's
   own words — and show that as the diverged surface, never hide it behind a
   spinner or undo it silently. `GIT_TERMINAL_PROMPT=0` and the pane's rules
   apply unchanged.
3. **Recipes that lie.** A trailer anyone can write is a trailer anyone can
   get wrong. The card's colour comes from replay evidence and nothing else;
   an unreplayed recipe is drawn as unrecorded, however confident its prose.

## Sequence

1. **Name the lenses.** Update the generated-file tab's and the merged view's
   guarantees to say "lens" and the three-column rule. No behaviour changes.
2. **The read-only history lens.** Commits as cards, oldest first, folded
   past, opened at the tail; recipe commits drawn as cells with the diff as
   output; the *edited since* fact on a recipe card; the floor row marked.
3. **Scaffold writes a commit.** File → New Project commits with the three
   trailers and writes no document, through a temporary index, refusing an
   occupied folder. The lens draws the three states above. The scaffold
   cell path is deleted. The `hick ingest --from '#cell'` verb stays.
4. **Replay.** The verb on a recipe card: sibling commit, fingerprint
   compared, rebase above the floor and merge below it, dirty tree refused.
5. **The tail.** The working tree as the last card; prose there is the next
   message; a command there emits.
6. **Editing the past.** Reword, reorder, drop as rebase above the floor,
   each one act.

Steps 2 and 3 together answer the complaint at the top; nothing after them
is needed to be rid of the cell.

## Open edges

- **What else emits a recipe.** Any `hick run` whose cell writes files could
  commit with one, which is `hick emit` grown up. Whether that is wanted, and
  whether a document version and a recipe should ever both be trailers on
  one commit, is not decided here.
- **A lens over a lens.** The history of a merged view — N branches' logs
  drawn as one story with variants — is the obvious next composition and
  nothing here says whether it is coherent.
- **Which lenses the peer channel may carry.** Remote view of a document is
  designed (`one-engineer-many-machines.md`); remote view of a history lens
  is a read of another machine's repository, and `read across, write local`
  suggests it is fine to read and never to rebase.
- **Blocks without a substrate boundary.** A commit's diff is one block; a
  person may want a hunk to be one. Whether hunks are blocks — Magit says yes
  — is deferred until the read-only lens shows whether a diff card is too
  coarse to read.
