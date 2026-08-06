# Hickory Docs — Pricing Strategy

Produced with `$recommend-saas-pricing-strategy` (2026-08-05). This is the
strategy document; the mechanical implementation (plans.json, Stripe catalog,
entitlements) follows `$implement-billing-chassis`.

## Business-model diagnosis

**Category**: developer documentation + computational notebooks, open-core.
Nearest paid comparables: ReadMe ($99–$399+/project/mo), GitBook (~$8–$12/user/mo),
Hex/Deepnote (~$24–$36/editor/mo), Observable. None of them verify that docs
still run; that is the buyer-visible differentiator ("your quickstart is tested
in CI"), and it is screenshotable: a red ❌ on a doc whose example broke.

**The outcome we anchor to**: a broken quickstart or drifted example is a
support ticket, a lost evaluation, or a failed onboarding. One prevented
"our install instructions silently stopped working" incident is worth more
than a year of the Team plan to any devtools company. Secondary anchor:
reproducing a statistical/analytical document without "works on my machine."

**Buying motion**: self-serve, bottom-up via the open-source repo and an HN
launch. No sales team. That mandates clear public packaging and a free tier
that produces the distribution (public docs hosted free = marketing surface,
same mechanic as ReadMe/Observable).

**Cost structure (portfolio accounting)**: Railway app + Postgres are the
marginal costs (~$10–20/mo at launch); execution runs on Nate's own Cloud
Canopy node, so compute margin is effectively hardware amortization until
third-party nodes are added. PostHog/SendGrid are shared portfolio overhead.
True floor per paying customer is near zero; price is set by value, not cost.
Flag: hosted execution minutes are the first thing that will exhaust a shared
allowance — meter them from day one even where unlimited-feeling.

## Should this be sold at all?

Yes, with eyes open. This is simultaneously a portfolio piece and a product,
and the open-source core would justify itself even at $0 revenue. But there is
a real buyer (devtools teams whose docs are product surface; data teams needing
reproducibility), a real channel (OSS + HN + "verified docs" badge backlinks),
and near-zero marginal cost. The failure mode to avoid is pricing it like a
prosumer toy. **Readiness gate, stated explicitly**: do not enable live-mode
Stripe until (a) hosted execution is stable on at least one canopy node with
quota enforcement, and (b) the verification CI story works end-to-end on a
stranger's repo. Until then, sandbox mode + a waitlist on the paid tiers is the
honest posture.

## Buyer map

| Segment | Value received | WTP signal | Motion |
|---|---|---|---|
| OSS maintainers / individuals | Verified public docs, hosted notebooks | Low ($0); they are the distribution | Free |
| Indie devtools / solo founders | Private projects, CI verification badge | $20–50/mo | Self-serve Pro |
| Devtools & platform teams | Docs-as-CI across a team, agent-authored docs, review workflow | $100–300/mo | Self-serve Team |
| Data/research teams, regulated | Reproducible analyses, provenance trail, SSO, own compute | $300+/mo, annual | Business / Enterprise |
| Self-hosters | Everything, on their own metal | $0 direct; ecosystem + funnel | OSS + BYON |

## Recommended grid

Value metric: **private projects + editors**, with **hosted execution minutes**
as the cost-protecting quota (generous, visible, not the headline). Buyers can
predict "how many private doc projects and people," not "how many minutes."
The AI agent is metered separately because its cost (LLM tokens) is real:
bring-your-own-API-key on lower tiers, included allowance on Business.

| Plan | Price | Fit | Includes |
|---|---|---|---|
| **Open** | $0 | OSS, individuals, evaluation | Unlimited public projects, 1 private project, 1 editor, 300 exec min/mo, community support, BYO-key agent |
| **Pro** | $29/mo ($290/yr) | Indie/solo commercial | 10 private projects, 3 editors, 2,000 exec min/mo, CI verification checks + badge, BYO-key agent |
| **Team** | $149/mo ($1,490/yr) | Devtools/platform teams (the default plan — positioned as such) | Unlimited private projects, 10 editors then $12/editor, 10,000 exec min/mo, agent allowance (metered, ~$20 LLM budget included), review/approval workflow, priority execution |
| **Business** | $449/mo (annual-first) | Data/research orgs, security-conscious | Everything in Team; SSO/SAML, audit provenance export, **bring-your-own canopy node** (their hardware, our control plane), larger agent allowance, 30 editors then $10/editor |
| **Enterprise** | Talk to us | Procurement, custom terms, air-gapped | Self-hosted control plane, custom nodes, support SLA |

- Execution overage: metered add-on ($0.10/min bundle-priced), never a surprise
  bill — hard stop + upgrade prompt by default.
- Self-hosting the OSS core stays free forever (single node, no hosted UI
  collaboration); that credibility is the moat and the funnel.
- Grandfathering: standard policy from day one — price changes never migrate
  existing subscribers; `plans.json` retains retired plans (the billing chassis
  makes this automatic).

## Rationale

- $29/$149/$449 follows the "price to business value, skip the $9 toy band"
  heuristic while staying self-serve-credible. The second-cheapest plan (Pro)
  will capture volume; Team is engineered to be the obvious default for the
  actual target buyer and carries most revenue.
- Per-project pricing (ReadMe-style) was considered and rejected: hickory-docs
  wants many small verified doc projects per org, and taxing project count
  fights the product's own adoption loop.
- BYON (bring your own node) as a Business feature converts the distributed-
  execution architecture into a price fence that costs us nothing and answers
  the security objection ("code runs on our hardware").

## Validation plan

1. HN launch with the Open tier + visible pricing page (sandbox checkout →
   waitlist capture). Instrument `pricing_page_viewed`, `plan_selected`,
   `checkout_started` from day one; the fake-door on Team/Business is the
   willingness-to-pay test.
2. 10 concierge onboardings from HN signups; ask the Team-plan question
   directly ("would your team pay $149 to make doc drift a CI failure?").
3. First pricing experiment (later, via `$optimize-saas-pricing`): Team at
   $99 vs $149 vs $199 — directional, large gaps, low traffic.
4. Decision rule: ≥5 fake-door Team selections or ≥1 concierge commitment
   within 6 weeks of launch → enable live billing; otherwise revisit packaging
   before touching price.
