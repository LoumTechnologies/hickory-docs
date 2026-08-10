# Failure ledger

Dead, abandoned, or materially retreated predecessors. Cause class is
`niche` (no real demand / bad economics — avoid), `execution` (fixable; the
niche may still be live and now less crowded), or `timing` (revisit if
conditions changed).

**Read the first entry before the others.** It is the one that pre-buys the
most expensive lesson available to this product.

---

## 1. Literate programming (WEB / CWEB / noweb), 1984 → never crossed over

- **What they tried:** Knuth's original thesis — one narrative source document,
  mechanically *tangled* into compilable code and *woven* into a formatted
  document. Exactly Hickory's `hick:doc` model.
- **Niche it maps to:** `literate-verified-docs` (the core niche).
- **Outcome:** not a shutdown — a forty-year failure to achieve adoption
  outside individual devotees, while the *idea* kept being re-invented.
- **Cited causes** (retrospectives, not a single post-mortem):
  1. **Parallel maintenance burden** — keeping prose and code in sync was
     exhausting enough that most practitioners abandoned it.
  2. **Authors isolated from their own artifacts** — because both the code and
     the formatted document are *derived*, neither can be edited directly, which
     cuts the author off from the writing they actually have to do.
  3. **Weak tooling** in the early era.
  4. **Weak examples** — beyond Knuth's own small program, the published
     exemplars did not sell the idea.
- **Cause class: `execution` + `timing`.** The demand for
  "the document and the program are one artifact" never died — Jupyter
  reconstituted it at enormous scale (40M+ monthly PyPI downloads) by removing
  the tangle/weave friction and never using the words "literate programming."
- **Post-mortem links:** Bob Myers, *Whither Literate Programming (2) — What
  went wrong?*; Kartik Agaram, *Literate programming: Knuth is doing it wrong*;
  Knuth's own page at Stanford.
- **Confidence:** high that these are the *cited* causes; medium that they are
  the *true* causes (see self-report bias in the unknowables).
- **Freshness:** retrospectives span 2014–2026; the underlying failure is
  historical and stable.

**What this buys, directly (inference):** Hickory's single sharpest existential
risk is not competition — it is cause #2. If a user cannot edit the woven output
or the tangled code and have the change round-trip into the source, this product
inherits the exact friction that killed forty years of predecessors. Hickory's
`promote` (session → pipeline) and CRDT ⇄ `.hick` round-trip are the mechanisms
that would answer it. **This should be a persona probe before it is a
feature claim.** Corollary from Jupyter's success: the winning framing may be
one that never says "literate programming" out loud.

---

## 2. Distill.pub — indefinite hiatus, July 2021

- **What they tried:** a peer-reviewed journal of interactive, reproducible,
  beautifully-rendered ML research documents — the most successful realization
  of "documents that run" as a *publication*.
- **Niche it maps to:** `reproducible-research-documents`.
- **Outcome:** one-year hiatus announced 2021-07-02, explicitly "may be extended
  indefinitely"; still not accepting submissions. Not acquired, not pivoted.
- **Cited cause of death** (a real, written post-mortem — *Distill Hiatus*):
  1. **Volunteer burnout.** The editorial team was unpaid and the structural
     frictions ground them down.
  2. **The journal model itself was doubted** — the team became unsure it made
     sense to run a journal at all rather than encourage authors to self-publish.
  3. Preserving Distill as-is was judged more valuable than diluting it with
     weaker editing.
- **Notably absent from the post-mortem:** any claim that readers didn't want
  the artifact. Demand for the output was never the stated problem.
- **Cause class: `execution`** (specifically, an unsustainable labor model).
- **Post-mortem:** https://distill.pub/2021/distill-hiatus/ · HN discussion 27718054
- **Confidence:** high (first-party, explicit). **Freshness:** 2021, stable.

**What this buys:** the "beautiful reproducible document" artifact is *loved*
and still nobody has made producing it cheap. Distill died on the cost of
**human editorial labor per document**. Any Hickory pitch whose implied
workflow is "and then a careful person curates it" is walking into the same
grinder; the agent-authored-document thesis is the counter-move, and it is
worth stating in exactly those terms.

---

## 3. Observable — twice-retreated from cloud-only notebooks (not dead)

- **What they tried:** a hosted, cloud-only reactive JavaScript notebook whose
  documents lived on their platform.
- **Outcome:** **alive, but the original shape was walked back twice** —
  Observable Framework (static site generation from files), then Observable
  Notebooks 2.0 (announced 2025-07-29): an **open file format** (a dialect of
  HTML), open-source tooling, and a **macOS desktop app**, explicitly combining
  the notebook UI with Framework's *file-over-app* philosophy. Public user
  unease is visible in their own forum thread "Where is Observable going?".
- **Cause class: `timing` / positioning correction**, not death.
- **Confidence:** high for the facts; medium for the causal reading.
  **Freshness:** 2025-07/08.

