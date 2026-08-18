# Provenance and standing: two axes, never one ladder

*Status: design of record for how a note says where its information came from
and how much weight it carries. Adopted 2026-08-18. Extends the lineage design
already implemented in `crates/hick-flow` (`SourceOrigin`) and
`crates/hickory-cli/src/agent_lineage.rs` (authorship from `git blame`). Pairs
with `ingest.md`.*

Two questions look alike and are not:

- **Where did these bytes come from?** Derivable. The document's structure and
  git already answer it, and nobody has to assert anything.
- **How much should I trust this claim?** Not derivable from anything. Whether
  Sam knows about Postgres indexes is a judgment a human makes about another
  human.

Collapsing them is the failure this document exists to prevent. The repository
already states the principle, about diagrams: *a drawing that appears
authoritative and is not is the exact failure this feature exists to prevent*
(`docs/guarantees/authoring/a-diagram-names-what-proves-it.md`). The same rule
applies here, and it has a rendering consequence: **derived provenance and
declared standing must never look alike.**

## Trust is two axes, not a ranking

The intuition "human-written is the gold standard" is half of this product's
existing position. Hick already has a different standard: bytes that
**reproduce**.

|  | Attributable — a named human is accountable, and git proves it | Reproducible — it re-derives, and `hick test` proves it |
|---|---|---|
| Prose someone typed | yes | no |
| `hick:exec` output | no | yes |
| A `hick:transform` passage | no | the *derivation* is pinned and checked; the prose is not |
| An ingested transcript | the speakers, if the recorder heard right | the bytes, against the source file's hash |

Neither column dominates. A number a human typed from memory is attributable
and wrong; a number a cell computed is unattributable and right. `hick:transform`
exists precisely because a model's summary starts out in neither column, and its
`from=` fingerprint drags it onto the second one — the claim is not "this prose
is true" but "this prose was written from exactly these bytes under exactly this
instruction", which is checkable offline, for free, forever.

## Axis 1 — provenance: derived, never declared

Nothing here asks the author to mark anything. It is read from three places
that already exist:

1. **`SourceOrigin`** (`crates/hick-flow/src/node.rs`) — literal, exec, paste,
   agent, variable, script. Ingest adds transcript-derived bytes to this
   vocabulary.
2. **Document structure** — a passage inside `<hick:transform>` was written by
   a model, and its `from=` says from what. Note that such prose is stored as
   *literal* text, so a reader that consults only `SourceOrigin` will call it
   human-typed. **Structure has to be consulted too.** This is the one place the
   existing lineage machinery is not sufficient on its own.
3. **`git blame`** (`agent_lineage.rs`) — who committed the span, or that it is
   uncommitted and therefore yours.
4. **The operating system's record of a download** — `kMDItemWhereFroms` on
   macOS, the NTFS `Zone.Identifier` stream on Windows, `user.xdg.origin.url`
   on the Linux browsers that set it. Nobody types it, which is what makes it
   evidence. It lands in an ingested note as `source-url:`.

   Two rules govern it. **The query string is dropped**: a Drive or S3 export
   link carries its credential in the query, and this value is written into a
   file that goes into a git repository and may be pushed to a remote somebody
   else can read — `config-and-environments` says never write a secret into a
   file we create, and a signed URL is a secret. And **absence means nothing
   was recorded**, never "this was not downloaded", because support is uneven
   and `curl` records nothing at all.

### Originator and conveyance

One label cannot describe an AI summary of humans talking. Provenance is a
pair:

| | Originator — whose information | Conveyance — how it got here |
|---|---|---|
| Typed prose | a person | typed directly |
| Scratchpad text | a person | typed directly, in the app |
| Ingested utterance | the speaker | machine-transcribed (may mishear) |
| Meeting summary | the speakers | machine-transcribed, then machine-summarized |
| Agent-cell output | a model | generated |
| Exec output | the machine | executed, reproducibly |

The distinction is the whole point of the feature. A verbatim human sentence
and a model's invented claim are both "AI-touched" under a single flag, and
they are not remotely the same thing. Two lossy steps between a person and a
paragraph is a fact the reader should be able to see.

### The asymmetry that constrains the wording

**"An AI wrote this" is provable. "A human wrote this" is not.** Anyone can
paste model output into their editor and commit it under their own name.

So every surface says **AI-touched** or **no evidence of AI**, and never
"human-written", "human-verified", or a green check. The strong wording would
be a claim the system cannot back, on exactly the question people would rely on
it for.

### File-level: has an AI ever touched this

