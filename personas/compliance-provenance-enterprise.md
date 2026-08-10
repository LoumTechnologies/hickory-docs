# Persona — Enterprise needing auditable lineage, meeting note → ticket → shipped byte

> **Unvalidated hypothesis.** Identity has not been through the brain's gate and
> this cohort has never been observed. Only the recon-owned sections below are
> populated. See `personas/README.md`.
>
> Added to scope by the operator, 2026-08-09. Distinct from the other four in a
> way that matters: **its demand driver is statute with published dates**, not a
> vendor's claim about what buyers want. That makes it the only cohort here
> whose *interest* is verifiable by desk research — while its *willingness to
> pay* is the least visible, because this market quotes rather than publishes.

## Alternatives set — what they use today

*Recon 2026-08-09. Note that this cohort's alternatives share almost nothing
with the other four cohorts' — no overlap in vendors, channels, or vocabulary.*

| Alternative | Kind | Note |
|---|---|---|
| Jama Connect · Siemens Polarion · PTC codebeamer · IBM DOORS Next | Direct rival (traceability) | The requirements-traceability incumbents in regulated development (IEC 62304, DO-178C, ISO 26262). **No public per-user pricing found for any of them** — entirely sales-quoted |
| Vanta · Drata · Secureframe | Direct rival (GRC) | Attest to *controls and policies*. Widely deployed, and the reflexive answer to "how do we prove compliance" |
| Jira + Confluence + a manually maintained traceability matrix | Generic substitute | **The true incumbent.** A spreadsheet or Confluence table where humans type links between tickets, requirements, tests, and commits |
| SBOM / supply-chain provenance (SLSA, in-toto, Sigstore, artifact repos) | Generic substitute | Covers *components entering the build*. Genuinely mechanical, and the direction CRA guidance already points |
| Code Ocean v4 | Adjacent rival, converging | Pitches "agentic workflows that generate reproducible, **compliant** results." Coming at this intersection from the research side |
| Norm AI and peers | Adjacent rival, funded | $120M Series C (July 2026) for regulatory-compliance agents whose pitch is a clear audit trail |
| An auditor, a screenshot, and a quarterly scramble | Do-nothing | What most organizations will still be doing on 2026-09-11 |

### Why the incumbents leave a seam (observation + inference)

**Observation:** ALM/traceability tools maintain requirement→test→commit links
as *records humans create and maintain*. GRC tools attest to policies and
controls. Supply-chain tooling covers components entering the build. **None of
them executes the work product or derives lineage mechanically.**

**Inference:** the auditable claim Hickory can make — *"this byte came from that
source, and re-running the document proves it"* — is a different **kind** of
evidence from a link somebody typed into a matrix. Whether an auditor accepts
mechanically-derived lineage in place of a maintained matrix is **completely
unknown** and is the single highest-value question about this cohort. Desk
research cannot answer it; one conversation with a compliance lead can.

## Regulatory clock (dated, high confidence)

| Instrument | Date | Obligation |
|---|---|---|
| EU Cyber Resilience Act | in force **2024-12-10** | — |
| EU AI Act Art. 12 | **2026-08-02** | Automatic event logging across the lifetime of high-risk AI systems |
| EU CRA | **2026-09-11** | Vulnerability/incident reporting within 24h; provenance tracking of artifacts; audit trails backing the technical documentation |
| EU CRA | **2027-12-11** | Full application to all products with digital elements on the EU market |

## Watering holes

*Recon 2026-08-09. **The finding here is the mismatch**, and it is the most
actionable single fact about this cohort.*

**This persona is not on Hacker News, not on Reddit, and not in Write the Docs
Slack.** Every channel that works for the other four cohorts fails for this one.

| Channel | Norms |
|---|---|
| LinkedIn | The actual professional network of this cohort. Long-form regulatory explainers circulate here and nowhere else |
| ORCWG (Open Regulatory Compliance Working Group) · OpenSSF | Where CRA implementation is being worked out in public. **Participation-first**; a vendor arriving to sell is dead on arrival, a vendor contributing to guidance is not |
| INCOSE, industry-specific bodies (medical device, automotive, aerospace) | Conference- and standards-driven, multi-year relationships |
| Auditor and consultancy relationships | The real distribution channel. Auditors recommend what they know how to audit — which is itself an argument for engaging them before building for them |
| Compliance-focused newsletters and law-firm client alerts | How this cohort learns what it must do. Not a place to post; a place to be cited |

**Consequence (inference):** entering this niche requires a channel the product
does not currently have, and contradicts the stated go-to-market
(`pricing-strategy.md`: "self-serve, bottom-up, no sales team"). The only route
compatible with that plan is **land-and-expand** — engineers adopt it for
verified docs and provenance, then drag it into a compliance review. That is a
sequencing claim, and it should be tested on the engineer, not the auditor.

## Mistaken-identity set

*Recon 2026-08-09.*

| They'll think it's… | Why they'd reach for it | Why it falls short |
|---|---|---|
| Jama / Polarion / codebeamer | "Traceability" is their word for this | Those trace *records humans type*. If someone forgets to link a commit to a requirement, the matrix is silently wrong — the exact failure mode mechanical derivation removes |
| Vanta / Drata | "We already have a compliance platform" | GRC attests to controls and policies, not to how a byte in the product came to exist. Different evidence, different question |
| An SBOM / supply-chain tool | CRA's provenance language points straight at it | SBOMs cover components *entering* the build. They say nothing about the derivation from a decision to the artifact it produced |
| A wiki or Confluence with better search | "It's documents, in a repo" | The documents execute, and the lineage is derived rather than asserted. A wiki records claims; this produces evidence |
| An audit-log / SIEM product | "Audit trail" collides exactly | Those log *access and events*. This records *derivation* — which output came from which input, reproducibly |

**Objection to expect first, from the most senior person in the room:** *"Will an
auditor accept this?"* Nothing in this recon answers that, and no amount of
further desk research will.
