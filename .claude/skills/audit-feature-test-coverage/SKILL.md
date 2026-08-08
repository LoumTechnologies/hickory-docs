---
name: audit-feature-test-coverage
description: Audit how well every advertised feature is tested, and make sure an AI coding agent can drive Playwright (or the repo's existing browser-automation tool) through real user flows — especially the happy path per plan tier — against a live environment, fixing code or environment issues it finds along the way. Use when asked to check test coverage of advertised/gated features, add or repair Playwright/E2E flows, set up per-plan-tier test accounts for agent-driven QA, verify a signup-to-value happy path actually works end-to-end, or make production safely testable by an autonomous agent. Complements audit-pricing-tables (checks what's promised) and plan-deploy-shared (production-safety and launch-state policy) by checking that what's promised actually works when driven like a real user.
---

# Audit Feature Test Coverage

## Goal

Every feature advertised anywhere (pricing tables, marketing copy, docs) should
be both automatically tested and drivable end-to-end by an AI coding agent
using browser automation, so regressions on the happy path are caught before a
real customer hits them. This skill audits and closes that gap; it does not
invent new features or change pricing.

Run `$audit-pricing-tables` first (or use its findings) to get the
feature-by-plan list this skill needs — the feature inventory should not be
re-derived from scratch here.

## Workflow

1. **Build the feature inventory.** Start from `$audit-pricing-tables` output
   or, if absent, enumerate features directly from `plans.json` entitlements
   plus anything advertised in pricing/marketing copy. For each feature, note
   which plan tier(s) grant it.

2. **Audit existing automated coverage.** For each feature:
   - Find unit/integration tests that exercise it (`mcp__semble__search`
     across `--content code` for the feature's gate/handler, then locate
     tests near it).
   - Find any existing E2E/browser tests (Playwright, Cypress, Puppeteer, etc.)
     that exercise it as a user flow rather than a unit.
   - Classify: no coverage / unit-only / E2E-covered / E2E-covered but stale
     (locators or flow no longer match current UI).
   - A feature with unit tests around its gate logic but no E2E test proving a
     real user can reach and use it is still a gap — report it as one.

3. **Confirm or set up agent-drivable browser automation.**
   - Check whether Playwright (preferred for its codegen/trace/agent-friendly
     APIs) or an equivalent tool is already installed and configured. If not,
     add it using the repo's primary language/tooling conventions and register
     a `just` recipe (`just test-e2e` or similar) per `$just`.
   - Confirm an AI coding agent can actually invoke it in this environment:
     browsers installed, headless mode works, base URL configurable via env
     var (never hardcoded), auth/login helper reusable across specs, traces
     or screenshots captured on failure so an agent can diagnose without
     rerunning blind.
   - Structure specs so a happy-path flow is one readable script per plan
     tier, not a monolith — an agent should be able to run and fix one tier's
     flow without touching the others.

4. **Set up per-plan-tier test accounts on production.** For each plan in
   `plans.json` (including any relevant retired/grandfathered shape if still
   enforced), there should be one durable account the agent can log into and
   exercise that tier's entitlements for real:
   - Check `plans.json`/Stripe for whether these accounts already exist
     (look for an internal/test flag or a naming convention). If missing,
     create them explicitly — do not reuse a real customer's account.
   - Mark each test account clearly as internal/synthetic: a distinct email
     domain or naming convention, an `internal_test_account` flag or
     equivalent property in the database, and excluded from PostHog
     revenue/analytics aggregates and from any customer-facing lists (support
     queues, admin "real customers" views, marketing exports).
   - Get each account onto its plan's actual entitlements without creating
     real recurring revenue: a Stripe test-mode-equivalent for live mode
     (100%-off coupon, comped subscription, or manually-set entitlement
     override that mirrors what a real purchase would grant) — never leave a
     live card being charged for an account nobody bills.
   - Store credentials for these accounts in the repo's existing secrets
     mechanism, never committed in plaintext. An AI agent invoked to run the
     flows needs to be able to obtain them the same way it obtains other
     environment secrets.
   - Before creating anything, check `launch-state` (per
     `$plan-deploy-shared`'s `pricing-and-launch-state.md`) — if the product is
     `launched`, be extra careful that synthetic accounts and their activity
     can't be mistaken for real customers anywhere (billing reports, support
     views, churn/retention dashboards).

5. **Write or repair the happy-path flow per plan tier.** For each tier's test
   account, script the primary signup-to-value (or login-to-value, for
   existing test accounts) path: the sequence of actions that lets that plan's
   customer actually get value from what they're paying for, hitting each
   entitlement that tier grants and confirming higher tiers see additional
   capability while lower tiers correctly see it gated. Prefer one flow per
   tier over one giant flow with conditionals.

6. **Run the flows and fix what's broken.** Execute each tier's flow (staging
   first if the flow is destructive or unproven; production once safe) and
   treat failures as real bugs to fix — code, config, feature flags, or
   environment — not as reasons to weaken the test. If a flow fails because
   the feature genuinely doesn't work, fix the feature. If it fails because a
   selector/locator is stale, fix the spec. Re-run until green, and capture
   the passing run's trace/screenshots as the coverage evidence.

7. **Wire this into recurring checks.** Add or update a `just` recipe and, if
   the repo has CI, a scheduled or on-deploy job that re-runs the per-tier
   happy-path flows against staging automatically, and against production on
   a lower-frequency schedule (production runs against real infra carry more
   risk/cost — don't run them on every commit). Report the recommended
   cadence rather than assuming CI configuration is in scope unless asked.

8. **Report.** Produce a coverage matrix (feature × plan tier × automated
   coverage level × E2E status) and a list of fixes made, distinguishing:
   - Bugs found and fixed (feature broken, now working).
   - Gaps closed (new E2E coverage added).
   - Remaining gaps needing follow-up (e.g. feature requires a paid
     third-party sandbox that isn't provisioned).
   - Production test-account setup performed, with where credentials live and
     how they're excluded from customer-facing views/analytics.

## Guardrails

- Never run destructive or data-mutating exploratory actions against
  production outside the dedicated synthetic test accounts. Real customer
  data and accounts are off-limits for automated flow-testing.
- Never leave a production test account able to incur real charges,
  send real emails/SMS to third parties, or trigger real support/ops alerts
  (route its notifications to a clearly-labeled internal destination or
  suppress them per the account flag).
- Don't let synthetic test-account activity pollute analytics used for
  pricing/experiment decisions (`$optimize-saas-pricing`,
  `$implement-growth-experiments`) — exclude by the internal-account flag at
  the query/dashboard level, not by hoping nobody looks.
- Prefer staging for first-run and iterative debugging of a new flow; only
  promote a flow to run against production once it's stable, since production
  runs are the ones with real cost, real rate limits, and real blast radius.
- Credentials for test accounts follow the same secrets handling as any other
  production credential — no plaintext in the repo, no reuse across
  environments.
- If a feature can't be safely tested against production at all (e.g. it
  sends irreversible real-world side effects with no safe synthetic mode),
  report that as a structural gap rather than forcing a workaround.
