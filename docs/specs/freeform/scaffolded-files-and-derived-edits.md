# Scaffolded Files, And Whether A Block Can Be A Set Of Edits

**Status: an investigation, not a plan.** Asked for on 2026-08-21 as
something to think about rather than build. Nothing here is implemented. The
conclusion is that the *need* is real and currently unmet, that the mechanism
proposed for it — CRDT edits — is the wrong one for a reason worth writing
down, and that the right mechanism is already three quarters present in this
repository under other names.

> "I want to think about if we can support a hick file that builds an app
> where a big part of bootstrapping the app is running a command like
> `dotnet new` or something, which generates a bunch of files; and then we
> want to modify those files. Can a code block consist of a group of CRDT
> edits that are applied on top of something else?"

## The gap, stated exactly

A literate document today declares whole files. `<hick:file path="…">` holds
every byte of what it writes, and that is what makes the guarantees work: the
weave is the file, provenance is byte-precise, `hick equiv` can compare a
restructure against a baseline, and an edit made on the generated side maps
back to a span in the document.

A scaffolder breaks that shape. `dotnet new webapi` writes forty files nobody
typed — a `.csproj`, a `Program.cs`, launch settings, a `.gitignore`, an
`appsettings.json` per environment — and the interesting work is changing
*four lines across three of them*. The two things a document can do about that
today are both bad:

- **Paste the scaffold in.** The document becomes forty files of somebody
  else's boilerplate with your four lines buried in it. Worse, it is a
  *snapshot*: the next SDK writes a slightly different `Program.cs`, and the
  document silently keeps generating the old one while claiming to be what
  `dotnet new` produces. That is the precise failure this product exists to
  prevent — an assertion about generated artifacts that has quietly stopped
  being true.
- **Leave them out.** The scaffold is a manual step in a README, and the
  document only owns the four lines. The reproducibility claim is gone: the
  document no longer describes the program.

So the missing primitive is real, and it is not a small convenience. It is
**"take that, and change it"** — a block whose content is a derivation from an
artifact the document did not write.

## What already exists that is closer than it looks

Four pieces, all shipping:

1. **Execution into volumes.** `<hick:container>` plus `<hick:exec>` can run
   `dotnet new` for real, and `<hick:volume>`'s `Output` / `InputOutput`
   kinds already model "a directory this run produced"
   (`crates/hick-exec/src/volume.rs`).
2. **Transcript caching.** Every exec's output is captured and cached
   (`hick-transcript`, `crates/hick-literate/src/capture.rs`), which is what
   lets `hick weave` produce a document's markdown on a machine that cannot
   execute anything — the phone, per `notes-ide.md`.
3. **Three-way merge, with a recorded base.** `hick-merge` already does
   exactly the operation this needs: base, generated, edited, and a strategy
   for the conflicts. It exists for the up-loop (an editor's save against a
   regenerated output) and the shape is identical.
4. **Structural navigation with no toolchain.** `hick-structure` compiles
   tree-sitter grammars into the binary, so the document can already *name a
   region of a C# file by its syntax* on a machine with no .NET installed.

Point 4 is the one that changes the answer, and it is easy to miss.

## Why CRDT edits are the wrong mechanism

The appeal is obvious and it is not naive: a Yrs update is not a positional
patch. Each insertion carries an identity, so an edit set can be applied to a
document that has moved underneath it and still land in the right place. That
is exactly the property a patch against a scaffold wants.

It does not transfer, for one reason that is fatal and structural:

**A re-run of the scaffolder does not produce a CRDT successor. It produces
fresh bytes.**

Yrs's position independence comes from *shared history*. Two replicas can
merge because both descend from the same document and their operations
reference the same item ids. `dotnet new` run twice produces two byte strings
with no shared history at all — the second is not an edited version of the
first, it is an unrelated artifact that happens to be similar. To apply
yesterday's CRDT edit set to today's scaffold you would first have to build a
CRDT document from today's bytes, and then *diff the two bases and rebase the
edits*, which is the three-way merge you were trying to avoid. The CRDT layer
would carry all of its cost and none of its benefit.

This is worth stating in general form, because the same reasoning applies to
several other tempting uses:

> A CRDT is the right structure for **concurrent editing of one object over
> time**. It is the wrong structure for **expressing a derivation from a
> foreign artifact**, because a derivation's base has no history to share.

Which is why `hick-grove` and `hickory-collab` are correct where they are —
the app's editor buffer and the file on disk *are* two writers to one object,
exactly as `AGENTS.md` says — and would be wrong here.

## What the right mechanism looks like

