# An edit that knows what it was

*Status: proposal, 2026-08-26. Not built. Two ideas arrived separately —
provenance-preserving formatters that run on save, and a syntax- and
symbol-aware CRDT that survives a JetBrains-grade or model-driven refactor —
and they are one idea at two speeds. Both exist because every tool that
rewrites a file today throws away the only thing this product needs from it.
Depends on `provenance-across-versions.md` (the correspondence journal and
its precision ladder), `an-index-beside-the-language-server.md` (SCIP is in
addition to LSP, never in place of it), `three-provenances.md` (derived and
declared must never render alike), and `agent-cells.md` (the agent's only
write path is the tool surface).*

## The one sentence

> **A tool that rewrites a file returns bytes. It should return bytes and a
> map.**

`rustfmt`, `prettier`, `black`, `eslint --fix`, a codemod, a JetBrains
extract-method, and a model rewriting a file are all the same shape: text in,
text out, and the correspondence between the two — which the tool held in its
hands the entire time — discarded on the way out. Everything downstream then
tries to reconstruct it: `git blame` guesses with a line heuristic, a merge
guesses with a diff, this product's pre-commit repair guesses at line
precision, and a sequence CRDT does not guess at all because it never knew
there was anything to know.

`provenance-across-versions.md` already named the fix in general —
*record the correspondence at the moment both sides are in hand* — and listed
the sites where the tool holds both sides: refactor mode, the up-loop, a
merge, a re-scaffold, the pre-commit hook. **A formatter is that same moment,
happening a hundred times a day, and nothing is standing there to catch it.**
So is a rename. So is an agent's edit.

## Two standalone tools, and why they are two

Both must survive this product, so both are ordinary tools with ordinary
users who have never heard of a `.hick` document. Working names, provisional:

| | **`throughline`** | **`grain`** |
|---|---|---|
| What it is | a rewrite protocol, a library, and a CLI that wraps existing formatters and fixers | a syntax- and symbol-aware layer over Yjs/Yrs |
| The unit | one rewrite of one file, at rest | concurrent edits, in flight |
| What it produces | `(bytes, correspondence, precision)` | convergent text **plus** the intents that produced it |
| Who wants it without this product | anyone whose blame is ruined by a format commit; anyone whose CI cannot tell a format from a fix | any collaborative code editor, any pair-with-an-agent tool |
| Licence | MIT | MIT |

They are two because their failure modes are different: `throughline` is about
a rewrite that already happened, and `grain` is about two rewrites that have
not finished happening. `grain` consumes `throughline`; nothing goes the other
way.

---

# Part one — `throughline`: a rewrite that keeps its thread

## The finding that makes it feasible

The obvious plan is to write provenance-preserving formatters, which means
rewriting `prettier` and `black` and `rustfmt`. That is a decade of work and
it will never catch up.

The plan does not need it, because of a property formatters already have:

> **A formatter does not change the tree. It changes the trivia between the
> tree's leaves.**

Parse before and after with the same grammar, discard trivia, and the two
trees are **isomorphic** — same shape, same leaf tokens, in the same order.
An isomorphism between two trees is a bijection between their leaves, and a
bijection between leaves is a **byte-precise correspondence**, recovered from
a black-box formatter that cooperated in no way at all.

Which means `throughline` wraps tools rather than replacing them, and gets
byte precision anyway. Two consequences fall out for free:

- **If the trees are *not* isomorphic, the tool did more than format.** That
  is a finding worth reporting on its own — an `eslint --fix` that changed
  behaviour, a codemod that dropped a call, a "formatter" plugin doing
  something it should not. Today that lands in a diff nobody reads because it
  is 400 lines of reindentation. `throughline` separates the two piles.
- **The same machinery answers the general case.** Given any before and after
  bytes — a model's whole-file rewrite, a hand edit, a patch from a stranger —
  tree alignment recovers the best correspondence available and *labels how
  good it is*. That is one verb, `recover`, and it is the single place
  matching logic lives. A second mechanism for "which span became which" would
  be a second answer to one question, which this product refuses elsewhere.

## Trivia is per-grammar, and pretending otherwise is the bug

"Ignore whitespace" is false in Python, where indentation is a token; false in
Markdown, where two spaces are a line break; false in a heredoc, a template
literal, a docstring, and a Go struct tag; and false in every `.hick` document,
where **almost every byte is raw text the parser must not touch**.

