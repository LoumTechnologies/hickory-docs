> Retired 2026-10-05. The homepage now contains one short document with a
> real browser debugger, paused on first load, and a desktop download section.
> Interest disclosures, the identity question, and the recorded-run demo are
> no longer on the homepage. The historical rationale below remains a record,
> not a guarantee of the current page. See `landing/homepage-opens-paused.md`.

# Opening An Interest Section Is Recorded, With Dwell On Close

Given the discovery landing page, when a visitor opens an interest section,
then exactly one `interest_expanded` event fires with `phase: "open"`, that
section's permanent `interest_id`, and the `open_index` of its position on the
page; and when they close it, a second `interest_expanded` fires with
`phase: "close"` and the `dwell_ms` it stayed open. A section that is open
when the page is hidden (a closed tab, a backgrounded mobile browser) flushes
its close event on `visibilitychange`, so "read it and left" stays
distinguishable from "never opened it".

Section bodies are hidden until opened, which is what makes an open a
deliberate act and therefore a usable interest signal rather than a
measurement of scroll depth.

---

Last LLM verification:
- Date: 2026-08-08
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/web/src/components/InterestSection.tsx` — `toggle` emits the
  open event with `open_index` and the close event with a `Date.now()` delta
  from `openedAt`; the `useEffect` registers a `visibilitychange` listener
  while open that flushes the close event once and clears `openedAt` so it
  cannot double-fire. The body `<div>` carries `hidden={!open}` and the
  heading button carries `aria-expanded`/`aria-controls`.
  `apps/web/src/landing/interests.ts` documents `id` values as permanent
  because renaming one silently splits a funnel.
- Test coverage: `apps/web/src/views/LandingView.test.tsx`
  (`records which interest was opened, and the dwell when it closes`,
  `reveals the body only once the section is opened`).
- Caveat requiring human review: the `visibilitychange` flush is not covered
  by a test — jsdom does not model the hidden/visible transition faithfully
  enough for the assertion to mean anything. It needs a live check in a real
  browser, which is a natural addition to the Playwright substrate described
  in `.claude/skills/audit-feature-test-coverage`. Until then, treat
  `dwell_ms` totals as a lower bound.
