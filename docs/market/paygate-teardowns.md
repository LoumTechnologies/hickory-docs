# Competitor paygate teardowns

All entries dated **2026-08-09** unless noted. Prices are **list** prices —
realized price after annual and enterprise discounting is private (see the
declared unknowables in `recon-2026-08-09.md`).

## Source-quality warning

A first pass on search returned a wall of `"<vendor> pricing 2026"` pages
published by *competing vendors* (Docsie, Julius, BunnyDesk, FeatureOS,
Featurebase). Those pages are marketing, and at least one was materially wrong:
Docsie's page reported ReadMe at **$79 / $349 / $3,000+**, while ReadMe's own
pricing page reports **$0 / $250 / $3,000+**. Every price below marked
*primary* was read off the vendor's own page; the rest are marked and carry
lower confidence.

---

## Mintlify — *primary*, mintlify.com/pricing

The most important teardown in this file, because it repriced the category.

| Plan | Price | Notable |
|---|---|---|
| Starter | **$0/mo** | 5 editor seats, web editor, git sync, search, auth, custom domains, MCP server, API playground. **10,000 AI credits/mo**, overage **$0.01/credit** |
| Pro | **$0/mo base** | Unlimited editor seats; adds Agent, Assistant, Automations, preview deploys, admin APIs. Committed-volume credit pricing |
| Enterprise | Custom | SSO, SCIM & RBAC, performance SLA, advanced insights, audit logs, white labeling, migration support |

- **Value metric: AI credits consumed.** Not seats, not projects, not pages.
  Seats were *removed* as a metric — Pro is unlimited-seat at $0 base.
- **What is free:** essentially the entire documentation-hosting product.
  Hosting, custom domains, git sync, search, and auth are no longer a paygate
  anywhere in this category.
- **What is gated:** agentic AI usage (metered), and the enterprise governance
  bundle (SSO/SCIM/RBAC, audit logs, SLA, advanced analytics).
- **Gap left open:** nothing in this catalog verifies that a documented example
  still runs. Mintlify generates and serves docs; correctness of the examples is
  the author's problem.
- **Context:** $45M Series B at a $500M valuation, April 2026, led by a16z and
  Salesforce Ventures (total raised $67M), ~20,000 companies. Stated thesis:
  become "the knowledge layer that makes products understandable, usable and
  discoverable by **AI agents**" — betting docs will be read more by agents than
  by humans. Confidence: **high** (vendor blog + multiple independent outlets).

**Strategic read (inference, not observation):** a well-funded incumbent has
just commoditized doc hosting to $0 and moved the meter to AI consumption. Any
Hickory plan whose gate is "private projects and editors" is charging for
something a funded competitor now gives away. The defensible meter is closer to
**execution** (which Hickory already treats as its cost-protecting quota) or to
**verification events**, not to seats.

---

## ReadMe — *primary*, readme.com/pricing

| Plan | Price | Notable |
|---|---|---|
| Starter | **$0** | 1 project, 1 published version, 1 admin |
| Pro | **$250/mo**, billed annually | Unlimited projects + versions, 5 admin seats, **+$20 per extra admin** |
| Enterprise | **$3,000+/mo**, annual only | Multiple combined projects, custom terms |

- **Value metric:** org-level tier, with admin seats as the expansion lever.
- **Free at every tier:** AI dropdown, `llms.txt`, MCP server. (Note: serving
  docs *to agents* is now table stakes, given away even on the free plan.)
- **Gated:** Ask AI is a **$150/mo add-on at every tier**. Pro unlocks Ask AI
  Lite, Agent Owlbert, AI Linter, GitHub AI Writer. Enterprise-only: Docs Audit,
  Private AI Context, Global Lint Rules, AI Translations.
- **Proven WTP:** the 1-project / 1-version free cap is aggressive, and the jump
  to $250 is a 12× step with no middle. That gap is where an indie tier lives.
- **Gap left open:** "Docs Audit" and "AI Linter" audit *prose quality*, not
  executable correctness. Nothing here runs an example.