So the isomorphism check is **configured per grammar**, not assumed, and each
configuration says which node kinds are trivia and which leaves compare by
content. A grammar with no configuration gets **line precision and says so** —
it does not get an optimistic byte-precise answer.

Say "**no evidence the tree changed**", never "semantics preserved". Tree
isomorphism is not semantic equivalence: a macro, a preprocessor, a
reflection-by-name lookup, or a string that is really code all escape it. It
is the same distinction `provenance-and-standing.md` draws between *derived
and checkable* and *declared and unverifiable*, and the same reason this
product says "AI-touched" rather than "human-written".

## The precision ladder, which already exists

`provenance-across-versions.md` gives correspondences a precision recorded by
the site that captured them. `throughline` adds rungs at the top of the ladder
rather than inventing a scale:

| Rung | Site | Precision |
|---|---|---|
| The tool told us | a formatter or refactor engine that emits a map natively | byte |
| We invoked the verb | rename/extract/move performed through a refactor engine we drove | byte |
| The trees are isomorphic | any black-box rewrite, verified after the fact | byte |
| The trees align but differ | a real change: a codemod, a model's edit | **node**, a new rung |
| No grammar, or no alignment | Markdown, a binary, a rewrite of everything | line, or diff |
| Nobody was watching | an edit outside every site | guess, or shrug |

**Node** is the one new rung and it earns its place: it is more than a line
match and less than a bijection, and collapsing it into either would lie in
one direction or the other.

## What this replaces in pre-commit and CI, stated narrowly

`throughline check` replaces **the formatting check and the autofix pass** —
`cargo fmt --check`, `prettier --check`, `black --check`, `eslint --fix` — and
becomes the one thing hooks and CI both call, which is the parity rule
(`pre-commit-ci-parity.md`) satisfied by construction rather than by keeping
two lists in step. It runs the same code path the IDE runs on save; that is
the point of it existing at all.

**It does not become a linter.** Diagnostics stay where they are: clippy,
`eslint`, the language server, `cargo check`. A tool that formats and also
opines is two tools that cannot be disabled separately. What `throughline`
adds to CI that no existing linter offers is the second pile: **this autofix
changed the tree**.

The check has three outcomes, and they are not two:

1. **Already canonical** — nothing to do.
2. **Would be rewritten** — the same failure `--check` gives today, plus the
   map, so the reviewer can see the rewrite is trivia and stop reading.
3. **A tool rewriting this file changed the tree** — a different failure, and
   the interesting one.

## Inside a `.hick` document, formatting is a reverse edit

A document's code is not in a file. It is in fragments, and the file appears
when something weaves it. A fragment is frequently not parseable on its own —
half a function, a loop body, one arm of a template — so **fragments cannot be
formatted in isolation**. The only well-formed act is:

> **Weave. Format the woven file. Map the format back through lineage. Apply
> the reverse edit.**

Which is not new machinery: it is `an-output-edit-lands-in-its-document.md`,
already built, with a formatter standing where the person's cursor usually
stands. The composition is `lineage ∘ throughline`: output byte → source span,
composed with old output byte → new output byte. Both are exact, so the
composition is exact, which is the property `provenance-across-versions.md`
already claims for recorded correspondences.

Three sharp edges, and each has an answer that refuses rather than guesses:

- **A reformat may move bytes across a fragment boundary.** A formatter that
  joins two lines contributed by two different `hick:copy` fragments produces
  a byte with two parents. **Refuse it**: report it the way a conflict is
  reported, name both fragments, and leave the document alone. A format that
  silently reassigns bytes between fragments is exactly the failure the
  `from=` refusal and the line-offset-patch refusal were both about.
- **A generated file refuses an edit**, including this one. Format-on-save of
  a woven file is only ever the reverse edit above; a formatter writing
  directly into `.hick-cache/` is a write that regenerates over itself.
- **A `.hick` document's own formatter may not touch a raw byte.** The
  no-escaping invariant means the only formattable surface is the indentation
  of namespace-prefixed tags, which makes the hick formatter nearly a no-op
  and that is correct. Do not invent a prose reflow.

## Format on save is an act, not a keystroke

Formatting on every save is a latency problem, a churn problem, and — in a
shared session — a fight. So:

- **On explicit save and at commit time, never on a sync tick.** The CRDT
  autosaves; that is not a save gesture.
- **One act in local history.** `local-history.md` says the unit is the act
  because almost every writer here is a batch writer, and a format is the
  purest batch write there is: one undo, not four hundred.
