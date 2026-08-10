# Persona — Delivery owner (tech lead / eng manager / PM who owns "did we build what we said")

> **Unvalidated hypothesis.** Identity has not been through the brain's gate and
> this cohort has never been observed. Only the recon-owned sections below are
> populated. See `personas/README.md`.
>
> Added 2026-08-09 (recon addendum) after the operator observed that the first
> five cohorts skipped the person who sits between the engineer who feels the
> pain and the auditor who signs off. **This cohort is the reachable one** — it
> reads Hacker News, lives in Jira, and does not require a procurement cycle.

## Split hypothesis (recorded, not acted on)

This persona plausibly contains **two** cohorts separated not by demographics but
by whether a **contract counterparty** exists:

- **(a) Internal delivery owner** — a product org. The artifact's job is
  *organizational memory*: why was this built, who approved the trade-off.
  Nobody outside the company ever reads it. WTP is modest and discretionary.
- **(b) Contract-bound delivery owner** — agency, consultancy, gov contractor,
  or a regulated in-house team. The artifact's job is **evidence for a
  counterparty**: proving the deliverable meets the spec, at acceptance or in a
  dispute. WTP is attached to the size of the contract, not to a tooling budget.

Per persona discipline these are **not split until evidence splits them**. The
split trigger: if a discovery run produces two clusters whose `interest_expanded`
patterns diverge on the compliance/evidence sections versus the memory/docs
sections, fork this file and divide the learnings log by evidence.

**The distinction matters more than usual here** because (b) is where the money
is and (a) is where the distribution is — the same tension the OSS core already
manages.

## Alternatives set — what they use today

*Recon 2026-08-09.*

| Alternative | Kind | Note |
|---|---|---|
| **Jira + Confluence + a hand-maintained link table** | Generic substitute | **The true incumbent.** Reported pain: "traceability requires more than issues — multi-level links between items, documents, tests, and source," which Jira does not natively give |
| **An Excel spreadsheet** | Generic substitute | Not a figure of speech: GSA's official M3 Playbook **Requirements Traceability Matrix template is an `.xlsx`**. The US federal government's reference implementation of this job is a spreadsheet |
| **AWS Kiro** | Direct rival, new and funded | Spec-driven IDE, launched internationally 2026-05-07 as the replacement for Amazon Q Developer. Requirements (EARS notation) → design → dependency-sequenced tasks, with **requirement-to-task and test-to-requirement traceability**. $20 / $40 / $100 / $200 per month |
| **GitHub Spec Kit** | Direct rival, free | MIT-licensed, agent-agnostic CLI. `constitution → specify → plan → tasks → implement`; **specs persist as editable repo artifacts rather than a throwaway chat plan**, and `tasks.md` keeps requirement-to-task traceability |
| BMAD-METHOD, OpenSpec, Augment Cosmos, `.cursor/rules` | Direct rivals | The rest of the 2026 spec-driven cohort |
| Jama Connect · Polarion · codebeamer · DOORS Next | Direct rival (heavyweight) | Where (b) ends up when a regulator is involved. No public pricing |
| ADRs (architecture decision records) in the repo | Generic substitute | Free, text, widely adopted, and genuinely good at "why was this built" — and completely unverified |
| A well-written PR description | Generic substitute | The honest default for (a) |
| **Do nothing** | Do-nothing | Rely on the memory of whoever was in the room. Works until they leave or the client disputes an invoice |

### The seam, stated precisely (observation + inference)

**Observation:** every alternative above — Kiro, Spec Kit, Jama, an RTM
spreadsheet, an ADR — records traceability as a **link that a human or an agent
asserted**. Kiro's own claim is that each test is "traceable to a requirement in
`specs/SPEC.md`"; Spec Kit's is that `tasks.md` links tasks back to
requirements. Nothing in that list **re-executes anything to prove the link
still holds.** DO-178C DAL A demands full bidirectional traceability across four
levels and the literature describes producing it as "labor-intensive and
susceptible to human error without appropriate tools."

**Inference:** the differentiator is not *having* a trace — the market now has
several free ways to have one. It is that the trace is **derived and
re-provable** rather than asserted, so it cannot silently rot when someone
forgets to update the matrix. That is a one-sentence pitch and it holds against
every alternative in the table.

**The boundary of that pitch, stated so it isn't oversold:** Hickory's provenance
is mechanical from *executed inputs* to outputs. The link from a **meeting note
or a client decision to a requirement** is still an assertion, exactly like
everyone else's. The honest claim is "the requirement→code→test→output half is
derived; the human-intent→requirement half is asserted, but versioned and
diffable in the same document." Selling more than that invites the first
technical evaluator to find the gap.

## Watering holes

*Recon 2026-08-09. Medium confidence — general knowledge except where cited.*

| Channel | Norms |
|---|---|
| Hacker News | Reachable here, unlike `compliance-provenance-enterprise`. "Spec-driven development" is an actively contested HN topic in 2026, which means an opinionated post has a live audience and a hostile one |
| r/ExperiencedDevs, r/engineeringmanagers, r/agile | Skeptical of process tooling by default; a post framed as "we stopped maintaining a traceability spreadsheet" outperforms a product launch |
| Lobsters | Invite-only, authored-by tag, low marketing tolerance |
| LinkedIn | Where cohort (b) actually is — agency and consultancy delivery leads live here, not on Reddit |
| Spec-driven-development discourse (Kiro/Spec Kit communities, dev.to, Medium) | A fast-moving 2026 conversation with no settled winner. Participation is cheap and the vocabulary is being set right now |
| Agile/delivery conferences, INCOSE for the regulated end | Slow, relationship-driven; relevant only to (b) |

## Mistaken-identity set

*Recon 2026-08-09. This cohort's mistakes are the most consequential of any
persona in the library, because two of them are things they may already be
paying for.*

| They'll think it's… | Why they'd reach for it | Why it falls short |
|---|---|---|
| **AWS Kiro / GitHub Spec Kit** | Nearly identical vocabulary: specs as repo artifacts, requirements traced to tasks and tests | **The closest comparison in the whole library, and it must be answered head-on.** Those generate a trace and *assert* it; the link is a line of markdown an agent wrote. Nothing re-runs to confirm the code still satisfies the requirement — so the trace degrades exactly like the spreadsheet it replaced |
| Jama / Polarion / codebeamer | "Traceability" is their word | A matrix of human-maintained links. Silently wrong the moment someone forgets to update it — the failure mode mechanical derivation removes |
| An RTM spreadsheet | It's what the contract asks for, literally (GSA ships an `.xlsx`) | It is a snapshot of claims with no relationship to what the build actually does |
| ADRs / a good PR description | "We already record why" | Prose about a decision, disconnected from the artifact. Nothing checks that the code still reflects it |
| A Jupyter/notebook thing | Cells, prose, execution | Reasonable for the OSS surface; misleading for this job. The artifact here is a versioned document that fails CI, not a session |
| Palantir Foundry's Ontology | If the extensible-taxonomy framing is ever said out loud, this is the comparison a senior technical person reaches for | Palantir's ontology is wired into operational systems at $500k–$2M/yr with implementation matching subscription. Scoping it to a git repo is a different product at a different price — and in this cohort's watering holes the name itself is a liability. See `docs/market/paygate-teardowns.md` |
| "Docs, but tested" | The product's own earlier framing | **Undersells it to this cohort specifically.** This persona does not buy documentation; it buys defensible answers to "did we build what we said, and can you prove it" |

**Objection to expect first:** *"How is this different from Kiro?"* — and after
that, *"who maintains the requirement text?"* Neither has been tested on a real
human.
