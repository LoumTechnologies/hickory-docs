# Platform Notes

Per-platform specifics: how to choose one, then vocabulary and traps. The rules
they implement are in `$plan-deploy-shared`.

**Verify against current documentation before relying on any detail here.**
Managed platforms change pricing, limits, and CLI surfaces frequently, and a
stale detail in a skill file is worse than no detail. Every figure below carries
the date it was recorded — treat anything older than a few months as a starting
point for a fresh check, not as fact.

---

## Current portfolio preference

**None recorded yet — decide per project** (as of 2026-08-05).

Update this section as experience accumulates. When a preference forms, say
*why* in one line, because the reason ages differently than the conclusion: "we
prefer X because its Terraform provider is official" stops applying the moment
the other one ships an official provider.

`$scaffold-new-project` reads this section when asking which platform a new
project should use.

## Choosing a platform

### Terraform provider maturity

This matters more than it looks, because the portfolio keeps the Stripe catalog
in Terraform regardless of substrate. Once Terraform is in the repo anyway, a
platform Terraform can also manage means **price ids flow from
`stripe_price.x.id` straight into a platform environment variable with no
copy/paste and no drift** — and platform config gets the same review surface as
everything else.

| Platform | Provider | Status (checked 2026-08-05) |
|----------|----------|------------------------------|
| **Render** | `render-oss/render` | **Official**, past 1.0 (v1.7.5). Resources include `render_web_service` and `render_env_group`. |
| **Railway** | `terraform-community-providers/railway` | **Community**, pre-1.0 (v0.6.2, Apr 2026). Resources: `project`, `environment`, `service`, `variable`, `shared_variable`, `variable_collection`, `custom_domain`, `service_domain`, `tcp_proxy`. Railway publishes no official provider. Full coverage needs an **account-level user token**, a broad credential to hand CI. |

Render's provider being official and past 1.0 is a genuine advantage. Railway's
is usable and covers variables — which is the resource that matters most here —
but it is pre-1.0, community-maintained, and a stalled maintainer becomes your
problem.

If you would rather not depend on either, `$plan-deploy-shared` →
`references/pricing-and-launch-state.md` describes resolving prices by
`lookup_key` instead, which removes the need to move price ids into the
platform at all.

### Pricing

Recorded 2026-08-05 from the vendors' published plans. **Both bill compute on
top of the plan fee**, so the monthly figure is a floor, not an estimate.

**Render**

| Plan | Fee | Includes |
|------|-----|----------|
| Hobby | **$0** + compute | Up to 25 services; 5 GB bandwidth; single-service previews; global regions & CDN; custom domains; firewall & DDoS mitigation; database PITR; chat support |
| Professional | **$20/mo** + compute | All Hobby features, plus unlimited seats & services; 25 GB bandwidth; full-stack previews; horizontal autoscaling; isolated environments; OIDC authentication; AWS Private Links; workspace audit logs |

**Railway**

| Plan | Fee | Includes |
|------|-----|----------|
| Trial | **$0** | 30-day trial with $5 credits, then $1/mo. Up to 1 vCPU / 0.5 GB RAM per service; 0.5 GB volume storage; community support |
| Hobby | **$5/mo** + compute ($5 credit included) | Up to 48 vCPU / 48 GB RAM per service; up to 5 replicas at 8 vCPU / 8 GB RAM each; up to 5 GB storage; single developer workspace; community support; 99.9% availability target; 7-day log history; global regions |
| Pro | **$20/mo** + compute ($20 credit included) | Up to 1,000 vCPU / 1 TB RAM per service; up to 42 replicas at 24 vCPU / 24 GB RAM each; up to 1 TB storage; unlimited workspace seats; Railway support; 99.99% availability target; 30-day log history; concurrent global regions |

Reading the difference:

- **At the bottom end Render is cheaper**: Hobby is $0 + compute against
  Railway's $5/mo. For a pre-launch product with a small staging environment,
  that gap is most of the bill.
- **At $20 the comparison inverts**, because Railway's $20 *includes* $20 of
  compute while Render's $20 is a plan fee on top of compute. Whether that
  favors Railway depends entirely on actual usage.
