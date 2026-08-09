# Driving A Home-Page Demo Is Reported As Revealed Interest

Given a visitor on the home page, when they advance the walkthrough, edit a
generated artifact, or use the git strip, then a `demo_engaged` event is sent
carrying `demo_id` (which demo), `step` (how far in they got), and `autoplay`
(whether a timer got them there or they did), alongside the attribution and
declared-segment properties every landing event carries. The event is not sent
when the visitor is already on the step they clicked, so the count means
"reached this step" rather than "clicked something".

The `autoplay` split is the whole point of recording it: the walkthrough can
play itself, and counting a timer tick as engagement would inflate exactly the
number that is supposed to mean "this person tried the product". Autoplay
stops at the hands-on steps and on any manual click, so a session that reaches
the round trip reached it deliberately.

This is the strongest revealed-interest signal the page has: opening an
interest section costs a click, but driving a demo costs effort, and it
separates people who read the page from people who tried the product. It is
therefore comparable against the identity they declared, the same way
`interest_expanded` is.

Nothing about the demos is gated on the event being delivered — a visitor with
analytics blocked drives them exactly as far.

---

Last LLM verification:
- Date: 2026-08-09
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `demo_engaged` is a member of the `LandingEvent` union in
  `apps/web/src/analytics/events.ts` and is on `ALLOWED_EVENTS` in
  `apps/server/src/routes/analytics.rs` — the two halves of the contract the
  server rejects drift between. `emit` stamps every event with attribution and
  declared segment via `baseProperties()`. Emitters:
  `apps/web/src/landing/demos/KnowledgeWorkDemo.tsx` (`advance`/`jumpTo`, both
  of which return early when the step does not change, and the `round-trip`
  emit when an output edit resolves back into the document),
  `ProgramDemo.tsx` (first interaction only), and `CollaborationDemo.tsx`
  (`typed`, `commit`, `push`, `pull`). Delivery is fire-and-forget in
  `apps/web/src/analytics/sink.ts`; no demo awaits it.
- Test coverage: `apps/web/src/landing/demos/demos.test.tsx`
  (`reports which step the visitor reached`,
  `does not report a step the visitor was already on`,
  `separates a demo that was watched from one that was driven`,
  `offers to play itself, and hands control back at the hands-on step`) and,
  for the server half, `apps/server/src/routes/analytics.rs::tests`
  (`rejects_an_event_name_that_is_not_on_the_allowlist`).
- Caveat requiring human review: `step` is a free string on the wire. The
  walkthrough's values come from `KNOWLEDGE_STEPS[].id`, which the file's
  header comment marks as permanent, but nothing mechanically stops a future
  edit renaming one and silently splitting a funnel in two.