- **In a shared session it is an intent, not a diff** — which is part two.

## Normalized merge, which the map makes newly possible

With an exact map in both directions, a three-way merge can be computed in
canonical space and **projected back into each side's own formatting**. Whole
classes of whitespace conflict stop existing, and `hick-merge` is the place it
would live.

The hazard is the obvious one: normalizing rewrites bytes nobody asked to
rewrite. So normalization is **for the purpose of computing the merge**, the
result is projected back, and a side that was not canonical before the merge
is not canonical after it. If that projection cannot be made exactly, the
merge falls back to today's behaviour rather than helpfully reformatting
someone's file during a conflict resolution.

---

# Part two — `grain`: intents beside the text

## What a sequence CRDT actually promises

Yjs guarantees **convergence**: everyone ends up with the same bytes. It says
nothing about whether those bytes compile, and the canonical failure is not a
conflict on either side:

> One person renames `orders` to `invoices` in thirty places. Concurrently,
> another writes a thirty-first use of `orders`. Both merges are clean. The
> program is broken, and no tool anywhere reported anything.

That is not a bug in Yjs. It is the boundary of what a sequence CRDT is for,
and the same boundary the JetBrains merge crosses when it re-applies a
refactoring to the other side instead of merging its text.

## What does *not* work, said first

**Do not replace the text CRDT with a tree CRDT.** Code spends most of its
life un-parseable — every character of every identifier being typed is an
invalid tree — so a tree CRDT over source has to represent a state its own
model forbids. Every structured code editor has died on this. The finding is
the same one the index/LSP question produced, in a different costume:

> **The syntax layer is in addition to the text, never in place of it.**

The text sequence CRDT stays the ground truth and keeps its convergence
guarantee untouched. `grain` is a layer above it that is allowed to be absent,
stale, or wrong without anything breaking — the same standing an index has.

## The mechanism: replicate the intent, re-execute after merge

A refactor is not thirty text edits. It is **one operation that produced
thirty text edits**, and the operation is small, nameable, and replayable:

```
rename(symbol=scip://…/orders#, from="orders", to="invoices")
extract(range=…, into="fn normalize_ref")
move(symbol=…, to="billing/refs.rs")
```

`grain` replicates the operation **beside** the text edits it produced. On
merge, the text merges as it does today — unchanged, uninvolved, still
convergent — and then the intent layer runs a **repair pass**: re-execute the
operation against the merged text and see whether it still holds. In the
example above, the thirty-first `orders` gets renamed, or a proposal to rename
it is raised.

The invariants that keep this safe are the ones this product already uses for
caches and indexes:

- **The text converges first and alone.** The repair never withholds, delays,
  or alters convergence. Turn `grain` off and you have Yjs, exactly.
- **Re-execution is deterministic and confined**, or it does not run. Same
  inputs, same result on every peer, or the operation is downgraded to a
  proposal a person acts on.
- **A repair that cannot be made exactly is a proposal, never a write.** The
  ladder from part one applies: a rename resolved through a live language
  server is byte-precise and derived; a rename resolved by name through
  `hick-structure`'s tree-sitter pass is a **guess and is labelled one**, which
  that crate already does.
- **Never require the index.** SCIP widens what a rename can see across
  documents and repositories; with no index the operation is narrower and says
  so. Nothing stops working.

## Symbol identity is the hard part, and it is bought, not invented

An operation naming `orders` textually is worthless. It has to name a symbol,
and symbol identity across files is what SCIP exists to produce and what a
language server produces live. `grain` takes both and takes the live answer
when they disagree — `an-index-beside-the-language-server.md`'s rule, applied
unchanged. Where neither exists, `grain` falls back to structural name
matching and every correspondence it records carries that precision.

This is also where `grain` inherits an unresolved decision rather than making
one: **the `scip` crate would be linked**, Apache-2.0 against an MIT-only
heading. Nothing in this plan settles that, and both exits — read the Protobuf
with our own schema, or amend the heading to say what the rule says — are
still the exits.

## The model: give it verbs, do not ask it for claims

The open question was whether the agent's edit tool changes, or whether the
model emits provenance claims when it refactors. **Not the second.** A model
asserting "these thirty edits were one rename" is *declared* provenance —
forgeable, unverifiable, and required by `three-provenances.md` to render as
an assertion in dashed grey. Recording a model's opinion as a derived fact
would collapse the one distinction that document exists to protect.

