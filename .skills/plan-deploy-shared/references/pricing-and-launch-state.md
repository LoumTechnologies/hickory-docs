# Pricing And Launch State

The rules here hold whether the catalog is materialized by Terraform or by a
sync script. The mechanism lives in the substrate skill —
`$plan-deploy-terraform` → `references/stripe-catalog-terraform.md`, or
`$plan-deploy-paas` → `references/stripe-catalog-script.md`.

## plans.json As Source Of Truth

Use a structured plan file, commonly `plans.json`, as the source for:

- Public plan names and descriptions.
- Payment-provider product identifiers and metadata.
- Recurring and one-time prices.
- Trial rules, annual discounts, coupons, and promotions if the product supports them.
- Feature entitlements, seats, usage limits, overage policy, and billing intervals.
- Migration/grandfathering metadata when plans evolve.

Keep the schema product-generic. Do not hard-code a specific product's limits
such as bandwidth, projects, seats, or storage into the skill. Let the target
repo define product-specific entitlement dimensions.

Some plans are **grandfathered**: still honored for existing subscribers, not
offered to anyone new, and therefore not advertised. That is a property in the
plan file, not a plan quietly deleted from it.

**Whatever materializes the catalog owns it.** Never create a product or price
by hand in the provider's dashboard, and never mint one imperatively from
application code at runtime. A dashboard-created price is invisible to the plan
file and will be silently clobbered or orphaned.

## Prices Are Append-Only

Stripe prices are append-only after use. A change to amount, currency,
interval, or tax behavior creates a **new** price and retires the old one,
keeping it available for existing customers. The only field that may change on
an existing price is `active`, which is the retire operation.

Make that structural rather than advisory, in whichever tool materializes the
catalog:

- Iterate over an **immutable price key** from the plan file, never a positional
  index — an unrelated edit must not re-key an existing price.
- Changing the amount for an existing key must report **no change**, not an
  update.
- Any remaining path to destroy-and-recreate must **hard-error**, not silently
  replace a used price.

Together these make changing a price *inexpressible*: the only available action
is to add a new key and retire the old entry. Accept the deliberate friction
that this also blocks removing a never-used price — tooling cannot distinguish
"never used" from "used once, long ago". Removing one is a deliberate operation
after checking usage with `just launch-state`.

## Sandbox vs live must fail, not warn

Staging and local dev use sandbox credentials; production uses live ones. The
deploy **fails** when the mode does not match the environment. A warning is not
a guarantee — this is the one integration where a mix-up charges real money.

Stripe encodes mode in the key prefix: `sk_test_` / `rk_test_` for sandbox,
`sk_live_` / `rk_live_` for live. Derive the mode from the key, derive what the
environment requires, assert they match, and make every catalog operation
depend on that assertion passing.

**Price ids are per environment.** Staging applies against a sandbox account and
production against the live account, so one logical price has two different
`price_...` ids. Store them keyed by environment
(`{"staging": "...", "production": "..."}`) rather than as a single string, and
have entitlement lookup search every environment's ids so a purchase resolves
regardless of where it was made. See `$implement-billing-chassis` for the full
shape.

**Do not make fulfillment depend on catalog metadata.** Provider metadata
support varies and is easy to lose. The reliable carrier is the **checkout
session** metadata the application sets when creating the session, which is also
what must be read back at fulfillment time. Use price `lookup_key` / `nickname`
to carry plan-file identity for human traceability only.

## Launch-State Command

Add a `just` command that answers "how launched are we?" before destructive
infrastructure, data, or billing changes.

```just
launch-state ENV="production":
    # ENV is staging | production — never "prod"
    # read plans.json
    # query the app database for real accounts and protected data
    # query the payment provider for subscriptions, payments, customers, price usage
    # print risk level and per-price usage
```

Report:

- Whether real user accounts exist.
- Whether real customer data exists or may exist.
- Whether the payment provider shows live customers, subscriptions, invoices,
  charges, payment intents, checkout sessions, or purchases.
- For each price in the current plan file: provider price id, product id,
  live/test mode, active flag, whether it has ever been used, current
  subscription count, historical purchase count, and recommended action.
- Whether each planned change is safe to apply, needs a new price, needs
  grandfathering, or needs manual review.

Use sandbox credentials for staging and live for production, and make the mode
obvious in the output.

**A command that cannot reach production must say what it could not check**
rather than reporting an absence it never verified. "No live subscriptions
found" and "could not query Stripe" are different answers, and only one of them
makes a teardown safe.

## Risk Levels

- `unlaunched`: no real users and no live purchases. Destructive changes may
  still need care, but no customer migration is implied.
- `soft-launched`: real accounts or test customers exist, but no live paid
  purchases. Protect user data; billing catalog changes are less constrained.
- `launched`: live purchases, active subscriptions, or real customer commitments
  exist. Avoid destructive changes, preserve used prices, plan grandfathering.

These map onto the `pre-launch` / `post-launch` instruction modules: reaching
`soft-launched` or `launched` is the signal to switch modules.

## Migration Rules

- Never delete a used price as the primary migration path.
- Prefer creating a new price and leaving existing subscriptions on old prices
  unless the user requests an explicit migration.
- If entitlements change for existing customers, document grandfathering
  behavior and the enforcement code paths.
- Keep public pricing, checkout creation, catalog materialization, and
  entitlement enforcement in sync from the same plan data.
