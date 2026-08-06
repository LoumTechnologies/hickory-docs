# Stripe Webhook Fulfillment Is Idempotent Under Concurrent Redelivery

Given a Stripe `checkout.session.completed` webhook, when the same purchase
object is delivered multiple times — including concurrently and with
*distinct* Stripe event ids — then exactly one fulfillment happens (one
subscription row, one plan upgrade, side effects fired once), because the
database enforces it with a UNIQUE constraint plus
`INSERT ... ON CONFLICT DO NOTHING RETURNING`, never check-then-insert; and
when a handler fails, its event-log claim is released so Stripe redelivery
retries instead of being dropped.

---

Last LLM verification:
- Date: 2026-08-05
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `apps/server/src/routes/billing.rs` — `checkout_completed` claims
  fulfillment via `INSERT INTO subscriptions ... ON CONFLICT
  (stripe_subscription_id) DO NOTHING RETURNING` (PK on the subscription id,
  `apps/server/migrations/0001_init.sql`) and gates all side effects on the
  claim being created by *this* caller; the `stripe_events` id log is a
  second layer only and is deleted on handler failure. The race test was
  verified to go red against a deliberately broken check-then-insert
  implementation (SELECT + sleep + INSERT) before trusting it.
- Test coverage: `apps/server/tests/integration.rs` —
  `webhook_fulfillment_races_to_exactly_one`: six spawned tasks on a
  multi-thread runtime released by a barrier, distinct event ids for the
  same subscription, asserting one subscription row, one plan change, all
  event ids logged, duplicate-id short-circuit, signature rejection, and
  dunning/cancel restricting without deleting.