Confidence: **high** for the plan shape; **medium** that the $250 figure is what
anyone actually pays (annual-only billing invites negotiation).

---

## Hex — *third-party*, julius.ai comparison page

- Professional **$36/editor/mo**; Team **$75/editor/mo**; plus pay-as-you-go
  compute **$0.32–$6.70/hour**.
- **Value metric:** hybrid — per-editor seat **and** metered compute.
- Confidence: **medium-low** (not read from Hex's own page; Hex publishes some
  pricing behind a contact form).

## Deepnote — *third-party*, deepnote.com/pricing via search summary

- Team **$39/editor/mo** billed yearly, **$49/mo** billed monthly, as of
  2026-08-01. Compute credits bundled into the tier rather than metered
  separately (the opposite choice from Hex).
- **Value metric:** per-editor seat, compute absorbed.
- Confidence: **medium**.

**Read across Hex/Deepnote:** the notebook category has settled on
**$36–$75 per editor per month** and disagrees only about whether compute is
bundled or metered. That band is the honest anchor for Hickory's Team tier —
$149/mo for 10 editors is **$14.90/editor**, roughly a third of the category
floor. That is either a deliberate wedge or money left on the table; it is not
a premium position.

---

## AWS Kiro — *third-party pricing*, kiro.dev/pricing (added 2026-08-09, addendum)

The teardown the first pass missed, and the most consequential one for the
provenance framing. Launched internationally **2026-05-07** as a ground-up
replacement for Amazon Q Developer.

| Plan | Price |
|---|---|
| Free | 50 credits |
| Pro | **$20/mo** |
| Pro+ | **$40/mo** |
| Pro Max | **$100/mo** |
| Power | **$200/mo** |

- **Value metric: AI credits**, overage **$0.04/credit**. Team plans mirror the
  same four paid tiers and add consolidated billing, usage analytics, and SSO
  via AWS IAM Identity Center.
- **What it does:** spec-driven development — one prompt becomes a requirements
  doc in **EARS notation**, a design doc, and a dependency-sequenced task list,
  with **requirement-to-task traceability** and each test "directly traceable to
  a requirement in `specs/SPEC.md`."
- **What it does not do:** re-execute anything to prove the trace still holds.
  The traceability is an **agent-authored link in markdown**.
