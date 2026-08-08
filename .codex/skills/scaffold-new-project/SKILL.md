---
name: scaffold-new-project
description: Bootstrap a new software project with the portfolio's standard conventions baked in. Use when the agent is asked to start a new project, create a new SaaS/product repo, set up a validation-tier landing repo, graduate a validated idea into a full build, or make an existing repo conform to portfolio conventions (justfile, portfolio.toml, guarantees, Terraform layout, master branch without production branch, PostHog, Stripe, SendGrid).
---

# Scaffold New Project

## Goal

Every project in the portfolio has the same skeleton, so context-switching
between many small products is cheap, fleet tooling can discover and cost every
project automatically, and later skills (`$implement-billing-chassis`,
`$plan-deploy-shared`, `$sunset-project`) find the shape they expect.

## Pick The Tier First

- **Validation tier** — for unvalidated ideas (see `$validate-idea`): static
  landing page, PostHog project, SendGrid list, `portfolio.toml`. No backend,
  no database, no staging, no Stripe. Scaffold only steps 1–4 below.
- **Full tier** — for validated ideas or explicitly commissioned products:
  everything below.

**Shared infrastructure is full tier with parts deliberately omitted.** Some
repos (a license service, an auth service, an internal API) serve other products
rather than customers. They still need the full delivery and infrastructure
skeleton, but they have no funnel to instrument and nothing to validate, so
PostHog, `experiments.toml`, and the landing page do not apply — set
`posthog = false` and say why in the repo's `CLAUDE.md` rather than scaffolding
empty analytics. Their cost is portfolio overhead: give them their own
`portfolio.toml` and DO projects so it stays attributable, and record which
products they serve.

## Workflow

1. Settle the language and stack once:
   - Ask the user which primary language to use if it is not obvious, then
     record it in the project-level `CLAUDE.md`/agent instructions. Standalone
     scripts are written in this language from then on (see the `just`
     instruction module).
   - Record other standing choices there too: framework, database, auth
     approach.
2. Create the repo skeleton:
   - Enable the `just` instruction module (`agent-toolbox enable just`) and put
     a `justfile` at the repo root as the only task runner entry point. Every
     recurring operation added later (dev, test, deploy, launch-state, seed)
     becomes a recipe. Scripts invoked by recipes use the primary backend
     language per that module.
   - `README.md` written with `$audience-first-docs` for the product's target
     user, not for the developer.
   - `docs/guarantees/` and `docs/specs/freeform/` seeded per
     `$guarantee-maintainer` (sibling trees under `docs/`, not audience folders;
     see `$documentation-layout` / the documentation-layout instruction).
3. Create `portfolio.toml` at the repo root. This is the machine-readable
   registry entry the fleet board discovers and costs projects by:

   ```toml
   [project]
   slug = "myapp"                # unique across the portfolio
   name = "My App"
   status = "validating"         # idea | validating | building | launched | sunsetting | archived
   tier = "validation"           # validation | full
   created = "2026-07-07"
   decision_deadline = ""        # validate-idea decision date, if validating
   budget_monthly_usd = 0        # per-environment cost ceiling enforced in CI

   [domains]
   production = "myapp.example"
   staging = "staging.myapp.example"

   [digitalocean]
   project_production = "myapp-production"
   project_staging = "myapp-staging"
   tag = "product:myapp"

   [stripe]
   mode = "none"                 # none | sandbox | live

   [vendors]                     # which shared portfolio vendors this project consumes
   sendgrid = true
   posthog = true
   ```

4. Wire shared portfolio vendors (these are portfolio overhead, not new
   per-project accounts — reuse the existing accounts):
   - PostHog: one project **per product per environment** — staging and
     production each get their own project and `POSTHOG_API_KEY`/
     `POSTHOG_HOST` in their own GitHub Environment, never a single key
     reused across both (see `$plan-deploy-shared`). Capture from the first
     page. Reusing one key for both environments silently merges staging
     test traffic into production analytics — verify via the PostHog MCP
     (`projects-get`) that two distinct project ids exist before calling
     this step done.
   - SendGrid: one list/segment named by slug; confirmed opt-in from day one
     (every project maintains an email list of users).
   - Domain/DNS records managed in Terraform once infra exists; validation-tier
     pages prefer a subdomain of an existing portfolio domain.

   Validation tier stops here.

