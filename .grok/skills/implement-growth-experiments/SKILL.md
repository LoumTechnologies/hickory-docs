---
name: implement-growth-experiments
description: Build the experiment framework into an app so pricing, advertising, and funnel experiments run continuously and produce regression-readable data. Use when the agent is asked to add an experiments registry, instrument pricing or ad experiments, wire PostHog flags/events for experimentation, set up UTM conventions and ad-spend tracking, export observations for regression analysis, or feed experiment results into the fleet portfolio board. For choosing what pricing experiment to run, use optimize-saas-pricing; this skill is the mechanical framework that makes any such experiment runnable and readable.
---

# Implement Growth Experiments

## Goal

Every app in the portfolio measures the real world's response to deliberate
stimuli — prices, plan grids, ad spend, creatives, landing copy — in one
standard shape, so effects can be estimated with linear regression and read on
the fleet board without per-app analysis code. The fleet repo
(`~/Documents/src/fleet`) consumes this contract; do not invent a variant shape.

## The Contract (defined here, consumed by fleet)

1. `experiments.toml` at the repo root registers every experiment:

   ```toml
   [[experiment]]
   id = "pro-price-2026-07"          # stable, kebab-case, dated
   hypothesis = "Raising Pro from $19 to $29 does not reduce revenue per visitor"
   kind = "pricing"                   # pricing | advertising | onboarding | other
   status = "running"                 # planned | running | complete | abandoned
   flag = "pricing-grid-v2"           # PostHog flag key when assignment is flag-based
   started = "2026-07-07"
   unit = "visitor"                   # randomization unit
   primary_metric = "purchase_completed.amount"
   stimuli = ["price_pro"]            # independent variables, numeric

   [experiment.decision]
   rule = "adopt if revenue/visitor effect >= 0 after 300 visitors per arm"
   ```

2. `experiments/<id>.csv` holds observations: a header row with one column per
   stimulus plus an `outcome` column; extra columns (unit id, timestamp) are
   ignored. One row per randomization unit. Values numeric; encode categorical
   variants as 0/1 dummy columns (e.g. `creative_b = 1`). No commas in values.
   v1 exports these from PostHog or the app database via a `just
   export-experiments` recipe; PostHog API sync is fleet's job (ticketed there).

3. Readouts come from `fleet-experiments readout` (or the fleet board): OLS of
   outcome on stimuli with coefficients, standard errors, t-stats, and R².

## Implementation Workflow

1. Register the experiment in `experiments.toml` before shipping any variant
   code, with the decision rule written down up front (pre-commitment is what
   makes the regression readable as evidence rather than a story).
2. Wire assignment:
   - Pricing/funnel: a PostHog feature flag or experiment keyed by the
     randomization unit; persist the assigned variant at the moment of exposure
     (see `$implement-billing-chassis` rule 3 — charge what was displayed).
   - Advertising: assignment happens at the channel. Encode it in UTMs:
     `utm_campaign = <experiment id>`, `utm_content = <variant>`; capture UTMs
     into PostHog person/event properties on landing.
3. Emit events that carry, on every exposure and outcome event: the experiment
   id, the stimulus values as numeric properties (e.g. `price_pro: 29`), and
   the outcome value (e.g. revenue amount, 0 for non-converters at window end).
4. For ad experiments, also record spend: keep a small spend log (CSV or table)
   of `date, channel, campaign, variant, spend_usd` maintained by hand or API,
   so cost-per-acquisition and spend-response regressions are possible.
5. Add the `just export-experiments` recipe that materializes
   `experiments/<id>.csv` from PostHog (HogQL query) or the app database.
6. Close the loop: when the decision rule triggers, set the experiment's
   `status = "complete"`, record the decision and the final readout in the repo
   (a short note in the experiment entry or docs), and act on it — adopt the
   variant in `plans.json`/copy, or revert.

## Regression Discipline

- Regression estimates causal effects only for randomized stimuli. Spend and
  channel data are usually observational — report those readouts as
  correlations and say so.
- Include every deliberately varied stimulus as a column; dummy-code variants;
  don't regress on outcomes or post-treatment variables.
- |t| > 2 is a directional signal on small samples, not proof. The decision
  rule, sample size, and honesty about power come from
  `$optimize-saas-pricing`'s statistics guidance — low-traffic products should
  prefer big deltas and directional reads over fake precision.
- One outcome per experiment (the registered `primary_metric`). Secondary
  metrics are diagnostics, not grounds for switching the win condition
  after the fact.

## Guardrails

- Same honesty rules as `$optimize-saas-pricing`: no dark patterns, no
  charging a price other than the displayed one, grandfather existing
  customers, have a support policy for customers who saw a different offer.
- Don't run overlapping experiments on the same unit and metric without
  recording both stimuli in both experiments' observations.
- Ad platform terms and consent: UTM capture is fine; do not import ad-platform
  user data into PostHog beyond what the platform's terms allow.