- Confidence: **medium-high** on plan shape and prices (multiple third-party
  sources agree; Kiro's own page not fetched); **high** on the product
  description (consistent across independent write-ups).

## GitHub Spec Kit — free, MIT

- Agent-agnostic Python CLI (`specify`) that drops markdown templates and slash
  commands into a repo and drives `constitution → specify → plan → tasks →
  implement` using whatever agent you already have.
- **The spec is the unit of work: it persists as editable repo artifacts rather
  than a throwaway chat plan**, and `tasks.md` keeps requirement-to-task
  traceability.
- **Price: $0.** Value metric: none — it is a giveaway that makes the agent
  you already pay for more useful.
- Peers in the same 2026 cohort: BMAD-METHOD, OpenSpec, Augment Cosmos,
  `.cursor/rules`.

**Strategic read (inference).** Two facts sit together and neither is comfortable
alone. First, **there is now a published, per-developer price for "requirements
traced through to code": $20–$200/month**, which is real WTP evidence where the
first pass had none. Second, **the same capability is available free and
MIT-licensed from GitHub**, and bundled into an IDE by AWS. The category has
therefore proven that people pay — and simultaneously proven that *asserted*
traceability is a commodity. The only position left standing is the one none of
them occupy: a trace that is **derived and re-provable**, not asserted.

## Palantir Foundry Ontology — *third-party pricing* (added 2026-08-09, 2nd addendum)

Not a competitor. Recorded because the operator asked whether Hickory's proposed
user-defined taxonomy is "kind of like Palantir's Ontology" — and the answer
governs how the product may and may not be positioned.

**What it is (observation):** an operational layer over data already integrated
into Foundry, connecting datasets and models to real-world counterparts. It
contains **semantic elements (objects, properties, links)** *and* **kinetic
elements (actions, functions, dynamic security)** — described by Palantir and
its partners as a single **executable artifact** where the data model, business
logic, and governance live together. AIP puts AI agents on that same ontology
under the same permissions.

**Pricing (third-party, medium confidence):**

- Most first contracts at mid-market and large enterprises: **$500k–$2M/year**.
- Core-based licensing quoted from **£66,000/server core/year** upward.
- **Ontology volume metered in gigabyte-months.**
- First-year implementation "commonly matches or exceeds the subscription" — a
  $1M platform decision is a $2–2.5M program.

**Where the analogy genuinely holds:** user-defined objects/properties/links as
the extensible core rather than hardcoded entity types; and semantics fused with
execution in one artifact rather than a metadata layer bolted on the side. Those
are the two design commitments the operator is proposing, and Palantir is
evidence that a serious market rewards both.

**Where it decisively does not (inference):** Palantir's ontology derives its
value from being wired into the customer's *operational systems*. Hickory's
would map artifacts in a *git repository*. That is a far smaller scope, a far
smaller cost, and a far smaller value per deployment — and Palantir's economics
are services-led ($500k+ with implementation matching subscription), the exact
opposite of the self-serve, no-sales-team motion in `pricing-strategy.md`.

**Positioning consequence — a real two-audience split:**

- **To an enterprise buyer (niche N2), "Palantir's Ontology, scoped to your
  repo and provably derived" is instantly legible** and does useful work in one
  sentence.
- **To the OSS and HN audience it is actively costly.** Palantir is politically
  radioactive in precisely the watering holes that constitute this product's
  distribution (`personas/oss-maintainer.md`, `personas/delivery-owner.md`:
  Hacker News, Lobsters, r/ExperiencedDevs).

**Therefore: the comparison belongs in the enterprise conversation and must
never appear on the landing page, in a Show HN, or in the README.** This is one
of the few places in this recon where the same true sentence helps with one
cohort and damages another — the split maps exactly onto the operator's stated
build-for-myself / sell-to-enterprises strategy.

**Legibility warning (inference):** Palantir requires a sales engineer to
explain the ontology. A general, user-defined semantic mechanism is *harder* to
demo in thirty seconds than a fixed one, which cuts against the landing page's
job. Mitigation: demo the shipped default vocabulary concretely and mention
extensibility second.

## The unpriced substitutes (free, and therefore the real competition)

None of these are companies, and all of them are what the personas actually use
today. Confidence: **high**; these are long-standing OSS tools.

| Tool | What it does | Where it stops |
|---|---|---|
| Rust doctests, Python `doctest`, `mdBook test`, Sphinx `doctest` | Execute examples embedded in source or docs, fail the build on mismatch | Scoped to one language's toolchain; no environments, no data lineage, no transcript |
| Quarto, Jupyter Book / MyST, R Markdown, `bookdown` | Render prose+code documents, re-executing on build | Reproducibility is best-effort; no expectation assertions, no provenance from output byte back to origin |
| Runme (Stateful, OSS) | Makes markdown runbooks executable — "Jupyter with a shell kernel," CLI + VS Code notebook | Ops runbooks, not verified documentation; no published commercial pricing found |
| Dredd, Schemathesis | Diff live API responses against an OpenAPI spec in CI | Checks the **API**, not the prose or its examples |
| `doc-drift-guard` and similar | Statically check that doc examples reference symbols that exist | Static only — never runs anything |
| **Do nothing** | Ship docs, find out from a user | Free, and currently winning |

**The gap, stated plainly (observation):** across every paid product torn down
here, *nobody gates execution-plus-expectation verification of a document,
because nobody sells it.* The free tier of the category does the rendering; the
free OSS tier does per-language example testing; nothing commercial sits
between them. Whether that is whitespace or an empty room is the open question —
see `niche-map.md` and `failure-ledger.md`.
