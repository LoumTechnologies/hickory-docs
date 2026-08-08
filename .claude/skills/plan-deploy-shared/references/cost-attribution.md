# Cost Attribution

The portfolio goal: many products live simultaneously, each mostly idle, with
recurring cost near zero until a product earns dedicated spend. Delivery
decisions optimize for the fleet, not the single project.

The *mechanism* for attribution is substrate-specific — cloud-provider projects
and resource tags under `$plan-deploy-terraform`, the platform's own
project/environment grouping under `$plan-deploy-paas`. The rules below are not.

## Project Tiers

- **Validation tier** — static landing page, analytics, an email list, nothing
  else. No infrastructure stack of its own, no staging environment, no
  database, no billing. See `$validate-idea`.
- **Full tier** — the complete delivery shape: cloud **staging** from day one
  (long-lived `master` + Deploy Staging), production only via the gated promote
  path, no production branch. Entered deliberately via `$scaffold-new-project`
  after validation, or on explicit request.

Do not build full-tier infrastructure for an unvalidated idea, and do not leave
a validated product on validation-tier scaffolding.

## Shared Overhead vs Per-Project Cost

**Shared portfolio overhead** — fixed vendor fees paid once for all products,
each with an included allowance: transactional email, product analytics, a
shared edge or DNS product, a wildcard/portfolio domain, shared state or archive
storage.

**Per-project cost** — this product's own compute, database, and staging; its
dedicated domain; its payment-processing fees; and any vendor allowance it
consumes *beyond* the shared free allowance.

Rules:

- A project's marginal cost **excludes** shared fixed fees; those are recovered
  at the portfolio level.
- When a project's usage approaches a shared allowance ceiling (emails, events,
  bandwidth), that is a pricing or graduation signal — **surface it**, do not
  silently upgrade the shared plan.
- **Never create a second account for a shared vendor to dodge attribution.**
  Attribute usage instead: per-product email categories or subusers,
  per-product analytics projects, per-product resource tags.
  - Note the deliberate exception: one analytics project **per product per
    environment** is required separation, not a duplicate account. That is about
    keeping staging events out of production numbers, not about spinning up a
    second organization.
- Keep staging cheap with size and retention (see `staging-parity.md`). Do not
  eliminate cloud staging or redefine a local dev environment as staging to save
  money.

## What Must Be True

Whatever the substrate, spend must be computable per product per environment
without a human reconstructing it from a bill:

- Every resource is grouped under something named for the product **and** the
  environment.
- Naming is predictable enough to filter on (`<slug>-<environment>-…`).
- A per-product budget expectation exists somewhere, and exceeding it is
  visible before the invoice rather than after.

If the substrate cannot express one of these, say so explicitly in the repo
rather than leaving the gap implicit.