Derived from git history — commits carrying an AI co-author trailer, and spans
whose origin is an agent cell — and **never stored in the file**. A flag the
file asserts about itself is a flag anyone can edit, which is the reason
`SourceOrigin::Agent` deliberately has no `author` field today.

Three honest limits, which the surface must state rather than imply away:

- **Squash and rebase erase it.** History rewriting is normal, and this is
  evidence, not proof.
- **"Has ever" is permanent and blunt.** A file an agent touched once, then a
  human rewrote entirely, stays flagged.
- **Outside a git repository there is no answer.** That reports as *unknown*,
  never as clean.

## Axis 2 — standing: declared, and visibly unverified

Standing is a claim about a claim, so it gets its own element and no pretence
of verification:

```
<hick:claim by="nate" standing="judgment" scope="product">
We won't build export until three people ask for it.
</hick:claim>

<hick:claim by="sam" standing="expert" scope="postgres">
A partial index is safe at this write volume.
</hick:claim>
```

- **`by`** — who is making it. In an ingested transcript the speaker is
  *derived* from the transcript and has a source; a `by=` typed by hand is an
  assertion with none. The two must not render identically, for the same reason
  as everything else here.
- **`scope`** — what it is a claim about. This is what makes `standing="expert"`
  meaningful: expertise is never global, and an expert speaking outside their
  scope is the exact situation you described wanting to catch.
- **`standing`** — a closed vocabulary:

| Value | Means |
|---|---|
| `expert` | speaking inside their own domain |
| `judgment` | a decision or opinion, explicitly *not* a claim of expertise |
| `report` | relaying what someone else said or observed |
| `assumption` | believed, unverified, and known to be |

A repository may declare additional values in frontmatter, so a team whose real
distinctions do not fit these four is not stuck. An **unknown** standing is a
**warning naming the value and the declared set** — never an error, following
the diagram rule: a tool that refused to weave would only teach people to stop
marking anything.

### Sparse by design

Most prose carries no `hick:claim` and never will. Unmarked prose is simply
unmarked, and **must not render as "unverified", "unknown quality", or anything
else that reads as a deficiency** — otherwise every note becomes a wall of
warnings, marking becomes bureaucratic, and people stop writing notes in the
tool. Mark the sentence somebody might later act on and regret; leave the rest
alone.

## What the reader sees

Whatever the app draws, three properties are non-negotiable:

1. **Derived and declared are visually distinct.** Provenance is evidence;
   standing is somebody's assertion. If they share a colour, the feature is
   worse than nothing.
2. **Compound provenance stays compound.** A summary shows both steps, not one
   "AI" badge.
3. **Absence of evidence is drawn as absence of evidence.** No green checks for
   "human".

## What is built, and what is not

Axis 2 is implemented: `hick:claim` weaves an attribution, the standing
vocabulary is closed and extensible per document, and the warnings are in
place — see `docs/guarantees/authoring/a-claim-says-who-is-asserting-it.md`.
Ingest and the derived `by=` on a `hick:said` are implemented too, as is
`source-url:` — the download-origin half of axis 1.

The scratchpad applies this document's rule at the point of entry: typed text
becomes **prose**, never a `hick:transcript`, because a transcript means "bytes
another tool produced" and would imply a machine step between the person and
the words that never happened.

Axis 1's **file-level AI-touch report is not built.** `SourceOrigin`,
`hick:transform` fingerprints, and `agent_lineage.rs`'s blame-derived
authorship all exist and predate this document, but nothing yet walks a file's
git history for AI co-author trailers and reports *AI-touched* / *no evidence
of AI*. Until that exists, this document is a design, not a description, for
that one part.

## Open edges

- **`by=` is unverified, and there is a real version that would not be.** Signed
  commits already anchor authorship for the person who *wrote the file*; nothing
  anchors an attribution *about a third party*. Whether that gap is worth
  closing is undecided.
- **Standing decays.** An expert judgment from two years ago about a system that
  has since been rewritten is still tagged `expert`. Nothing here models time.
- **Nobody has drawn this yet.** Every rendering claim above is a constraint on
  a UI that does not exist. The app currently draws lineage ribbons; how
  provenance pairs and standing sit alongside them is unspecified and is the
  most likely place this design turns out to be wrong.
- **This adds friction to writing.** `hick:claim` is a tag someone has to type
  in the middle of a thought. If it is not nearly free in the editor — a
  selection and a keystroke — it will not be used, and an unused marking system
  is worse than none because it makes the marked subset look complete.
