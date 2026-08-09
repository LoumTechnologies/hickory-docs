# Landing Attribution Credits The First Touch, And Survives The Hash Router

Given a visitor arriving with UTM parameters in the query string
(`https://…/?utm_source=hn&utm_campaign=launch#/`), when the landing page
reports any event, then those parameters are read from the query string —
which sits before the fragment, so the hash router never consumes them — and
the *first* tagged arrival is recorded permanently as that visitor's first
touch. A later visit from a different source reports its own `utm_*` values
but does not overwrite the first touch or the derived `intended_segment`. An
untagged arrival never stamps a first touch, so opening the bare domain
before clicking an ad does not permanently credit the visitor as direct.

`intended_segment` is `utm_campaign` from the first touch, or `$none` — which
is its normal value in the discovery posture, where no segment has been named
yet. Over-long UTM values are truncated to the beacon's 300-character limit in
the browser rather than being rejected as a 400.

---

Last LLM verification:
- Date: 2026-08-08
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/web/src/analytics/attribution.ts` — `readUtm` parses
  `location.search` (documented at the call site as being before the hash) and
  slices values to 300; `firstTouch` returns early on an empty current UTM set
  so untagged loads never stamp; `attribution` derives `intendedSegment` from
  `firstTouch.utm_campaign` first. `apps/web/src/analytics/events.ts`
  (`baseProperties`) puts `intended_segment` — `$none` when absent — plus the
  current `utm_*` and `referring_domain` on every emitted event. Storage
  access is wrapped so a private-mode browser degrades to a per-load identity
  instead of throwing.
- Test coverage: `apps/web/src/analytics/attribution.test.ts`
  (`reads UTM parameters from the query string, which the hash router never sees`,
  `truncates an over-long UTM value to the beacon's limit rather than being rejected`,
  `keeps the first touch when a later visit arrives from somewhere else`,
  `does not stamp an untagged visit as the first touch`,
  `reports no intended segment in the discovery posture`).
- Caveat requiring human review: first touch is stored in `localStorage`, so
  it is per-browser. The same person on a phone and a laptop is two visitors
  and cannot be joined; clearing storage resets attribution. This is a real
  and permanent limit of client-side attribution, recorded as a declared
  unknowable in `docs/specs/freeform/landing-discovery.md`.
