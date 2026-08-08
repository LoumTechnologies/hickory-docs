---
name: audit-pricing-tables
description: Audit plans.json (the source-of-truth for pricing plans and capabilities) against every pricing table, comparison grid, and gate/prompt displayed anywhere in the current repository. Use when asked to check pricing-table accuracy, find features missing from the pricing page, verify gates match plan limits, reconcile a detailed feature-comparison table against a high-level pricing table, or confirm PostHog tracks feature usage well enough to inform pricing decisions. Complements implement-billing-chassis (builds the chassis) and optimize-saas-pricing (designs experiments on top of it) by checking that an existing chassis stays truthful as plans.json and the UI drift apart over time.
---

# Audit Pricing Tables

## Goal

`plans.json` is the source of truth (also consumed by Terraform per
`$plan-deploy-shared`). Everywhere else that describes plans — high-level pricing
page, detailed feature-comparison table, in-app upgrade prompts, gate error
messages, docs — is a rendering of it that can silently drift. This skill finds
every place drift happened and reports it as findings, not a rewrite.

Do not redesign pricing or packaging here. If a mismatch looks like it should
be resolved by changing what a plan *includes* (a strategic call, not a bug),
call it out as a recommendation — don't just "fix" it by editing `plans.json`.

## Workflow

1. **Locate the source of truth and its renderers.**
   - Find `plans.json` (or equivalent). Use `mcp__semble__search` for
     "pricing plan capabilities" / "plans.json" across `--content config` and
     `--content code` before grepping.
   - Find every consumer: high-level pricing page/section, detailed
     feature-comparison table, plan-selector dropdowns, upgrade/upsell modals,
     gate/paywall components, quota error messages, docs or README pricing
     mentions, Terraform (Stripe catalog), and any hardcoded plan-name or
     price strings outside `plans.json`.
   - Note whether the detailed comparison table and the high-level table are
     the same component or two independent surfaces — they usually drift
     independently and both need checking.

2. **Diff features, not just prices.** For each plan, build two lists side by
   side: entitlements/capabilities declared in `plans.json`, and features
   claimed or implied in each displayed table. Classify every mismatch:
   - **Missing from table**: `plans.json` grants a capability but no displayed
     table mentions it. Report as a finding — customers can't discover what
     they're paying for. (This is a display bug even when it's *not* the case
     below.)
   - **Table overpromises**: table lists a feature as included/excluded but
     `plans.json` (or the code path it maps to) disagrees, or the feature
     doesn't actually exist in the codebase yet (i.e., listed as shipped but
     unimplemented). Report as a correctness bug — highest severity, this is a
     false advertising / support-ticket risk.
   - **Table under-promises on purpose**: `plans.json` grants access that the
     table doesn't advertise, and nothing else in the repo explains why
     (no comment, no experiment, no ticket reference). This is *allowed* — plans
     can intentionally hold back advertised scope — but flag it explicitly as a
     **strategic decision needing confirmation**, not a bug, and ask whether it's
     deliberate (e.g. a soft-launched capability, a support-only unlock) or
     drift.
   - **Granularity mismatch**: high-level table and detailed comparison table
     disagree with each other at the level of detail each claims to support
     (e.g. high-level says "unlimited X", detailed table gives a numeric cap
     that doesn't match `plans.json`'s actual limit).

3. **Verify every claimed feature is implemented.** For each row in the
   detailed comparison table, confirm the feature exists in code, not just in
   copy — grep/semble-search for the gate or code path. A checked box for a
   feature with no implementation is worse than a missing row: it's an
   affirmative false claim. Report unimplemented-but-advertised features as
   correctness bugs, not backlog items.

4. **Verify gates and UI prompts against actual limits.** For every quota,
   limit, or capability gate (paywalls, feature flags, disabled buttons,
   usage-limit banners, upgrade-prompt copy, error messages on 402/403):
   - Confirm the number/boolean the gate enforces matches the entitlement
     value in `plans.json` for that plan (via whatever entitlement-resolution
     function `$implement-billing-chassis` established — there should be one;
     flag scattered plan-name string checks as a finding if found).
   - Confirm the prompt text quotes the correct plan name, correct limit
     number, and points at a plan that actually unlocks the thing being gated
     (a common drift bug: prompt says "upgrade to Pro" but the feature was
     moved to a different tier in `plans.json`).
   - Confirm retired/grandfathered plans still gate correctly per their
     original entitlements, not the current default plan set.

5. **Check PostHog usage tracking on gated/paid features.** For each
   capability declared in `plans.json`, confirm there's a PostHog event fired
   when a user actually exercises that feature (not just when they view a
   pricing page or hit a gate) — a distinct instrumentation goal from
   `$implement-growth-experiments`'s experiment-stimulus events. This is about
   *willingness-to-pay signal*, not experiment readout:
   - Feature-use events should carry enough properties to answer "who used
     this, how much, on what plan" — e.g. plan id/name, account id, and a
     magnitude property where the feature has one (rows exported, seats used,
     API calls made), not just a boolean "used_feature" ping.
   - Gate-hit events (`gate_blocked`, `upgrade_prompt_shown`, `quota_exceeded`)
     are equally important: an ungated feature generates no signal about
     whether it's worth paying for, but a *blocked* attempt is a strong signal
     — confirm these fire too, with the plan the user was on and the limit
     they hit.
   - Report features with zero usage instrumentation as findings — this
     product can't tell if a feature justifies its plan placement without
     this data.
   - Don't invent a new event taxonomy per feature; check whether the repo has
     an existing usage-event convention and reuse it.

6. **Produce the report.** Group findings by severity, matching the
   classification in step 2:
   - **Bugs (fix)**: table overpromises, unimplemented advertised features,
     gate/prompt values that don't match `plans.json`, broken entitlement
     lookups, missing usage instrumentation on paid features.
   - **Drift (confirm with user)**: features present in `plans.json` but
     absent from tables, with no evidence it's deliberate.
   - **Strategic calls (flag, don't fix)**: plans.json grants scope the
     table deliberately withholds, or vice versa, where the repo shows
     evidence it's intentional — still surface it so the user can confirm the
     business decision is still correct.
   For each finding: file/component, the plan(s) affected, what `plans.json`
   says vs. what's displayed/enforced, and severity. Only make code changes
   the user asks for — default to reporting, since some "mismatches" are
   pricing decisions outside this skill's authority.

## Guardrails

- Never resolve a display/entitlement mismatch by silently loosening or
  tightening entitlements — that's a pricing decision, escalate it.
- Treat `plans.json` as authoritative for entitlements even when a table
  looks more "correct" by inspection; the table is the thing that can be
  wrong.
- Don't flag missing-from-table capabilities that are clearly internal/admin
  (not customer-facing plan differentiators) — check the entitlement's name
  and where it's read before reporting it as a pricing-table gap.
- If `plans.json` doesn't exist yet or entitlement enforcement is scattered
  string checks with no single resolution function, note that as a
  prerequisite finding pointing at `$implement-billing-chassis` rather than
  trying to build the chassis inline here.
