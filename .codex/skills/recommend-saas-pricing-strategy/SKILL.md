---
name: recommend-saas-pricing-strategy
description: Evaluate a SaaS business model and recommend pricing strategy, packaging, segmentation, and monetization changes. Use when asked to audit or critique pricing, decide who the product should charge, set or raise prices, design plan tiers, choose value metrics, assess willingness to pay, add enterprise packaging, or recommend a pricing grid before running experiments. For designing or implementing A/B tests, email-list offer tests, landing-page variants, analytics, or checkout changes in a codebase, use optimize-saas-pricing instead.
---

# Recommend SaaS Pricing Strategy

## Core Approach

Evaluate pricing as a business model and positioning decision before turning it into an experiment. Start from buyer identity, value received, acquisition channel, sales motion, usage pattern, and the business outcome the product improves. Recommend pricing that a real buyer can understand, justify, and buy.

Read `references/saas-pricing-strategy.md` when the task asks for diagnosis, plan/packaging recommendations, value metrics, price levels, or business-model critique. Keep `SKILL.md` loaded for workflow; use the reference for heuristics and concrete tactics.

## Workflow

1. Build the business-model context:
   - Identify the product category, target customer, buyer, user, buying trigger, acquisition channel, sales motion, switching cost, and operational importance.
   - Separate prosumers, freelancers, small teams, agencies, departments, regulated businesses, and enterprise buyers when relevant.
   - Ask for missing context only when it materially changes the recommendation. Otherwise state assumptions and proceed.

2. Diagnose current monetization:
   - Find the outcome the product improves and anchor price to avoided cost, generated revenue, saved labor, reduced risk, or one painful event avoided.
   - Compare current plan prices against buyer value, acquisition cost, support burden, and sales complexity.
   - Use portfolio cost accounting when computing floors and margins: shared vendor fees (e.g., SendGrid's monthly fee with included emails, PostHog's included events, shared edge/tunnel infrastructure) are portfolio overhead spread across all projects, not a cost this product's price must cover alone. A project's true floor is its marginal cost (its own DigitalOcean resources, domain, Stripe fees, vendor usage beyond shared free allowances) plus a contribution toward portfolio overhead recovery. Do not conclude "unprofitable" by charging one product the full fixed fee of a vendor every project shares — and do not treat shared allowances as free forever; flag when a product's usage will exhaust one.
   - Flag likely underpricing, especially $9, $19, and low-end plans for products sold to businesses.
   - Identify whether the model should be self-serve, sales-assisted, enterprise, usage-based, seat-based, tiered, add-on based, or hybrid.

3. Recommend segmentation and packaging:
   - Segment by value received, buyer type, usage scale, collaboration needs, compliance/security, integrations, support, and operational importance.
   - Prefer plan names that help buyers self-select by identity or company context. Avoid names that embarrass serious businesses into a low-status plan.
   - Put high-value business features in higher tiers, but do not rely on customers accurately forecasting obscure usage counts.
   - Include an enterprise path when procurement, security review, custom terms, high usage, or high-consequence workflows are plausible.

4. Decide whether the product should be sold at all, before proposing a number.

   "Do not sell this" is a legitimate and useful recommendation. Reach for it
   when the expected value of selling is negative, not merely small. Signals:
   a saturated category where the differentiator is technical rather than
   buyer-visible; no identified acquisition channel at a cost the plausible
   price can repay; or a product that is a demo, an example app, or a portfolio
   piece rather than something with a buyer.

   Weigh the *costs* of selling explicitly, because they are easy to omit: a
   Stripe catalog to maintain, refunds, licence support, and a shipping
   obligation. Against a handful of sales those costs dominate.

   Say so plainly and recommend the alternative — ship it free as a showcase or
   lead generator, fold it into a product that does have a buyer, or park it per
   `$sunset-project`. Never manufacture a price to fill a slot in a catalog. A
   documented decision not to sell is a better deliverable than a number nobody
   believes.

   Keep this separate from product readiness. "The price is right but the
   product is not ready" is a different finding from "there is no viable price",
   and conflating them loses information. State readiness gates explicitly when
   the price is sound but shipping it would be premature.

5. Propose the pricing strategy:
   - Recommend plan names, prices, buyer fit, value metric, included features, limits, annual discount, trial posture, and enterprise path.
   - Explain the economic logic behind each tier in plain business terms.
   - Preserve existing paid customers unless the user explicitly asks for migration. Default to grandfathering current customers.
   - Call out operational prerequisites such as support readiness, sales qualification, billing setup, analytics, messaging, or terms changes.

6. Recommend validation steps:
   - Suggest interviews, sales calls, concierge offers, quoted deals, landing-page tests, email-list offers, or pricing experiments appropriate to the traffic and sales motion.
   - For implementation-heavy experiment work, hand off to `$optimize-saas-pricing`.
   - For low-traffic SaaS, prefer directional validation with large enough changes to matter rather than tiny A/B tests.

## Deliverables

Produce:

- Business-model diagnosis.
- Buyer and segment map.
- Recommended pricing grid or packaging model.
- Rationale for value metric, prices, plan names, limits, and enterprise path.
- Migration or grandfathering recommendation for existing customers.
- Validation plan and instrumentation gaps.

If working in a codebase, inspect pricing pages, plan constants, billing setup, entitlement gates, onboarding, emails, analytics, and database fields only as needed to understand the current business model. Do not implement experiment or billing changes unless the user explicitly asks; use `$optimize-saas-pricing` for that work.

## Guardrails

- Do not present legal advice. If the user asks about legality, state that pricing tests and pricing changes are common but jurisdiction, discrimination rules, consumer protection rules, contracts, and regulated industries may require counsel.
- Do not recommend deceptive billing, hidden fees, dark patterns, or silently charging customers more than the price they accepted.
- Do not automatically migrate existing customers to higher prices. Recommend grandfathering or explicit migration communication unless the user requests otherwise.
- Do not treat plan math as a substitute for customer value, buyer psychology, acquisition economics, and willingness to pay.