5. **Choose the runtime substrate — ask, do not assume.** This is a real fork
   with different modules, different skills, and different recurring cost. Put
   the question to the user before scaffolding anything infrastructure-shaped,
   and record the answer in `AGENTS.md` / `CLAUDE.md` so no later session
   re-asks:

   | Answer | Enable | Skill |
   |--------|--------|-------|
   | We provision it ourselves (DigitalOcean) | `continuous-delivery-shared` + `continuous-delivery-terraform` | `$plan-deploy-terraform` |
   | A managed platform runs it (Railway, Render, Fly) | `continuous-delivery-shared` + `continuous-delivery-paas` | `$plan-deploy-paas` |

   `continuous-delivery-shared` is enabled either way; the substrate modules
   are mutually exclusive.

   - If **self-provisioned**: choose the runtime shape via the matrix in
     `$plan-deploy-terraform` → `references/infrastructure-policy.md` (App
     Platform default; droplets + deploy agent only for named PaaS gaps). One
     DigitalOcean project per environment, named `<slug>-production` /
     `<slug>-staging`, every resource tagged `product:<slug>` and
     `env:<environment>` so cost stays attributable. Terraform workspaces named
     `staging` and `production`; state in Spaces; plans as PR comments.
   - If **managed platform**: ask **which one**, and record it. The comparison —
     pricing tiers, Terraform provider maturity, and the portfolio's current
     preference — is in `$plan-deploy-paas` →
     `references/platform-notes.md`. Do not default to whichever platform the
     last project used without checking that file.

   Either way:
   - Long-lived **`master`** git branch only — never `production`, `main`, or
     `staging`. Production is promote-addressed via **Trigger Promote to
     Production** / **Promote to Production** (cloud) or **Trigger Stable
     Release** / **Stable Release** (downloadable).
   - GitHub Environments named **`staging`** and **`production`** (never
     `prod`).
   - Cloud staging from day one for cloud products (cheapest practical scale,
     same substrate class as production). Local machines are **local dev**, not
     staging.
   - **Terraform owns the Stripe catalog regardless of substrate** — a managed
     platform running the app does not make the billing catalog dashboard-owned.
     That means remote, locked, encrypted Terraform state exists even for a PaaS
     project, because the state holds the Stripe API key. Set that up here, not
     later.
   - GitHub Actions CI/CD from the start: Deploy Staging on `master` push;
     production only through gated promote (cloud) or stable-release
     (downloadable) workflows — never branch-push to production, and never the
     platform's own auto-deploy-on-push.
6. Set up billing per `$implement-billing-chassis` when the product will
   charge: `plans.json`, sandbox Stripe in staging with key-mode validation,
   entitlement module, webhook endpoint, PostHog revenue events.
7. Add the operational floor before first production release:
   - `just launch-state` recipe (per `$plan-deploy-shared` references).
   - The experiment framework per `$implement-growth-experiments`: an (initially
     empty) `experiments.toml` registry and the PostHog event conventions, so
     pricing/ad experiments are runnable from day one and readable on the fleet
     board.
   - Seed script generating representative fake data for staging (staging never
     contains real customer data).
   - Uptime check once production is live; deploy annotations into PostHog.
8. Finish by verifying the skeleton: `just --list` shows the recipes, CI is
   green on the initial commit, `portfolio.toml` parses, and the README passes
   a `$persona-walkthrough`-style skim for the target user.

## Guardrails

- Do not scaffold full-tier infrastructure for an unvalidated idea; that is the
  main source of idle portfolio cost. Graduation is a deliberate step.
- Do not create new vendor accounts when a shared portfolio account exists;
  new fixed fees need explicit user approval.
- Conventions here are defaults, not law: when the user overrides one, record
  the override in the project's `CLAUDE.md` so future sessions stop re-asking.
