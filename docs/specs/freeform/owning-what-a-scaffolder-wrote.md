# Owning what a scaffolder wrote

*Status: design of record for how a document owns files it did not type.
Adopted 2026-08-23. **Sequence steps 1 and 2 are built** (2026-08-23):
`hick ingest --from '#cell' <doc>.hick` writes `<hick:ingested>` with the
gitignore filter and the non-UTF-8 refusal, and the ingested `SourceOrigin`
keeps blame and the reverse edit honest — see
`docs/guarantees/authoring/ingest-owns-what-a-scaffolder-wrote.md` and
`docs/guarantees/lineage/ingested-bytes-are-not-yours.md`. **Steps 3–5 (the
re-ingest merge, volatile regions, the correspondence record) are not.**
**Supersedes**
`scaffolded-files-and-derived-edits.md` on its mechanism — `from=` pointing at
a captured exec, and a `<hick:derive>` block of structurally-anchored edits over
it. That document remains accurate and worth reading on the two things it
settled: why **CRDT edits** cannot express a derivation from a foreign artifact
(a re-run shares no history with the previous run, so the property that makes
CRDTs merge is absent exactly where it would be needed), and why **line-offset
patches** — `patch`, quilt, a `series` file — are the fifty-year-old version of
the same mistake. Both refusals stand. What changes is what replaces them.*

> "I want to think about if we can support a hick file that builds an app where
> a big part of bootstrapping the app is running a command like `dotnet new`,
> which generates a bunch of files; and then we want to modify those files."

## The gap, unchanged

A literate document declares whole files. `<hick:file path="…">` holds every
byte it writes, which is what makes the guarantees work: the weave is the file,
provenance is byte-precise, and an edit on the generated side maps back to a
span in the document.

`dotnet new webapi` writes forty files nobody typed, and the interesting work is
changing four lines across three of them. Paste the scaffold in and the document
becomes a silent snapshot — the next SDK writes a different `Program.cs`, and
the document keeps generating last year's while claiming to be what `dotnet new`
produces. Leave it out and the scaffold is a step in a README, so the document
no longer describes the program.

## Why the recorded-transcript answer fails

The superseded design's step 1 was a `hick:file` whose bytes come from a
captured exec output — `from="scaffold:src/Program.cs"` — resting on the claim
that "the base is pinned and recorded, so the weave is reproducible offline."

**The base is gitignored.** Captured transcripts live in
`.hick-cache/transcripts/` (`crates/hick-literate/src/cache.rs:99`), and
`hick init` writes `.hick-cache/` into `.gitignore`
(`crates/hickory-cli/src/init.rs:165`). A clone therefore holds the reference
and not the referent, and its only recovery is to run `dotnet new` itself —
which is the manual-README failure the design existed to rule out. `from=` is
reproducible on the machine that made it, which is precisely the property a
scaffold must not have.

That is not a bug in the cache. A transcript cache *should* be gitignored: it is
derived, it is large, and `local-only.md` is emphatic that the durable state is
the user's git repository. The mistake was building a durable claim on top of a
disposable artifact.

## Ingest is the mechanism

Scaffold output is material somebody else's tool produced, arriving in a notes
folder. That is `ingest.md`'s subject exactly, and its whole discipline
transfers without amendment:

- **It never deletes the user's bytes.** They came from outside and we do not
  own them.
- **It never calls a model.** Parsing bytes into document elements is offline,
  first-party, deterministic.
- **Identity is the content hash of the source bytes**, recorded so that
  ingesting the same thing twice produces one result rather than two.

What does not transfer is the **door**. Ingest's three ways in are an inbox
directory and a scratchpad, and a scaffolder's output is neither — it is a
volume inside a container that the document itself just ran. So this is the same
verb through a different door: `hick ingest` reads an exec's output volume,
which is already extracted, unpacked, and merged into the pipeline result
(`crates/hick-literate/src/lib.rs:1804-1833`). Nothing watches a directory and
nothing is moved.

The unit of identity changes with the door. The inbox's rule is one note per
source file; here it is **one fingerprint per run, N files under it** — a
scaffold is a single event that happens to write forty things, and forty
unrelated hashes would lose the fact that they came from one command.

## What it writes, and why it nests

The instinct that the files belong *under* the exec is right — containment says
"running this produced these" without a string reference to resolve. Two things
have to be fixed for it to survive contact with the rest of the language.