- **Two environments, not one.** Staging plus production doubles compute, and
  the per-plan service and seat limits apply to the total. Price the pair.
- **Isolated environments and horizontal autoscaling are Professional-only on
  Render**, and Render's Hobby tier gives single-service previews only. If the
  staging/production split needs to be a first-class isolated environment rather
  than a second service, that pushes Render to $20.
- **Availability targets differ by tier on Railway** (99.9% Hobby, 99.99% Pro).
  Neither is a meaningful SLA pre-launch; both matter post-launch.

Do not treat the cheaper plan fee as the cheaper platform. Compute dominates
once anything real is running, and both bill it separately.

### Other considerations

- **Committed config.** Render (`render.yaml`) and Fly (`fly.toml`) express more
  of their configuration as committed files than Railway does, which directly
  reduces the drift problem described in the skill.
- **Migration cost is low but not zero.** Both run containers behind a managed
  proxy with managed Postgres, so moving is mostly re-creating services and
  variables. What does not move automatically: the database (dump and restore,
  with downtime), domains and DNS, and any platform-specific config file.

---

## Vocabulary

| Concept | Railway | Fly.io | Render |
|---------|---------|--------|--------|
| Deployable unit | Service | App / Machine | Service |
| Environment split | Environments within a project | Separate apps (`-staging` suffix) | Separate services, or a preview environment |
| Config as code | Config file per service | `fly.toml` (committed) | `render.yaml` blueprint (committed) |
| Managed Postgres | Postgres plugin | Fly Postgres / managed offering | Render Postgres |
| CLI deploy | `railway up --service … --environment …` | `flyctl deploy --app …` | Deploy hook or API |

Where a committed config file exists, use it — see the drift discussion in the
skill.

## Common traps

**Auto-deploy on push.** Every one of these platforms can watch a branch and
deploy on its own. That path has no `production` GitHub Environment gate, so it
violates the promote rule in `$plan-deploy-shared` while looking convenient.
Turn it off for production. If you leave it on for staging, make sure it is not
*also* racing the Deploy Staging workflow — pick one.

**`PORT` is assigned, not chosen.** The platform injects it and routes to it.
The app must read it from the environment and bind `0.0.0.0`, not a hardcoded
port and not loopback. A service that binds `127.0.0.1` passes local dev and is
unreachable when deployed.

**Database connections are the first ceiling.** Managed Postgres tiers have low
connection limits, and each application instance opens a pool. Scaling replicas
multiplies connections — the scale-up that was supposed to help can exhaust the
database instead. Set an explicit pool size per instance and note the arithmetic
in the traffic-surge runbook.

**Build environment ≠ runtime environment.** Frontend build-time variables
(`VITE_*`, `NEXT_PUBLIC_*`) are baked into the bundle at build time, so changing
one requires a **rebuild**, not a restart. Anything baked into a browser bundle
is public — never a secret.

**Ephemeral filesystems.** Anything written to local disk is lost on redeploy
unless a volume is explicitly attached. Uploads go to object storage.

**Sleeping and cold starts.** Free and low tiers may idle a service to sleep.
A sleeping staging service makes post-deploy smoke checks flaky, which reads as
a broken test rather than a sleeping service — give health checks a generous
first-request timeout, or keep staging warm.

**Backups.** Confirm the retention window and **actually test a restore** before
there is data worth losing. "The platform handles backups" is an assumption
until someone has restored one.

## Running cost, once chosen

Plan fees are in "Choosing a platform" above; this is about what happens after.

Compute is billed per-minute or per-second for what is actually running, which
makes the traffic-surge runbook's temporary-bump advice genuinely cheap: a week
one tier up costs little, and reverting is immediate. It also means an idle
staging environment still costs money — a Teardown Staging path (or scaling
staging to zero between uses) is a real lever, per `$plan-deploy-shared` →
`references/staging-parity.md`.

Attribute cost per product per environment using the platform's own project and
environment grouping, and record the expected monthly figure somewhere the next
invoice can be checked against.
