---
name: plan-deploy-shared
description: Plan, implement, or review the substrate-independent half of production and staging delivery for SaaS products — environments and the promote-to-production path, GitHub Environments and branch policy, staging/production parity and staging data rules, plans.json-driven billing catalogs with sandbox/live enforcement, per-environment product analytics, launch-state risk assessment before destructive changes, and traffic-surge runbooks. Use alongside exactly one substrate skill: $plan-deploy-terraform when this product owns its infrastructure as code, or $plan-deploy-paas when a managed platform (Railway, Fly, Render) owns provisioning. Everything here holds either way; the substrate skill supplies the mechanism.
---

# Plan And Deploy — Shared

This skill is the half of delivery that does not change when the substrate does.
Read it for **every** cloud product. Then read exactly one of:

- **`$plan-deploy-terraform`** — we own the infrastructure and describe it as
  code (DigitalOcean, Terraform workspaces, state, plans in PRs).
- **`$plan-deploy-paas`** — a managed platform owns provisioning (Railway, Fly,
  Render); there is no infrastructure repo to apply.

When a rule below needs a mechanism — "the deploy must fail on a wrong Stripe
key" — the substrate skill says how. Do not implement the mechanism from this
file alone, and do not weaken the rule because the substrate makes it awkward.

## Environments, branches, and promotion

- There are two major **environments**: **production** (ALWAYS spelled out,
  NEVER "prod") and **staging**. Each has a GitHub Environment and attributable
  cloud resources. Production has **no git branch**.
- There must **always** be a long-lived **`master`** git branch. There must
  **never** be a `production`, `main`, or `staging` branch.
- **Staging is always cloud-hosted** for cloud products. Never call a machine on
  a developer's laptop "staging" — that is the local dev environment.
- Push (or merge) to **`master`** deploys staging automatically and runs
  smoke/E2E against it.
- **Production deploys only through the gated path**: **Trigger Promote to
  Production** → **Promote to Production** (`workflow_dispatch` + the
  `production` GitHub Environment). Deploy the chosen ref (normally the `master`
  tip), health-check it, and stamp an immutable `vX.Y.Z` when `bump` is not
  `none`. Never auto-promote from a branch push or PR merge.
- Downloadable products use **Trigger Stable Release** / **Stable Release**
  instead — see `continuous-delivery-downloadable`. Do not use "Stable Release"
  wording for cloud jobs.
- **Roll back** by re-running the promote path with an older ref and
  `bump: none`. Application rollback is clean; schema and infrastructure changes
  do not auto-revert.

See `references/promotion-and-environments.md` for the workflow table, rules,
and what each substrate plugs into it.

## Staging fidelity and data

Staging exists to catch what local dev cannot. It is worth very little if it
differs from production in shape rather than in size.

- Same substrate *class*, same deploy path, same TLS mechanism, same database
  engine, same migration path, same config schema. Differ in size, replica
  count, retention, and test-mode third-party accounts.
- Separate domains, DNS records, OAuth callbacks, cookies, webhook endpoints,
  and TLS issuance.
- **Never put real customer data in staging** — staging sends real email. Gate
  outbound email/SMS/webhooks so staging cannot contact a real customer by
  accident. Evaluate production data for its messy shapes (NULLs, missing
  values, mistyped entries) and maintain a seed script that reproduces them.

See `references/staging-parity.md`, including the order to reduce staging cost
in.

## Billing catalog

- **`plans.json` is the single source of truth** for plans, prices,
  entitlements, and grandfathering metadata. The payment provider's catalog is
  **generated from it** — never created by hand in a dashboard, never minted
  imperatively from application code at runtime.
- **Staging and local dev use the provider's sandbox; production uses live
  credentials**, and the deploy **fails** rather than warns when the mode does
  not match the environment.
- **Prices are append-only.** Key each price by an immutable identifier; never
  mutate or destroy a published one. Superseding means adding a new key and
  leaving the old addressable for existing subscribers.
- Price ids are **per environment** — one logical price has a different id in
  sandbox and live. Store them keyed by environment and have entitlement lookup
  search all of them.

See `references/pricing-and-launch-state.md`. The enforcement mechanism is
substrate-specific: Terraform `precondition` + `prevent_destroy`, or an
idempotent sync script with the same guarantees.

## Vendors and analytics

- Use SendGrid for transactional email, PostHog for product analytics, feature
  flags, and experiments, Stripe for billing.
- Provision **one analytics project per product per environment** — staging and
  production never share one, the same split as sandbox/live payment keys.
  Inject each environment's keys through the matching GitHub Environment per
  `config-and-environments`.
- **Send a deploy annotation from the CD pipeline** so metric changes can be
  correlated with releases.
- Before believing staging and production are separated, actually query the
  provider and confirm two distinct project ids exist and that recent events
  from each environment's domain land in the matching one. The same variable
  name existing in both GitHub Environments does not prove the values differ.
- Distinguish shared portfolio overhead (fixed vendor fees with included
  allowances, paid once for all products) from per-project cost. Never create a
  duplicate account for a shared vendor to dodge attribution — attribute usage
  instead. See `references/cost-attribution.md`.

## Launch state

Maintain a **`just launch-state`** command that answers "how launched are we?"
before any destructive infrastructure, data, or billing change. It should report
whether real accounts exist, whether real customer data exists, whether the
payment provider shows live customers or subscriptions, and per-price usage —
then classify risk as `unlaunched`, `soft-launched`, or `launched`.

A command that cannot reach production should say what it could not check
rather than implying the answer is "nothing". See
`references/pricing-and-launch-state.md`.

## Traffic-surge readiness

Any event that can spike traffic (a Show HN / Product Hunt launch via
`$community-engagement`, a paid push via `$ads`, press) needs a **concrete,
pre-written scale-up plan** — not a design session under load. Maintain it at
**`docs/operators/runbooks/traffic-surge.md`** with the product's *real*
numbers:

- **What saturates first, cheapest mitigation first.** Static pages usually
  scale nearly free; the API, TLS termination, and the database are the real
  limits. List the order things fall over and the no-spend levers before any
  resize.
- **The fast path, as exact commands.** The specific variable or recipe that
  adds capacity, the next 1–2 size steps, each step's **$ per month and
  pro-rated per day**, how long it takes, and whether it drops traffic. An
  operator under load pastes commands; they do not read infrastructure code.
- **Temporary vs permanent.** A temporary bump is a change with a scheduled,
  pre-drafted revert. A permanent one updates the budget baseline too.
- **The trigger.** Which monitoring alert means "execute now", and the health
  URL to watch during and after.
- **Same gates as everything else.** Scale-ups go through the normal deploy
  path, never console clicking — and when a live-reaction knob exists, the
  runbook documents it and the declared state is reconciled afterward.
- **Launch coupling:** `$community-engagement` and `$ads` treat a launch slot as
  READY only when this runbook exists and its commands have been sanity-checked.
  When it is missing, writing it is part of this skill's job.

## References

- `references/promotion-and-environments.md` — the canonical workflow names,
  triggers, gates, concurrency, immutable tags, and rollback.
- `references/staging-parity.md` — what may differ between staging and
  production, what may not, and the cost-reduction order.
- `references/pricing-and-launch-state.md` — plans.json as source of truth,
  append-only price rules, launch-state, risk levels, migration rules.
- `references/cost-attribution.md` — project tiers, shared overhead vs
  per-project cost, and what must be true for spend to be attributable.
