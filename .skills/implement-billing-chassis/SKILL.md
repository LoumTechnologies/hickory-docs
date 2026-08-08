---
name: implement-billing-chassis
description: Implement the billing, plans, and entitlement foundation of a SaaS product so pricing can be experimented on continuously and safely. Use when the agent is asked to add subscriptions, integrate Stripe checkout and webhooks, build a pricing page from plan data, enforce feature gates or quotas, wire PostHog flags/experiments into pricing display, handle upgrades/downgrades/dunning/cancellation, or make an existing billing setup experiment-ready. For choosing the pricing strategy use recommend-saas-pricing-strategy; for designing a specific experiment use optimize-saas-pricing.
---

# Implement Billing Chassis

## Goal

Build billing so that changing prices, plan names, tiers, and packaging is a
config-plus-flag change, not a refactor. Every project in the portfolio should
share this shape so `$optimize-saas-pricing` experiments can be implemented in
minutes and `$sunset-project` can wind billing down safely.

## Architecture Rules

Keep four concerns separate, all derived from one `plans.json`:

1. **Catalog** — Stripe products and prices, created by Terraform from
   `plans.json`. Prices are append-only: never mutate amount, currency, or
   interval on a used price; add a new price entry instead (see
   `$plan-deploy-shared` references for launch-state and migration rules).
2. **Display** — the pricing page renders a *plan set* from `plans.json`. Which
   plan set a visitor sees is resolved through a PostHog feature flag or
   experiment, defaulting to the `default` set. Never hard-code prices in
   templates.
3. **Assignment** — when a visitor starts checkout, persist the exact plan-set
   variant, plan id, and Stripe price id they saw. Checkout must charge the
   price that was displayed, even if the flag rolls out differently mid-session.
4. **Entitlements** — enforcement reads from the account's *purchased* price id
   mapped back through `plans.json` (including retired plan entries), never from
   the currently displayed grid. This makes grandfathering automatic: old
   customers keep the entitlements of the plan they bought.

## plans.json Shape

Keep the schema product-generic; the target repo defines its own entitlement
dimensions. Required capabilities:

- Multiple named plan sets (`default`, plus experiment variants), each an ordered
  list of plans with name, description, prices, trial rule, and entitlements.
- Every price carries an **immutable `key`** — a string the author never
  changes, distinct from the Stripe price id. This is what makes append-only
  implementable: Terraform keys resources by it (`for_each`, and Stripe's
  `lookup_key`), so adding or retiring a price never re-indexes its neighbours.
  Changing an amount means **a new key** (`…-v2`), never an edit. A price
  identified only by array position or by its Stripe id cannot be managed
  append-only; the id does not exist until after the first apply.
- Each price declares `kind`: `one_time` or `recurring`. `interval` is present
  only when `kind` is `recurring`. Do not model one-time prices as a recurring
  price with a null interval — Stripe treats the presence of a recurring block
  as the difference between a purchase and a subscription.
- Retired plans/prices stay in the file with `retired: true` so entitlement
  lookup and grandfathering keep working. Retiring sets Stripe `active = false`;
  it never deletes.
- Optional experiment block naming the PostHog flag key that selects the plan set.

**Plan sets are a display concept, and the catalog is their deduplicated union.**
Two sets routinely offer the same plan at the same price; a variant that only
reorders or renames plans must not mint a second Stripe object. Flatten every
plan set, dedupe by price key, and build the catalog from that. Then assert that
one key never appears with two different economics (amount, currency, interval,
per-seat flag, owning product) across sets — that collision is the "showed one
price, charged another" bug in latent form, and it should fail the plan.

**A Stripe price id is per environment, not global.** Staging uses a sandbox
account and production uses the live account, so the same logical price has two
different `price_...` ids and one string field cannot hold both. Make the field
accept either a bare string or a map keyed by environment:

```json
"stripe_price_id": { "staging": "price_abc", "production": "price_xyz" }
```

Two consequences worth building in from the start:

- A newly added price is committed with a **null** id and only gets one when
  Terraform applies, so structural validation of `plans.json` must not require
  ids. Check "every live price is materialized" separately, at service startup,
  for the current environment — that is where a missing id must fail loudly.
- Entitlement lookup maps a *purchased* price id back to a plan, and the stored
  purchase carries whichever id its environment used. Resolution must therefore
  search **all** environments' ids, not just the current one's.

### One-time and perpetual-licence products

The schema above was written for subscriptions. Downloadable products sold as
perpetual licences use the same file, with these additions — do not fork the
schema for them:

- **Seats are Checkout quantity, not tiers.** Set `seat_model:
  "checkout_quantity"` and `per_seat: true` on the price. Do not create 1–5 /
  6–20 / 21+ plans; they add edge cases and give buyers something to argue
  about. Create separate plans only when the tiers genuinely differ by
  *features*.
  - When the unit of value is a household or a site rather than a person, say so
    (`seat_model: "household"`, `seat_max: 1`) and grant devices as an
    entitlement. Charging per person for shared access to shared data prices the
    product's core benefit as an upsell.
- **The perpetual-licence entitlement is a version cap plus a time window.**
  `license_type: "perpetual"`, `max_major_version: N`, and
  `updates_included_months: M`. The licence keeps working forever, capped at
  major version N; updates beyond the window need a renewal. Enforcement
  compares the running build's major version against the *purchased* plan's
  `max_major_version`, resolved through plans.json exactly like any other
  entitlement — including for retired plans.
- **An update renewal is its own plan**, typically a recurring yearly price with
  `license_type: "update_window"`, `extends_updates_months`, and
  `raises_max_major_version`. It must be optional. A perpetual licence that stops
  working without the renewal is a subscription wearing a costume, and buyers who
  chose perpetual specifically to avoid that will treat it as a bait and switch.
