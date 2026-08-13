# Driving The Home-Page Demo Is Reported As Revealed Interest

Given a visitor on the home page, when they run the executable cells, edit the
document, or edit a generated file, then a `demo_engaged` event is sent
carrying `demo_id` (which demo) and `step` (what they did), alongside the
attribution and declared-segment properties every landing event carries. It is
sent for the **first** interaction only, so the count means "this visitor tried
the product" rather than "this visitor typed n characters".

This is the strongest revealed-interest signal the page has: opening an
interest section costs a click, but driving the demo costs effort, and it
separates people who read the page from people who tried the product. It is
therefore comparable against the identity they declared, the same way
`interest_expanded` is.

Nothing about the demo is gated on the event being delivered — a visitor with
analytics blocked drives it exactly as far.

The `autoplay` property is retained in the event type but is no longer sent.
The page's one demo does not play itself, so there is no timer tick that could
be miscounted as engagement. The field stays on the wire only so a historical
query that splits on it still parses.

---

Last LLM verification:
- Date: 2026-08-13
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `demo_engaged` is a member of the `LandingEvent` union in
  `apps/web/src/analytics/events.ts`. `emit` stamps every event with
  attribution and declared segment via `baseProperties()`. The only emitter is
  `apps/web/src/landing/demos/ProgramDemo.tsx`, whose `engage` helper returns
  early once `engaged` is set — so `run-cells`, `edited-document` and
  `edited-output` compete to be the one event, and whichever the visitor did
  first is what is reported. Delivery is fire-and-forget in
  `apps/web/src/analytics/sink.ts`; the demo does not await it.
  There is no longer a server-side allowlist to keep in step: `apps/server` was
  deleted with the hosted product, and the site posts to PostHog directly
  (`docs/specs/freeform/local-only.md`).
- Test coverage: `apps/web/src/landing/demos/demos.test.tsx`
  (`records engagement once, naming the demo and what was done`).
- Caveat requiring human review: `step` is a free string on the wire, and
  nothing mechanically stops a future edit renaming one of the three values and
  silently splitting a funnel in two.