**What this buys:** a well-resourced incumbent independently converged on
Hickory's format thesis — *the document is a plain file you own, in your repo,
not a row in someone's database*. That is corroboration of the thesis and
simultaneously the arrival of a competitor onto it. It is also a **dry-up
signal fired early** for any niche premised on "we're the only ones who treat
documents as files."

---

## 4. Gigantum — absorbed, April 2025, **no post-mortem found**

- **What they tried:** open-source platform for reproducible data science with
  automatic versioning of code, data, and environment.
- **Outcome:** joined Digital Science (alongside ripeta), April 2025.
  Acquisition/absorption, not a public shutdown.
- **Cited cause of death:** **none found.** Acquisition announcements are
  marketing and say nothing about whether the standalone business worked.
- **Cause class: unknown — do not infer.** Flagged as a declared unknowable.
- **Confidence:** high that the acquisition happened; **zero** on the cause.
  **Freshness:** 2025-04.

---

## 5. Netflix Polynote — quiet, **not dead**

- Polyglot notebook with an IDE-inspired design, open-sourced 2019. As of a
  maintainer answer in Dec 2024: still used at Netflix, still maintained, but
  released internally rather than publicly.
- **Recorded here so a later session does not mistake public silence for death.**
- Confidence: medium (a GitHub discussion answer, not an announcement).

---

## 6. Nextjournal — **no evidence of death; do not record as dead**

Searched specifically for a shutdown. Site live, product pages current, GitHub
org active through Dec 2025. **Finding: alive.** Logged because "I assumed it
died" is a cheap way to mis-map a niche as vacant.

---

## 7. The Semantic Web (RDF / OWL / RDFa), 1999→ — failed at scale, **with a surviving counter-case**

Added 2026-08-09 (second addendum), when the operator proposed a user-defined
taxonomy — an inline, hick-native equivalent of JSON-LD — instead of first-class
entity types in the core. That is a good instinct standing on a very large
graveyard, and the graveyard has an unusually clear lesson.

- **What they tried:** universal, user-defined, machine-readable semantics
  authored into documents, with formal-logic reasoning over the result.
- **Niche it maps to:** `user-defined-taxonomy` (the operator's proposed core
  extensibility mechanism).
- **Outcome:** widespread failure of the upper stack. Surviving use is in
  enterprise knowledge graphs, now branded "semantic enterprise standards"
  rather than "semantic web" — the name itself became a liability.
- **Cited causes** (multiple independent retrospectives, incl. practitioners
  who built the tooling):
  1. **Encoding metadata was time-consuming and error-prone.** Unpaid authoring
     work, done by hand, forever.
  2. **OWL required formal-logic training**; proponents spent their time
     explaining basics and working around the language.
  3. **Reasoners did not scale** to the web.
  4. **Killer comparison:** basic indexing and search produced similar results
     "in a much more developer-friendly way." The alternative was worse in
     theory and better in practice.
- **Cause class: `execution`** — the demand for machine-readable meaning was
  real and is now being served by other means. The authoring cost killed it.
- **Confidence:** high on the cited causes (many independent sources, including
  hostile-witness accounts from within the community).

### The counter-case that matters more than the failure

**JSON-LD + schema.org succeeded enormously** — **45M+ web domains** using schema
markup by 2024. The same idea, the same era, the opposite outcome. Cited causes
of *success*:

1. **A simpler, decoupled form.** A `<script>` block separate from the markup —
   easier to implement and maintain at scale than inline RDFa, and independent
   of how the page renders.
2. **Somebody made it pay.** Google backed JSON-LD in 2015 and recommended it
   outright in 2017; structured data bought visible search appearance. Adoption
   followed the incentive, not the standard.

**The synthesis (inference, and the most transferable finding in this ledger):**
an open taxonomy succeeds when **something forces a payoff for filling it in**,
and dies when annotation is unpaid work. Note that this is the *third
independent restatement of one constraint* in this recon:

- literate programming died of **parallel maintenance burden** (§1),
- traceability best practice is "capture evidence as a **byproduct of normal
  work** rather than asking the team to do extra work" (recon addendum A9),
- the semantic web died of **hand-annotation cost** while JSON-LD won by being
  cheap and **rewarded**.

Three different decades, three different fields, one constraint. **Hickory
already owns a forcing function none of the predecessors had: `hickory test`
can fail the build on the taxonomy.** An annotation that gates CI is not unpaid
work — it is load-bearing. That is the specific reason the operator's
user-defined-taxonomy direction is defensible where RDF was not, and it is the
same "derived, not asserted" wedge one level up.

**Design consequences the failures dictate** (inference):

- **No formal logic, no reasoner, no OWL dialect.** The literature is
  unambiguous about which layer killed it.