- **Trials degrade, they do not disable.** Prefer a time-limited demo that
  becomes read-only over one that locks the user out of data they created. For
  tools reached for during an emergency (disk full, deadline), keep the core
  action working and expire the paid layer instead — being dead at the moment of
  need converts a prospect into a permanent non-customer.

## Implementation Workflow

1. Inspect what exists: plan constants, Stripe usage, webhook handlers, feature
   gates, account/subscription tables, pricing page, PostHog wiring. Reuse the
   repo's patterns; do not introduce a second billing path.
2. Create or normalize `plans.json` and the Terraform that materializes the
   Stripe catalog from it. Staging uses sandbox keys, production uses live keys,
   with deploy-time validation of key mode that fails rather than warns (per
   `$plan-deploy-shared`; the mechanism is `$plan-deploy-terraform` or
   `$plan-deploy-paas` depending on the substrate).
3. Implement checkout: server-side creation of Stripe Checkout sessions from a
   plan-set entry, carrying `{product_slug, plan_set, plan_id, price_id}` in
   session metadata.
4. Implement the webhook endpoint with idempotent handlers and signature
   verification, storing subscription state locally.

   **Let the database enforce idempotency; never write check-then-insert.**
   Put a UNIQUE constraint on the Stripe object that identifies the purchase
   (the checkout session id for one-time sales, the subscription id for
   recurring) and make fulfillment an upsert-or-return:
   `INSERT ... ON CONFLICT DO NOTHING RETURNING *`, then read the row back if
   nothing was returned. A concurrent redelivery blocks on the winner's
   uncommitted row and then finds it. A `SELECT` followed by an `INSERT` has a
   window between the two statements that two redeliveries both pass through,
   and it passes every sequential test.

   Return a flag saying whether *this* caller created the row, and gate side
   effects (the receipt email, the license key email, analytics) on it —
   otherwise a redelivery emails the customer twice.

   Also log every event by its Stripe event id and skip ids already seen, but
   treat that as a second layer, not the mechanism: Stripe can emit distinct
   event ids for the same object, so the event log alone does not prevent double
   fulfillment. **On handler failure, release the claim** (delete the log row) so
   Stripe's redelivery is retried rather than skipped as already-seen — a failed
   event that keeps its claim is silently dropped forever, meaning a customer
   paid and got nothing.

   Handlers to implement:
   - `checkout.session.completed` → activate subscription, record purchased
     price id and plan-set variant on the account.
   - `customer.subscription.updated` / `deleted` → sync status, handle
     downgrade/cancel at period end.
   - `invoice.payment_failed` → dunning state; restrict gracefully, never
     delete data on payment failure.

   For one-time purchases there is no subscription lifecycle:
   `checkout.session.completed` is the only event that matters, and it must
   record the purchased price id, the plan-set variant, and the **quantity**
   (the seat count). Quantity is easy to drop here and impossible to recover
   later — without it you cannot tell a one-seat purchase from a ten-seat one.
   `charge.refunded` should revoke or flag the licence.
5. Implement entitlement enforcement as one function/module that answers
   "can this account do X / how much of X" from stored subscription state plus
   `plans.json`. All gates call it; no scattered plan-name string checks.
6. Wire PostHog:
   - Flag/experiment lookup for plan-set selection on the pricing page.
   - Events with revenue and variant properties: `pricing_page_viewed`,
     `plan_selected`, `checkout_started`, `purchase_completed` (amount,
     currency, price id, plan set), `subscription_canceled`, `plan_changed`,
     `payment_failed`.
   - Identify accounts consistently so PostHog revenue analytics and experiment
     readouts work per `$optimize-saas-pricing`.
7. Build the customer-facing lifecycle surface: billing portal link (prefer the
   Stripe-hosted portal), upgrade/downgrade paths, cancellation that survives
   until period end, and receipt/dunning emails through the portfolio SendGrid
   account.
8. Test the chassis:
   - Unit tests for entitlement resolution including retired/grandfathered plans.
   - Webhook handler tests with replayed and duplicated events (idempotency),
     against a **real database**, not a mock — the guarantee under test is a
     unique constraint, and a mock will happily fake it.
   - **Test the redelivery race concurrently, and make sure the test actually
     races.** Two traps, both observed while building the licensing service:
     - Polling several futures from one task (Rust's `futures::join_all`,
       JS's `Promise.all` on a single-threaded runtime) interleaves them in a
       fixed order and does *not* race the database. A deliberately broken
       check-then-insert implementation still passed such a test. Use genuinely
       spawned tasks on a multi-threaded runtime, released together by a barrier.
     - A test that redelivers the *same event id* may be short-circuited by the
       event log before it ever reaches the fulfillment code, and so passes
       regardless. Also test **distinct event ids for the same purchase object**,
       which is what isolates the unique constraint.
   - Verify these tests fail against a deliberately broken implementation before
     trusting them. An idempotency test that has never gone red is a guess.
   - Stripe test clocks for trial expiry, renewal, and dunning where practical.
   - A checkout smoke test in staging against sandbox Stripe.

## Guardrails

- Never show one price and charge another; assignment persistence (rule 3) is
  what makes flag-based price experiments honest.
- Never mutate or delete a Stripe price that has ever been used; check
  launch-state before catalog changes.
- Existing customers stay on their purchased terms by default; migrations are an
  explicit, user-requested project.
- Webhooks are the source of truth for paid state, not checkout redirects; the
  success page must tolerate the webhook arriving late.
- No dark patterns: cancellation must be self-serve and no harder than signup.