Not textual patches either. `patch`, `quilt`, and Debian's `series` files are
the fifty-year-old version of this idea, and their failure mode is famous: the
base moves by three lines and every hunk after it needs a human. Line offsets
are the weakest possible anchor.

The anchor should be **structural**, and this repository can already compute
structural anchors offline. Something in the shape of:

```
<hick:derive from="scaffold:src/Program.cs">
  <hick:replace select="method:Main/body">
    …the C# you want instead…
  </hick:replace>
  <hick:append select="element:PropertyGroup">
    <Nullable>enable</Nullable>
  </hick:append>
</hick:derive>
```

Four properties follow, and each one is the reason to prefer this over both
alternatives:

- **The document stays small and says what you meant.** The diff a reviewer
  reads is "the body of `Main`, replaced" — not four lines inside forty files
  of boilerplate.
- **The base is pinned and recorded, not assumed.** `from="scaffold:…"` names
  an exec whose output is already captured in the transcript, so the weave is
  reproducible offline and on a machine with no .NET — the same property that
  makes cells weavable on the phone.
- **Provenance gets *better*, not worse.** The untouched bytes of
  `Program.cs` carry `SourceOrigin::Exec` (the scaffold's), your replacements
  carry `SourceOrigin::Literal` spans in the document, and the ribbons show
  three colours in one file. That is a strictly more honest picture than
  pasting the scaffold in, where forty files of somebody else's code would
  claim to be literal document text you wrote.
- **Drift becomes a checkable event rather than a silent one.** When the SDK
  moves and `method:Main/body` no longer resolves, `hick check` says so and
  names the edit. Compare the paste-it-in approach, where the same drift
  produces no signal at all — the document simply keeps writing last year's
  scaffold. The machinery for saying it is already here:
  `hick_literate::equiv::compare_outputs` and the refactor baseline are the
  same question asked about a different pair of artifacts.

## The hard parts, honestly

**Reproducibility is the real constraint, not the syntax.** A document that
runs `dotnet new` is only reproducible if the SDK version is pinned *and* the
transcript is good enough to weave from without re-running it. The second half
is the load-bearing one: `local-only.md` and `notes-ide.md` both require that
a document renders with no execution, and a scaffold whose output is not fully
captured would be a document that only works on the machine that made it. This
is a constraint on *what counts as a scaffold*, and it should be enforced
rather than hoped for: an exec whose output volume is not fully captured
cannot be a `from=`.

**Anchors are guesses, and `hick-structure` says so.** Its resolution is by
name, and its own module doc is explicit that overloads, shadowing, and
dynamic dispatch defeat it. An anchor that resolves to two places must be a
refusal, not a coin flip — and the refusal has to name both, in the style
`user-facing-errors.md` requires.

**An unresolvable anchor must fail the weave, not degrade.** This is the one
place to be less forgiving than the CSV reader. A table that cannot be parsed
still opens because a reader wants the data; a derived file whose edit did not
apply is a *program* that is silently missing a change, which is worse than no
file at all.

**Ordering.** Edits within a `derive` must apply in a defined order against a
defined base — all against the original, not each against the last — or two
edits touching nearby regions become order-dependent in a way nobody can
review. All-against-the-original is the rule that makes a `derive` block
readable as a set rather than as a program.

**The no-escaping invariant survives, but only just.** An edit's content is
raw bytes, byte for byte, as every hick body is. So every anchor lives in an
attribute, and `select=` grows a small grammar of its own. That is acceptable;
what is not acceptable is any temptation to escape the content to make the
anchor expressible inline.

## What I would actually do, in order

Nothing yet — this is an investigation. But if it were picked up, the order
that de-risks it is:

1. **`from=` with no edits at all.** A `hick:file` whose bytes come from a
   captured exec output rather than from the document. That alone solves
   "the scaffold is in the repository and reproducible", it needs no anchor
   grammar, and it makes the provenance story concrete enough to look at.
2. **Whole-file replacement of a derived file**, so the escape hatch exists
   before the clever part does.
3. **Anchored edits, text first** — a recorded context string, resolved
   uniquely or refused. Boring, and it proves the drift-detection story.
4. **Structural anchors** through `hick-structure`, once there is something to
   compare them against.

Steps 1 and 2 are worth doing on their own merits even if 3 and 4 never
happen. That is the test of a good decomposition here.

## The short answer

Yes to "a block that is a set of edits applied on top of something else" — it
is the missing primitive, and the product is worse without it.

No to "CRDT edits" as the way to express them: a scaffolder's re-run shares no
history with its previous run, so the property that makes CRDTs merge cleanly
is absent exactly when it would be needed. The base must be *recorded* and the
anchors *structural*, and both of those are things this repository can already
do.
