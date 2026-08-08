---
skills:
  - scaffold-new-project
  - validate-idea
---

# Start with the real delivery path, then mature controls

Work through demand validation first when the idea is unproven (`$validate-idea`,
validation-tier scaffold). Once you are building a real product (full tier), use
the delivery path below from day one — do not invent a temporary branch model
and swap it later.

## Cloud products

- Day one of full tier: long-lived **`master`** branch, **cloud staging**
  environment, and automatic **Deploy Staging** on `master` updates. There is
  **no** `production`, `main`, or `staging` branch.
- Production is a **gated** path (**Trigger Promote to Production** →
  **Promote to Production**), not a branch you push to. Ship to the real
  production domain when you deliberately promote — not by growing a second
  long-lived branch. Use promote language for cloud Actions jobs, not
  “Stable Release” (that name is for downloadable products).
- Local machines are the **local dev environment**, never "local staging."
  Staging is always in the cloud for cloud services.
- As the product matures, add controls around that path (required reviewers on
  the `production` GitHub Environment, stricter budgets, more parity checks).
  Do not delay creating cloud staging until "later."

## Downloadable products

- Always start with official **stable** releases (major and minor may be zero,
  e.g. `0.0.1`, `0.0.2`).
- Use the word **unstable** (not "edge" or "prerelease") for the non-stable
  channel. Implementation may still use GitHub `prerelease: true` as a mechanic.
- As the product matures, keep **Unstable Release** automatic on `master` and
  **stable** behind **Trigger Stable Release**. See
  `continuous-delivery-downloadable`.

By the time a public launch matters, staging (cloud) or unstable (downloadable)
plus a deliberate stable/production gate should already be the normal path.
