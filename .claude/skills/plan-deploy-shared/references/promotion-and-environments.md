# Promotion And Environments

The delivery path, independent of who provisions the infrastructure. The
substrate skill supplies only the highlighted deploy step.

## Canonical workflows — cloud products

| Workflow `name:` | Trigger | What it does |
|------------------|---------|--------------|
| **Deploy Staging** | Push to `master` | Bring the **staging** environment to the pushed commit, **[substrate deploy step]**, then run staging smoke/E2E. |
| **Trigger Promote to Production** | `workflow_dispatch` only, behind the **`production`** GitHub Environment | Human-approved gate: choose the ref (default: `master` tip) and a `bump` policy (`patch`/`minor`/`major`/`none`), then start **Promote to Production**. |
| **Promote to Production** | Started by the trigger (production-gated path) | **[substrate deploy step]** for the chosen ref, health-check, then stamp immutable `vX.Y.Z` when `bump != none`. |
| **Teardown Staging** | `workflow_dispatch` | Destroy staging on demand (optional cost control). |

The **[substrate deploy step]** is:

- **`$plan-deploy-terraform`** — `terraform apply` in that environment's
  workspace, then build/push an immutable image and deploy it (App Platform
  apply, or a call to the environment's deploy agent).
- **`$plan-deploy-paas`** — the platform's CLI or API deploying the ref to that
  platform environment. There is no apply phase, because there is no
  infrastructure state to reconcile.

File names may lag (`promote-production.yml`, `trigger-promote-production.yml`);
the workflow `name:` field is what operators see. Prefer names that say
**promote**, not **release**, for cloud. Reserve **Trigger Stable Release** /
**Stable Release** for downloadable products.

## Rules

- There is **no** `production`, `main`, or `staging` git branch, and **no**
  workflow that deploys production on a branch push.
- GitHub Environments are named **`staging`** and **`production`** — never
  `prod`.
- Branch protections and Actions jobs target **`master`** and PRs into it.
  Production protection is the `production` environment gate on **Trigger
  Promote to Production**, plus a required reviewer once the product is mature
  enough to warrant one. Do not add the reviewer before it is necessary.
- Every `vars.*` / `secrets.*` reference lives inside a job that names an
  `environment:`. A repo-level value is reachable from every branch and bypasses
  the production gate — see `config-and-environments`.
- Use **concurrency groups per environment** so two deploys cannot race.
- Use **immutable deploy identity** — normally the commit SHA. Never `latest`.
- **Deploy the exact commit that went green**, not whatever the branch points at
  by the time the deploy workflow starts.
- **Roll back** by re-running the promote path with `ref` set to an older
  tag/SHA and `bump: none`. Application rollback is clean; database schema and
  infrastructure changes do not auto-revert — plan those separately.

## Health checks and failure handling

- The promote path health-checks the deployed environment before it is
  considered successful, and the health URL is the same one the traffic-surge
  runbook tells an operator to watch.
- Design deploy operations so success is observable even if the app restarts
  networking components mid-deploy. Prefer an async job plus polling over one
  long streamed HTTP response.
- Failed deploy output should carry diagnostics — process/container state,
  recent logs, the deployed identity, and which health check failed — not just a
  non-zero exit.
