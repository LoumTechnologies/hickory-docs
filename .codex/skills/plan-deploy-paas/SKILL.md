---
name: plan-deploy-paas
description: Plan, implement, or review staging and production delivery on a managed platform (Railway, Fly.io, Render, Heroku) where the platform owns provisioning of the runtime. Use when choosing between managed platforms on pricing and Terraform-provider maturity, deploying a SaaS product to a PaaS, wiring a platform CLI into GitHub Actions behind a production environment gate, configuring platform environments and managed Postgres, getting Stripe price ids from the Terraform-owned catalog into platform variables without copy/paste, compensating for having no reviewable infrastructure plan, or deciding whether a product has outgrown its PaaS. Always read alongside $plan-deploy-shared, which holds the branch policy, promote path, staging rules, and billing-catalog discipline this skill supplies mechanisms for. The alternative substrate skill is $plan-deploy-terraform.
---

# Plan And Deploy — Managed Platform (PaaS)

Read **`$plan-deploy-shared` first**. It defines the environments, the promote
path, staging parity, the billing-catalog rules, launch-state, and the
traffic-surge runbook. This skill only says how those are achieved when a
managed platform owns provisioning.

**A repo enables `continuous-delivery-paas` or `continuous-delivery-terraform`,
never both** — they are the two options on one axis. That is about who
provisions the **runtime**. It does not mean Terraform is absent: the Stripe
catalog stays Terraform-owned here (see below), so
`$plan-deploy-terraform`'s catalog reference is still the right thing to read
for that mechanism.

## What changes, and what does not

| | Terraform substrate | Managed platform |
|---|---|---|
| Runtime provisioning | `terraform apply` per workspace | The platform's dashboard/CLI/provider |
| Declared runtime state | Terraform state, reviewed as a plan in a PR | Usually **none** — see "Living without a plan" |
| Deploy step | Build/push image, apply, or call a deploy agent | Platform CLI |
| TLS | ACME, staging-first rollout | Managed by the platform |
| Database | Managed Postgres you provision | Managed Postgres the platform attaches |
| Cost attribution | Provider projects + resource tags | The platform's own project/environment grouping |
| **Stripe catalog** | **Terraform** | **Terraform** — unchanged |

**Everything in `$plan-deploy-shared` still applies unchanged**: no production
branch, gated promote, staging in the cloud at the same shape as production, no
real customer data in staging, append-only prices, per-environment analytics
projects, `just launch-state`, the surge runbook. A managed platform makes
provisioning easier; it does not relax any of those.

## Choosing the platform

**Which** managed platform is a deliberate choice made once, at scaffold time,
and recorded in the repo's agent notes — not inherited from whatever the last
project used. `$scaffold-new-project` asks; this skill supplies the comparison.

The two axes that actually decide it:

- **Terraform provider maturity.** The portfolio keeps the Stripe catalog in
  Terraform on every substrate, so Terraform is present regardless. A platform
  Terraform can *also* manage means price ids flow from `stripe_price.x.id`
  directly into a platform variable — no copy/paste, no drift, and platform
  config gets the same PR review surface as everything else. Render's provider
  is official and past 1.0; Railway's is community-maintained and pre-1.0
  (though it does cover variables).
- **Pricing shape, priced across both environments.** Plan fees differ in which
  direction they favor depending on scale, and both platforms bill compute on
  top. A staging environment plus a production one is the unit to price, not a
  single service.

Both, with figures and dates, are in `references/platform-notes.md` — along
with the portfolio's current preference, which is the first thing to check.

## Environments

- Create **two platform environments** named `staging` and `production` — never
  `prod`, matching the GitHub Environment names.
- Each gets its own managed database. Never point staging at the production
  database, not even read-only: staging sends real email and runs unreleased
  migrations.
- Each gets its own domain, its own vendor keys, and its own analytics project.
- Application configuration lives in **the platform's environment variables**,
  one set per platform environment, using the canonical names from
  `config-and-environments` — same names, different values, no `STAGING_*`
  twins.
- **Deploy credentials** (the platform API token, service names) live in the
  matching **GitHub Environment**, not in repo-level secrets, so the production
  token is behind the production gate.
- Maintain the canonical inventory (`docs/operators/ENVIRONMENTS.md`) and a CI
  check that fails when a workflow reads a name that is not listed, or reads one
  from a job with no `environment:`.

## Deploy workflows

The workflow table in `$plan-deploy-shared` →
`references/promotion-and-environments.md` is unchanged; the substrate deploy
step is a platform CLI invocation.

```yaml
# Deploy Staging — push to master, after CI is green
jobs:
  deploy:
    environment: staging          # required: puts vars/secrets behind the gate
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.event.workflow_run.head_sha }}   # the commit that went green
      - run: <platform-cli> up --service "$SERVICE" --environment staging
        env:
          PLATFORM_TOKEN: ${{ secrets.PLATFORM_TOKEN }}
```

Rules specific to this substrate:

- **Disable the platform's own Git auto-deploy** if it can deploy production
  from a branch push. That is exactly the ungated path `$plan-deploy-shared`
  forbids, and it silently bypasses the `production` environment gate. Deploys
  come from GitHub Actions or nowhere.
- **Deploy the commit CI went green on**, not the branch tip.
- Health-check after deploying, before calling the promote successful.
- Roll back by re-running the promote workflow with an older ref and
  `bump: none`. Most platforms also offer a dashboard rollback — prefer the
  workflow so the deployed identity stays traceable to a ref.
- Use concurrency groups per environment.

## Living without a plan

The real cost of a managed platform is that **there is no reviewable diff of
infrastructure changes and no state to reconcile**. A setting changed in the
dashboard at 2am leaves no trace. Compensate deliberately:

1. **Write the intended state down.** `docs/operators/ENVIRONMENTS.md` for
   variables and secrets; a short "platform layout" section for services, their
   build/start commands, attached databases, domains, and resource sizes.
2. **Review drift on a schedule**, not never. Once a release, compare the
   dashboard against the document. Treat an undocumented difference as either a
   documentation bug or an unauthorized change — decide which.
3. **Prefer the platform's config-as-code file when it has one**
   (`fly.toml`, `render.yaml`, a Railway config). It is not Terraform, but a
   committed file is reviewable and a dashboard toggle is not.
4. **Never let a fix live only in the dashboard.** If an incident is resolved by
   changing a setting there, the same change lands in the document (or config
   file) in the same session, or it will be lost at the next rebuild.

State this gap explicitly in the repo's agent notes rather than leaving a future
reader to assume a plan exists.

## Stripe catalog — still Terraform

`$plan-deploy-shared` requires that `plans.json` is the source of truth, that a
sandbox/live mismatch **fails** rather than warns, and that prices are
append-only. **A managed platform running the app says nothing about who owns
the billing catalog**, and Terraform enforces those rules structurally where a
script only approximates them. So the default here is the same as on any other
substrate: Terraform owns the Stripe catalog, per `$plan-deploy-terraform` →
`references/stripe-catalog-terraform.md`.

Consequences worth planning for at scaffold time:

- **Remote, locked, encrypted Terraform state exists even on a PaaS**, because
  the state holds the Stripe API key. This is the one piece a "just use a
  platform" setup tends not to have.
- **Getting price ids into the app.** Best to worst:
  1. Terraform sets the platform's environment variable directly from
     `stripe_price.x.id` — no copy/paste, and a dashboard edit is reverted on
     the next apply. Needs a platform provider good enough to trust; see
     `references/platform-notes.md`.
  2. A deploy step reads `terraform output -json` and pushes the values through
     the platform CLI. No provider dependency; nothing detects drift.
  3. Skip moving ids entirely — Terraform sets a stable `lookup_key` on each
     price and the app resolves ids by that key. Adding a plan then needs no new
     variable and no redeploy, and the per-environment id problem disappears
     because each account returns its own id for the same key. Resolve lazily
     with a cache, never at boot, so a missing or unreachable payment
     credential degrades to a no-op instead of blocking startup.

Only when a repo genuinely has no Terraform at all does the fallback apply: an
idempotent sync script carrying the same guarantees, written in the repo's
primary backend language and run from the same gated workflow that deploys. See
`references/stripe-catalog-script.md`, which also states what that form does not
give you.

## Managed data services

- Use the platform's managed Postgres for both environments. Take its backup
  defaults seriously enough to check them — verify the retention window and that
  a restore has actually been tested before there is anything to lose.
- Connection limits are the most common first constraint. Horizontal scaling
  multiplies connection-pool size, so scaling out can *cause* database
  exhaustion rather than relieve it. Set an explicit pool size per instance
  rather than letting the ORM default couple pool size to replica count.
- Migrations run through the app's own migration path on deploy, the same way in
  both environments.

## When to leave

A managed platform is the right default for a small SaaS. Reconsider when:

- Per-unit cost at steady load clearly exceeds equivalent self-managed capacity,
  and the load is predictable enough that the comparison is real.
- The product needs something the platform does not offer — raw TCP/UDP, custom
  TLS or wildcard certificate handling, host networking, privileged containers,
  a specific region or compliance boundary.
- Undocumented dashboard drift has caused a real incident more than once.

Migrating means adopting `$plan-deploy-terraform` and dropping this skill.
Because `$plan-deploy-shared` holds the branch policy, promote path, and catalog
rules, that migration changes the deploy mechanism and not the delivery model.

## References

- `references/stripe-catalog-script.md` — the **fallback** for repos with no
  Terraform: an idempotent `plans.json` → Stripe sync approximating the
  append-only and key-mode guarantees Terraform gives structurally.
- `references/platform-notes.md` — **the portfolio's current platform
  preference**, the Terraform-provider and pricing comparison used to choose
  one, then per-platform vocabulary, known traps, and running cost.