**The exec's body is the command, as raw text.** Mixed raw text and child tags
already parse, so forty file bodies would sit as siblings of the command with
nothing marking which bytes get run. The existing idiom fixes this: wrap the
command in a child so it stops being ambient text, exactly as `<hick:copy>`
inside `<hick:exec>` already does.

**`hick:file` containing `hick:exec` already means the opposite.** It means "run
this, paste the output here" (`parse_file_with_nested_exec`,
`crates/hick-lang/src/lib.rs:1978`). If `hick:exec` containing `hick:file` meant
"running this produced these", the same two tags in either order would mean
inverse things, and a reader would have to check the nesting direction to know
which way the data flows. An intervening element that names the relation removes
the ambiguity — the containment a reader parses is `exec > ingested > file`,
which resembles nothing else.

```
<hick:exec container="sdk" image="mcr.microsoft.com/dotnet/sdk:9.0">
<hick:copy id="scaffold">
dotnet new webapi -o .
</hick:copy>
<hick:ingested from="#scaffold" sha256="9f2c…" at="2026-08-23" files="38" skipped="2">
<hick:file path="Program.cs">
…every byte the scaffolder wrote, and then the four you changed…
</hick:file>
…
</hick:ingested>
</hick:exec>
```

`sha256=` is over the run's output as a whole, and it is the recorded base
everything below depends on. `skipped=` is not decoration — see *what cannot be
ingested*.

## Three properties this buys

**Your four lines need no new primitive.** They are ordinary edits to ordinary
`hick:file` bytes. No `select=` grammar, no anchor resolution, no rule that an
unresolvable anchor must fail the weave, no refusal when an anchor resolves to
two places. The entire hard-parts section of the superseded design does not
arise, because there are no anchors.

**The reverse edit works.** Exec-origin bytes are `synthetic` and rejected with
422 by `POST /api/docs/:id/outputs/edit`
(`docs/guarantees/execution/output-lineage-round-trips-byte-for-byte.md`) —
correctly, because there is nowhere in the document to put an edit to a
command's output. Ingested bytes are byte-precise spans in the document, so
editing `Program.cs` in the app lands in the `.hick` through the path that
already exists.

**The base survives a clone**, which is the whole point of the previous section.

## Provenance: present bytes that are not yours

Ingested bytes must not read as `Literal`. The superseded document was right
that pasting a scaffold makes "forty files of somebody else's code claim to be
literal document text you wrote", and ingesting them without marking them has
exactly that defect. They must not read as `Exec` either, or they become
synthetic and the reverse edit dies with them.

The language already has this shape. `SourceOrigin::Paste` carries a selector
*and*, when the bytes are byte-identical to a region of a source file, the file
and span they match (`crates/hick-flow/src/node.rs:44`) — present bytes that
also name where they came from. An ingested origin is the same dual thing: a
byte-precise span in the document, carrying the fingerprint of the run that
produced it. Ribbons then show three colours in one file honestly — the
scaffolder's bytes, your four lines, and anything woven in — where a paste would
have shown one colour and lied.

## The re-run, and the bulk

**The re-run is a three-way merge with a real base.** The old ingested bytes are
the base, the fresh `dotnet new` is theirs, the document is ours, and
`hick-merge` already does three-way merge over documents. This is also the
richest kind of continuity recording site — all three sides in hand at one
moment — and `provenance-across-versions.md` carries the row.

Its precision is coarse, and for a principled reason rather than a gap in
coverage: two runs of a scaffolder share no history, so **no byte-precise thread
exists to record even with the tool watching the whole time.** That is the
distinction the continuity design would otherwise blur, and this is the case
that forces it.

**The bulk is a real cost and is not paid off.** The repository does carry forty
files of boilerplate. Ingest fixes the *silence* — drift becomes a merge you are
shown rather than a divergence nobody detects — and it does not fix the noise in
a diff. The compensation is that the noise happens once, in the ingest commit,
and your four lines are the next commit: `git log -p` shows precisely "the body
of `Main`, replaced", which is the legibility the `derive` block was reaching
for. It is `expression-and-log.md`'s division of labour — the document describes
the present, git holds the past — applied to somebody else's bytes.

## What cannot be ingested, and must be refused by name

