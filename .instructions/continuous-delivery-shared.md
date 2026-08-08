---
skills:
  - plan-deploy-shared
---

# Continuous Delivery — shared rules

**Always** use continuous delivery, even from the very beginning of a project.

This module holds every continuous-delivery rule that does **not** depend on how
the infrastructure is provisioned. Enable it in every repo. Then enable exactly
**one** substrate module alongside it — they are the two options on a single
axis, and enabling both is a configuration error:

- `continuous-delivery-terraform` — **we** own the infrastructure and describe
  it as code (DigitalOcean, Terraform workspaces, remote state, plans in PRs,
  Terraform-owned Stripe catalog, ACME/TLS rollout).
- `continuous-delivery-paas` — a **managed platform** owns provisioning
  (Railway, Fly.io, Render, Heroku). Every rule below still applies; only the
  mechanism differs.

Also enable `continuous-delivery-downloadable` for products shipped as
downloads. That is a separate axis — product shape, not substrate — so a
downloadable product still enables this module, plus a substrate module only if
it actually owns infrastructure (e.g. a Stripe catalog or licence service).

## Branch policy

- There must **always** be a long-lived **`master`** branch.
- There must **never** be a `production`, `main`, or `staging` branch.
  Production has **no branch** — it is a manually triggered promote path,
  Heroku-style.
- **Never** enable Github Actions jobs on arbitrary branches; they should only
  exist on `master` or on PRs into `master`.

## Canonical workflow names — cloud products

**Do not** use downloadable-product “Stable Release” / “Trigger Stable Release”
names for cloud workflows. Cloud Actions UI names are **promote** language;
**release** language is reserved for downloadable products (see
`continuous-delivery-downloadable`).

| Workflow `name:` (Actions UI) | Trigger | What it does |
|-------------------------------|---------|--------------|
| **Deploy Staging** | Push to `master` | Ungated auto-deploy to the **cloud** staging environment + live E2E. Staging stays as similar to production as possible (real TLS, separate secrets, same substrate class). Never host staging on a developer machine. |
| **Trigger Promote to Production** | `workflow_dispatch` only, `production` GitHub Environment gate | Human-approved gate: choose the ref to promote (default: `master` tip), optional version `bump` (`patch` / `minor` / `major` / `none`), then start **Promote to Production**. With `bump: none` and an older ref/tag, redeploy without cutting a new tag. |
| **Promote to Production** | Started by the trigger workflow (or an equivalent production-gated path) | Deploy the chosen ref to production infra, health-check it, then stamp immutable `vX.Y.Z` when `bump != none`. Idempotent enough to re-run for rollback with an older ref and `bump: none`. |
| **Teardown Staging** | `workflow_dispatch` | Destroy staging infra on demand (optional cost control). |

File names may lag (`promote-production.yml`, `trigger-promote-production.yml`);
the workflow `name:` field is what operators see. Prefer names that say
**promote**, not **release**, for cloud.

## Promotion rules

- Github Actions must automatically deploy `master` to the cloud staging
  environment (**Deploy Staging**) on every update.
- There **must** be a **Trigger Promote to Production** workflow
  (`workflow_dispatch` + `production` GitHub Environment gate) that leads to
  **Promote to Production** as above. As the software matures, a required
  reviewer on that environment is appropriate; do not add it until necessary.
- **Never** automatically promote to production based on PRs merged or branches
  being pushed to. Staging deploys are automatic; production is always
  deliberate.
- The promote path should create an immutable tag (when `bump != none`), using
  the semantic version derived from the explicit bump.
- **Roll back** by re-running **Trigger Promote to Production** (then **Promote
  to Production**) with `ref` set to an older tag/SHA and `bump: none`.
  App-image rollback is clean; infrastructure and DB-schema changes do not
  auto-revert.

## Staging fidelity and data

- Staging is always **cloud-hosted**. A machine on the developer's laptop is the
  **local dev environment**, never "local staging."
- Staging uses the same resource *types* as production at the cheapest scale.
  Staging does not load test.
- **Never put real customer data in staging** — staging sends real emails.
  Evaluate production data for its messy shapes (NULLs, missing values,
  mistyped entries) and maintain a seed script that reproduces them
  representatively.

## Billing catalog

- **`plans.json` is the single source of truth** for pricing plans, including
  grandfathered plans that are inaccessible to new users and therefore not
  advertised. The billing catalog at the payment provider is **generated from
  it** — never created by hand in the provider's dashboard, and never minted
  imperatively from application code at runtime.
- **Staging and local dev must use the payment provider's sandbox; production
  must use live credentials** — and the deploy must **fail**, not warn, when the
  wrong kind of key is present for the environment. This generalizes the
  sandbox rule in `config-and-environments` to the one integration where a
  mix-up charges real money.
- **Prices are append-only.** Key each price by an immutable identifier and
  never mutate or destroy a published one; superseding a price means adding a
  new key and leaving the old one addressable for existing subscribers.

Whichever substrate module is enabled supplies the enforcement mechanism —
Terraform `precondition` + `prevent_destroy` under
`continuous-delivery-terraform`, or an idempotent sync script with the same
preconditions on a PaaS.

## Analytics and release correlation

- Provision **one product-analytics project per product per environment**
  (staging and production never share one, the same split as sandbox/live
  payment keys). Inject each environment's keys through the matching GitHub
  Environment per `config-and-environments`.
- **Send a deploy annotation from the CD pipeline** so metric changes can be
  correlated with releases.
- Before believing staging and production are separated, actually query the
  analytics provider and confirm two distinct project ids exist and that recent
  events from each environment's domain land in the matching one — the same
  variable name existing in both GitHub Environments does not prove the
  *values* differ.

## Operational readiness

- Maintain **`just launch-state`** (see `pre-launch` / `post-launch`) so any
  destructive infrastructure or billing change can first be checked against
  whether real users and real money are involved yet.
- Maintain a **traffic-surge runbook** at
  `docs/operators/runbooks/traffic-surge.md` with the product's real numbers:
  what saturates first, the no-spend levers, the exact scale-up commands with
  per-month and pro-rated-per-day cost, whether each step drops traffic,
  temporary-vs-permanent revert steps, and which monitoring alert means
  "execute this now." An operator under load pastes commands; they do not read
  infrastructure code.
