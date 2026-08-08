# Stripe Catalog Without Terraform — the fallback

**Read this only when the repo genuinely has no Terraform.** The default on a
managed platform is still a Terraform-owned catalog — see
`$plan-deploy-terraform` → `references/stripe-catalog-terraform.md`. A managed
platform running the *app* says nothing about who should own the *billing
catalog*, and Terraform enforces the rules structurally where a script only
approximates them.

`$plan-deploy-shared` → `references/pricing-and-launch-state.md` states those
rules. Terraform enforces them with `precondition`, `ignore_changes`, and
`prevent_destroy`. Without Terraform you write the enforcement yourself, and it
is easy to write something that *documents* the rules instead of enforcing them.

The test of this script: **changing a price in `plans.json` must be
inexpressible**, not merely discouraged.

## Shape

One idempotent command, in the repo's primary backend language, runnable from a
gated workflow:

```just
sync-billing-catalog ENV:
    # ENV is staging | production — never "prod"
```

It reads `plans.json`, compares it to the provider's current catalog, and makes
only additive changes. Run it in the same gated workflow that deploys that
environment, before the app is deployed, so a plan the app expects always
exists.

## Guarantee 1 — key mode must fail, not warn

Before any request that could create or modify a catalog object:

```
mode      = key starts with sk_live_ / rk_live_  -> "live"
            key starts with sk_test_ / rk_test_  -> "test"
            otherwise                            -> "unrecognized"
required  = ENV == "production" ? "live" : "test"

if mode != required: abort with a non-zero exit before any API call
```

Two things matter here and are easy to get wrong:

- **Abort before the first write**, not on the first mismatch you notice partway
  through. A partially-applied catalog against the wrong account is worse than
  no catalog.
- **`unrecognized` is a failure**, not a fallback to test. A restricted key with
  an unexpected prefix must stop the run rather than be guessed at.

Also assert that `ENV` is one of `staging` / `production` — a missing argument
must fail, never default to production.

## Guarantee 2 — prices are append-only

For each plan entry, keyed by an **immutable price key** from the plan file:

| Situation | Action |
|-----------|--------|
| Key exists in the file, no matching price at the provider | **Create** it, record the id |
| Key exists in both, all fields match | Nothing |
| Key exists in both, **amount/currency/interval/tax differs** | **Abort with an error** naming the key and telling the author to add a new key and mark the old one retired |
| Key marked retired in the file, price still active | Set `active = false` — the only permitted mutation |
| Price exists at the provider, key absent from the file | **Abort** — never delete. Report it for human review |

The third row is the whole point. A script that "updates" a changed price
silently breaks existing subscribers; a script that *refuses* forces the author
into the append-only workflow. Make the error message say exactly what to do:
add a new key, mark the old one retired, re-run.

Never match prices by position or by iteration order. Match by the immutable
key, carried on the price's `lookup_key` (and `nickname` for human
readability), because provider metadata support is inconsistent.

## Guarantee 3 — ids are recorded per environment

One logical price has a different `price_...` id in sandbox and live. Write the
resulting ids back keyed by environment:

```json
{ "priceIds": { "staging": "price_abc", "production": "price_xyz" } }
```

Store this where the application can read it, and have entitlement lookup search
**every** environment's ids so a purchase resolves regardless of where it was
made. Do not overwrite one environment's ids with another's — a sync run for
staging must not touch the production entries.

## Guarantee 4 — dry run by default in review

Give the command a mode that prints what it *would* do without doing it, and run
that on pull requests. It is the closest available equivalent to a Terraform
plan comment, and it is the only chance a human gets to see a catalog change
before it reaches a live account.

## What you still do not get

Be honest in the repo about the residual gap versus Terraform:

- No state file, so a price created by hand in the dashboard is invisible until
  the script's "exists at provider, absent from file" check catches it — which
  is why that check must abort rather than warn.
- No locking. Two concurrent runs against the same environment can race; use the
  workflow's concurrency group to prevent it.
- No plan/apply separation beyond the dry-run mode you wrote.

Check the residual gap with `just launch-state` before any billing change, per
`$plan-deploy-shared`.