**Non-UTF-8 files.** A `hick:file` body is raw bytes under the no-escaping
invariant, and a volume's contents are read as lossy UTF-8 today
(`crates/hick-literate/src/volume_state.rs:100`), so a binary artifact would be
silently mangled into the document. Ingest must count it and name it — the
`skipped=` attribute above — in the style `claude-code-sessions.md` already uses
for what an import drops. A side-car for binaries is a later question and is not
designed here.

**Anything the project would gitignore.** `bin/`, `obj/`, `node_modules/` — a
scaffolder writes build output alongside source, and ingesting it would put
derived bytes into the document that owns the source. The repository's own
`.gitignore` is the filter, which needs no new configuration and is what the
user already means.

The superseded document's constraint survives, inverted. It said an exec whose
output volume is not fully captured cannot be a `from=`. The rule now is: **an
exec whose output cannot be represented as document bytes cannot be ingested**,
and the refusal names the files and why, per `user-facing-errors.md`.

## The hard parts, honestly

**Scaffolders are not deterministic.** `dotnet new` mints a user-secrets id;
other generators stamp a timestamp, a GUID, or the current directory's name into
their output. A re-run therefore differs in ways that are pure noise, and a
three-way merge that surfaces them as conflicts teaches people to click through
conflicts. Something has to distinguish *the SDK changed this* from *the SDK
randomizes this*, and the honest first answer is that the person marks a region
as volatile once and the merge stops asking. Detecting it automatically sounds
helpful and would produce surprises.

**Review noise is worst on the first commit and permanent in blame.** Forty
files arriving as one document diff is tolerable; the same forty files
reappearing in the document's blame every time somebody asks who wrote a line
near them is the ongoing cost. The ingested origin is what keeps blame honest —
it is the difference between "you wrote this" and "this arrived on 2026-08-23
from `dotnet new`".

**A scaffolder that writes outside its volume** — into `~/.dotnet`, into a
global package cache — has done something the document cannot own, and the
document must not pretend otherwise. That is the sandbox's boundary
(`docs/guarantees/execution/a-sandboxed-cell-cannot-reach-past-its-workdir.md`),
and the scaffold's reproducibility claim stops at it.

**The SDK version is the other half of reproducibility.** An ingest records what
one version of one tool produced on one day. Pinning the image is what makes the
next run comparable at all, and an ingest from an unpinned image should say so
rather than implying a base it cannot reproduce.

## Sequence

1. ~~**`hick ingest` over an exec's output volume**~~ **Built 2026-08-23.**
   With the gitignore filter and the non-UTF-8 refusal, writing
   `<hick:ingested>` and its fingerprint. One thing the design did not name
   fell out of building it: **a volume a document has ingested is no longer
   flushed as a pipeline output**, because otherwise the next `hick run`
   silently overwrites your four lines with the scaffolder's originals.
2. ~~**The ingested origin** in provenance~~ **Built 2026-08-23.**
   `SourceOrigin::Ingested { file, span, run }` and `Origin::Ingested`, which
   is editable (so the reverse edit works on scaffold bytes) and never reports
   as `literal` (so blame does not say you wrote them).
3. **Re-ingest as a three-way merge** through `hick-merge`, with the recorded
   `sha256` as the base.
4. **Volatile regions**, once the merge has produced enough false conflicts to
   show what they actually look like.
5. **The correspondence record** from the re-ingest, if and only if continuity
   is on (`provenance-across-versions.md`).

Steps 1 and 2 are worth doing on their own merits even if the rest never
happens, which is the test of a decomposition here.

## Open edges

- **Binary scaffold output has no home**, and some scaffolders emit it on the
  first run. Counted and refused is honest; it is not sufficient forever.
- **Whether an ingest should be re-runnable in place.** Re-ingesting rewrites a
  region of the document from a command's output, which is a document editing
  itself — legal, since a person directed it and nothing re-produces it later,
  but it is close enough to the re-derivable history `changes-not-commits.md`
  refuses that the boundary deserves saying out loud.
- **Two documents ingesting the same scaffold** — a monorepo where two documents
  each own part of what one `dotnet new` wrote — has no rule here, and the
  obvious one (the fingerprint is shared, the files are partitioned) is
  asserted rather than designed.
- **The volatile-region marking is a declared claim**, so it is unverifiable and
  belongs in the declared family's language rather than looking derived
  (`three-provenances.md`).
