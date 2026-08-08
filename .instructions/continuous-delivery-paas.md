---
skills:
  - plan-deploy-paas
---

# Continuous Delivery — managed platform (PaaS)

Enable this **alongside `continuous-delivery-shared`**, which holds the branch
policy, workflow names, promotion rules, staging-data rules, billing-catalog
discipline, and analytics rules that apply no matter who provisions the
infrastructure. This module only adds what is specific to a managed platform —
Railway, Fly.io, Render, Heroku — owning provisioning.

**This module and `continuous-delivery-terraform` are the two options on one
axis. Enable exactly one.** Nothing in a repo on this module should grow a
`cloud/` Terraform tree.

## Choosing the platform

**Which** platform is decided once, when the project is scaffolded, and recorded
in the repo's `AGENTS.md` / `CLAUDE.md` so no later session re-litigates it.
Never inherit the choice from whatever the previous project used.

The comparison — Terraform provider maturity, pricing across both environments,
and the portfolio's current preference — lives in `$plan-deploy-paas` →
`references/platform-notes.md`. The short version: Render's Terraform provider
is official and past 1.0 while Railway's is community-maintained and pre-1.0,
which matters because **Terraform is in the repo either way** (see below);
pricing favors different platforms at different scales, and both bill compute on
top of the plan fee.

## Rules

- **Terraform still owns the Stripe catalog.** A managed platform running the
  app does not make the billing catalog dashboard-owned — the rules in
  `continuous-delivery-shared` (generated from `plans.json`, append-only prices,
  sandbox/live mismatch fails rather than warns) are best enforced by Terraform
  on this substrate too. That means remote, locked, encrypted Terraform state
  exists even here, because the state holds the payment provider's API key.
- **Prefer letting Terraform manage the platform's own configuration** where the
  platform has a usable provider. The payoff is concrete: price ids flow from
  the catalog into platform environment variables with no copy/paste, dashboard
  edits get reverted on the next apply, and platform config becomes reviewable
  in a PR. Where the provider is missing or too immature to trust, the fallbacks
  are a deploy step that pushes `terraform output` values through the platform
  CLI, or resolving prices by `lookup_key` so the ids never need to move at all.
- **Staging and production are separate platform environments**, named
  `staging` and `production` (never `prod`), matching the GitHub Environment
  names. Each gets its own managed database, domain, vendor keys, and analytics
  project. Staging never points at the production database, not even read-only.
- **Deploys come from GitHub Actions, never from the platform's own
  branch-watching auto-deploy.** Auto-deploy has no `production` environment
  gate, so it silently bypasses the promote path `continuous-delivery-shared`
  requires. Turn it off for production; if it stays on for staging, make sure it
  is not racing the Deploy Staging workflow.
- **Deploy credentials live in the matching GitHub Environment** (platform API
  token, service names), never as repo-level secrets — see
  `config-and-environments`.
- **Application configuration lives in the platform's environment variables**,
  one set per platform environment, under the canonical names from
  `config-and-environments`. Same names, different values, no `STAGING_*` twins.
- **Write the intended platform state down**, because there is no plan to review
  and no state to reconcile. Keep the variable and secret inventory in
  `ENVIRONMENTS.md`, and a short record of services, build/start commands,
  attached databases, domains, and sizes. Prefer the platform's own committed
  config file (`fly.toml`, `render.yaml`, a Railway config) wherever one exists.
- **A fix made in the dashboard is not done until it is written down.** An
  undocumented dashboard change survives until the next rebuild and then
  vanishes; treat drift found later as either a documentation bug or an
  unauthorized change, and decide which.
- **The billing catalog is generated from `plans.json`**, never by hand and
  never by application code at runtime. Terraform is the default owner (above);
  an idempotent sync script is the **fallback** for a repo with no Terraform at
  all. Either way it must fail — not warn — on a sandbox/live key mismatch, and
  must refuse to mutate or delete a published price. See `$plan-deploy-paas` →
  `references/stripe-catalog-script.md` for the script form and what it does not
  give you compared with Terraform.
- **TLS, backups, and scaling are the platform's**, which does not make them
  someone else's problem: confirm the certificate covers every domain in use,
  confirm the backup retention window, and **test a restore** before there is
  data worth losing.

## Outgrowing the platform

Reconsider the substrate when per-unit cost at steady, predictable load clearly
exceeds equivalent self-managed capacity; when the product needs something the
platform cannot offer (raw TCP/UDP, custom TLS handling, host networking,
privileged containers, a specific region or compliance boundary); or when
undocumented dashboard drift has caused a real incident more than once.

Migrating means swapping this module for `continuous-delivery-terraform`.
Because `continuous-delivery-shared` carries the branch policy, promote path,
and catalog rules, that migration changes the deploy mechanism and not the
delivery model.