The first, and in a specific way: **add refactor verbs to the tool surface.**
The agent's only write path is already the tool surface (`agent-cells.md`) and
the harness — not the model — already writes `<hick:read>` and `<hick:wrote>`.
A `rename` tool follows the same shape: the model asks for a rename, **the
harness performs it** through the language server, and the correspondence is
**derived by the harness from what it did**. The model supplies intent; the
tool supplies the fact. That is the existing pattern, not a new one.

Then the incentive, honestly stated: a model that renames with `edit` instead
of `rename` gets a coarse correspondence, and nothing can force it not to.
That is fine and it is already the designed fallback — `throughline recover`
gives node precision, and the agent-guess-plus-human-confirmation path in
`provenance-across-versions.md` is what catches the rest. **The whole-file
rewrite is the real enemy**, and the highest-value single change here is
making it the last resort rather than the default gesture.

---

## Refusals

- **Never claim semantics are preserved.** The claim is *no evidence the tree
  changed*, under a per-grammar trivia configuration, and the wording says so.
- **Never fork or vendor a formatter.** Formatters are **spawned**, like
  language servers, debug adapters and indexers, from the same sandboxed
  catalogue with the same "your own copy wins" discovery — `throughline`
  wraps, verifies, and maps.
- **Never let the syntax layer gate convergence.** Text merges first, alone,
  and the repair is allowed to fail.
- **Never present a recovered correspondence as a recorded one**, or a
  name-matched rename as a resolved one.
- **Never write a correspondence for a rewrite the tool refused.** A refused
  cross-fragment format records nothing; a half-applied act is worse than an
  unwatched one.
- **Never make either tool required.** Format-on-save off, no `grain`, no
  index, no key — the product works, with the coarseness it has today.
- **Never add a server, an account, or telemetry to either**, including in
  their standalone lives. A tool that runs over private repositories earns
  that by talking to nobody.
- **Copyleft blocks linking, not spawning.** Check each crate that would be
  linked — the `scip` reader, any parser generator, any formatting library —
  against the rule, not the project's headline licence.

## Sequence

1. **`throughline recover`** — before and after bytes in, correspondence and
   precision out, over the grammars `hick-structure` already compiles in
   (Rust, Python, JavaScript, TypeScript). No formatter integration at all.
   This is the whole substrate and the cheapest thing to be wrong about.
2. **The isomorphism check**, with per-grammar trivia configuration and the
   third CI outcome — *a tool rewriting this file changed the tree*. Usable by
   strangers at this point, on any repository, with no other part of this
   built.
3. **`throughline fmt`** — wrap the installed formatters, one catalogue entry
   per language, and make it the one thing pre-commit and CI call.
4. **Format-on-save in the IDE, as a reverse edit**, with the cross-fragment
   refusal, one act in local history, and the correspondence written to the
   journal — which requires no new journal machinery, only a new writer to it.
5. **Refactor verbs on the agent's tool surface**, harness-derived
   correspondences, starting with `rename` because it is the one whose
   correctness is checkable.
6. **`grain`** — intents beside the text, and the repair pass — last, and only
   once several sessions have produced the clean-merge-broken-build failure
   often enough to show what the repair should actually do.
7. **Normalized merge**, if the map turns out to be trustworthy enough to
   project back. It is the most valuable and the easiest to regret.

## What would kill this

- **Isomorphism is rarer than claimed.** If real formatters routinely change
  the tree — Python's `black` and its magic trailing comma, `prettier` on
  JSX — then the byte rung is mostly unreachable and this degrades to a
  slightly better line matcher. **Measure it before step 3**, on this
  repository, and write the number down.
- **Per-grammar trivia configuration does not converge.** Four grammars is a
  weekend; thirty is a maintenance surface with no end, and every wrong entry
  produces a confidently wrong byte-precise map. A wrong map is worse than no
  map, which is the argument for defaulting an unconfigured grammar to line
  precision and leaving it there.
- **Re-execution needs a language server that is not running.** The repair
  pass on a peer with no toolchain — a phone, a fresh clone, CI — has no way
  to resolve a symbol. The proposal-not-write rule covers it, but if that is
  the common case then `grain` is a feature of one machine in the fleet and
  should be described as one.
- **Nobody wants the second pile.** If *a tool changed the tree* fires only on
  real bugs twice a year, it is a correct feature with no audience, and
  `throughline` is then a formatting runner that happens to keep blame intact
  — still worth having, worth much less.
