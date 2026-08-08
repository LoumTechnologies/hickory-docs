---
name: optimize-saas-pricing
description: Design and implement SaaS pricing, packaging, and monetization experiments in a software codebase. Use when asked to create pricing A/B tests, plan email-list offer tests, test plan names or feature placement, add pricing landing-page variants, instrument revenue/conversion metrics, or implement pricing-page, checkout, billing, analytics, lifecycle email, or experiment changes for a SaaS product. For evaluating a business model and recommending an overall pricing strategy, use recommend-saas-pricing-strategy instead.
---

# Optimize SaaS Pricing Experiments

## Core Approach

Treat pricing as product and positioning work that needs measurement. Start from a candidate pricing strategy, buyer segment, or monetization concern, then design a low-risk experiment that can be implemented and read from the codebase.

Read `references/patio11-saas-pricing.md` when the task asks for experiment design, email-list offer tests, landing-page variants, metrics, or pricing implementation tactics. If the user primarily wants a business-model diagnosis, buyer segmentation, price grid, or strategic recommendation before experimenting, use `$recommend-saas-pricing-strategy`.

## Workflow

1. Inspect the existing experiment and monetization surface:
   - Pricing page, plan constants, billing provider integration, checkout/session creation, feature gates, quota enforcement, trial flow, onboarding, emails, analytics, and database fields for account/plan state.
   - Identify where public promises are made separately from where entitlements are enforced.
   - Preserve existing paid customers unless the user explicitly requests migration. Default to grandfathering current customers.

2. Turn the candidate change into a testable hypothesis:
   - Name the business question, the buyer segment, the observed problem, and the expected revenue or conversion effect.
   - If the strategy is underspecified, propose only the minimum diagnosis needed to form a coherent experiment. Do not expand into a full pricing strategy audit unless the user asks.
   - Prefer experiments around displayed pricing, plan order, feature placement, plan names, annual discount, trial/checkout flow, or channel-specific offers.

3. Design the experiment:
   - State the hypothesis, primary metric, guardrail metrics, audience, unit of randomization, sample split, exposure window, and decision rule.
   - For low-traffic SaaS, prefer larger price deltas, email-list offer tests, sales-assisted quotes, or landing-page/channel tests over tiny A/B tests.
   - Avoid unfair surprises. If testing lower/higher public prices, have a customer-support policy ready for honoring or manually extending offers.

4. Plan statistics pragmatically:
   - Use revenue per visitor/account, checkout-start rate, paid conversion, average revenue per account, annual-plan selection, expansion, refund, support burden, and retention as appropriate.
   - When judging profitability, use contribution margin against the project's marginal cost only (its own infrastructure, domain, Stripe fees, vendor usage beyond shared allowances). Shared portfolio vendor fees (SendGrid, PostHog, shared edge infrastructure) are overhead recovered across the whole portfolio, not a per-experiment cost input.
   - Do not overfit on trial signup volume. A higher price that reduces signups can still win on revenue and customer quality.
   - For small samples, recommend a sequential or fixed-window directional read with clear stop criteria rather than fake precision.
   - Define the minimum detectable effect only when baseline volume exists. If the codebase lacks historical analytics, first implement tracking and run a baseline period.

5. Plan email-list pricing tests when relevant:
   - Verify the product has a list, lifecycle emails, newsletter tooling, or consent records.
   - Randomly hold out a small percentage or split a selected segment into offer cells.
   - Send different offers, plan bundles, annual discounts, or price points to comparable groups.
   - Track opens/clicks only as diagnostics; judge by purchases, upgrades, replies, qualified calls, or retained revenue.
   - Include a manual support path for users who ask about different offers.

6. Build only the required implementation:
   - Add or update pricing copy, plan configuration, experiments, analytics events, billing metadata, email templates, or landing pages in the repo's existing style.
   - Register the experiment in the repo's `experiments.toml` and instrument it per `$implement-growth-experiments` (PostHog flags/events, numeric stimulus properties, observation exports) so results are regression-readable on the fleet board.
   - Use the existing billing provider patterns. For Stripe work, also use the Stripe skill if available.
   - Keep public pricing and backend entitlement changes deliberately coordinated. If an experiment should change display only, explicitly avoid changing enforcement.
   - Add tests around plan configuration, checkout payloads, entitlement gates, experiment bucketing, and analytics events when those paths exist.

## Deliverables

For strategy-only requests, produce:

- Experiment plan with hypothesis, variants, metrics, audience, runtime, decision rule, risks, and support policy.
- Instrumentation plan for missing events or revenue data.
- Email-list, landing-page, or sales-assisted test plan when useful.

For codebase requests, implement the smallest viable slice and report:

- Files changed and behavior changed.
- How the experiment is configured and how to read results.
- Tests or checks run.
- Follow-up operational tasks such as list segmentation, analytics dashboard creation, or billing-provider setup.

## Guardrails

- Do not present legal advice. If the user asks about legality, state that pricing tests are common but jurisdiction, discrimination rules, consumer protection rules, contracts, and regulated industries may require counsel.
- Do not recommend deceptive billing, hidden fees, dark patterns, or silently charging customers more than the price they accepted.
- Do not automatically migrate existing customers to higher prices. Recommend grandfathering or explicit migration communication unless the user requests otherwise.
- Do not let statistical neatness override business reality. A pricing decision can be valid with interviews, sales calls, or directional revenue evidence when traffic is low.