- **Ship an example vocabulary in a non-`hick:` namespace, never as core types.**
  Almost nobody authors an ontology from scratch; schema.org succeeded because
  someone else published the vocabulary and users only referenced it — and
  schema.org types were never part of HTML. That is exactly the namespace model:
  **the mechanism belongs in core, the vocabulary never does.** So the wedge
  ships as a forkable `.hick` document defining `requirement` / `decision` /
  `source` under its own prefix, with zero entity types reserved in `hick:`.
  Solving the blank-page problem and reserving core types are different
  decisions; conflating them is what the first draft of this note did.
- **The payoff must land in the same commit as the annotation.** JSON-LD's
  reward was a visible rich snippet. Here it is `check` going red and `trace`
  filling in.
- **Never use the phrase "semantic web."** Practitioners abandoned the term
  themselves; it now signals a decade of failure to precisely the technical
  audience being pitched.

---

## 8. HyperCard (1987–2004) — died; Access and FileMaker did not

Added 2026-08-09 (third addendum), when the operator asked whether there is a
HyperCard / MS Access / FileMaker equivalency in a user-defined-taxonomy plus
reactive-query design. There is, and the dividing line in that history is not
the one people usually assume.

- **What it was:** an end-user authoring environment — stacks, cards, buttons,
  scripts — on the premise that regular people can build software.
- **Outcome:** killed quietly after Jobs' 1997 return, in a strategy of fewer,
  more focused products.
- **Cited causes:**
  1. **Distribution, not technology.** HyperCard 2.x moved from *free with every
     Mac* to a **$49.95 standalone product**, fragmenting the user base — older
     Macs had 1.x, new users had to choose to buy. The free-distribution model
     that made it ubiquitous in 1987–1990 was broken by its own pricing change.
  2. **No network story.** Atkinson's own stated professional regret: *"I grew
     up in a box-centric culture at Apple. If I'd grown up in a network-centric
     culture, like Sun, HyperCard might have been the first Web browser."*
  3. Portfolio cull — it did not fit the iMac/iPod/OS X Apple.
- **Cause class: `execution`** (packaging and distribution), with a `timing`
  component (missed the network).
- **Confidence:** high; the pricing history and Atkinson's regret are both
  widely and consistently reported.

**The survivors, checked rather than assumed:** **Claris FileMaker** shipped a
2026 release, describes itself as forty years of custom business solutions, and
is adding AI features. **Microsoft Access** is still in use and still rated
well. The end-user-database category did not die — *HyperCard specifically*
died, and largely of packaging.

### The line that actually matters (inference — see caveat)

Sorting that history by "did it have a reactive dependency graph" gets the wrong
answer. Sorting it by **"did it build applications"** gets the right one:

| | Reactive derived values | Application builder | Outcome |
|---|---|---|---|
| **Excel** | yes — cells hold formulas over other cells, single-direction dataflow, changes propagate automatically | **no** | The most widely used end-user programming environment ever built |
| HyperCard | limited | **yes** — stacks, cards, buttons, UI | Died |
| Access / FileMaker | limited | **yes** — forms, reports, UI | Alive, but as *application platforms* with the sales and support motion that implies |

**A reactive dependency graph is not what makes a tool low-code. A view/form
builder is.** Excel is a reactive dataflow engine with no application layer and
it is the success story of the entire category; the tools that grew UI builders
became application platforms, which is a different business with a different
motion.

**Consequence for this product:** a standing query that maintains a derived list
inside a document is a **formula, not a form** — the Excel side of the line. The
low-code drift risk lives in the *view* layer, not the *query* layer, and the
guardrail should be drawn there. Note that Palantir illustrates the drift
precisely: its ontology sits under Workshop applications, dashboards, and
write-back workflows (`paygate-teardowns.md`).

**Caveat, stated because it matters:** no source was found that directly
compares Excel and HyperCard on this axis. The table above is **inference** from
two separate bodies of evidence, not a cited finding.

**Second consequence, from cause #1:** HyperCard died partly by moving from free
to paid and fragmenting its own base. That is a direct warning for the open-core
plan in `personas/oss-maintainer.md` — the free tier *is* the distribution, and
moving something load-bearing behind the paywall is how the base fragments.

---

## The absence that matters most

**No company attempting exactly "commercially-sold, executable, CI-verified
documentation" was found to have died.** That has two readings and this ledger
refuses to pick one:

- **(a) Whitespace.** Nobody died there because nobody has seriously tried it as
  a commercial product; the free OSS tier (doctests, Quarto, Runme) and the
  free hosting tier (Mintlify) sandwich an unoccupied middle.
- **(b) Empty room.** Nobody died there because there is no buyer to die for,
  and the two nearest attempts that *did* die — literate programming and
  Distill — both died of the labor/friction cost of producing the artifact,
  which is a cost this product also has to pay.

The evidence available to external research cannot separate these. **A cheap
probe can**: the discovery landing page already exists and has never been driven
by traffic. That is the correct next spend, not more desk research.
