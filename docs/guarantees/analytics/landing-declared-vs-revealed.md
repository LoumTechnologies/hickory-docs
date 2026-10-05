> Retired 2026-10-05. The homepage now contains one short document with a
> real browser debugger, paused on first load, and a desktop download section.
> Interest disclosures, the identity question, and the recorded-run demo are
> no longer on the homepage. The historical rationale below remains a record,
> not a guarantee of the current page. See `landing/homepage-opens-paused.md`.

# Declared Identity Is Recorded Separately From Revealed Interest, And Both
# Ride Every Event

Given the landing page's optional identity anchor, when a visitor answers it,
then a `segment_declared` event fires carrying the identity they just chose —
not the stale value read before the click — and that claim is stored and
attached as `declared_segment` to every later event from that browser.
Every event also carries `intended_segment` (what the campaign believed it was
buying, from first-touch `utm_campaign`), so any single event answers "did we
estimate this person correctly?" without joining sessions together. Both
properties are `$none` when unknown, never absent, so a query cannot silently
drop the unanswered case.

A visitor who never answers loses nothing: the anchor gates no content and no
call to action.

---

Last LLM verification:
- Date: 2026-08-08
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/web/src/analytics/events.ts` — `baseProperties` stamps
  `intended_segment`, `declared_segment`, `referring_domain`, and current
  `utm_*` on every event; `emit` special-cases `segment_declared` to overwrite
  `declared_segment` with the new claim *before* sending and only then calls
  `setDeclaredSegment`, which is what prevents the first declaration from
  being reported as `$none`. `apps/web/src/analytics/attribution.ts` persists
  the claim under `hickory.declared_segment`.
  `apps/web/src/views/LandingView.tsx` renders the anchor as an aside that
  collapses after one answer; both CTAs sit outside it.
- Test coverage: `apps/web/src/views/LandingView.test.tsx`
  (`records the declared identity and carries it on later events`,
  `asks for identity only once, and does not block anything on the answer`,
  `stamps intended segment as absent when no campaign named one`).
- Caveat requiring human review: the declared-vs-intended comparison is only
  meaningful for visitors who both arrived through a tagged link and chose to
  answer. In the discovery posture most visitors do neither, so this
  guarantee makes the comparison *possible*, not *representative*. The
  correction probe (`segment_corrected`) is allowlisted on the server but has
  no UI yet — it belongs to the confirmation posture, which has not been
  built. See `docs/specs/freeform/landing-discovery.md`.
