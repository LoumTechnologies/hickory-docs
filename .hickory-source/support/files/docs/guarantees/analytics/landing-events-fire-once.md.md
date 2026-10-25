<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/analytics/landing-events-fire-once.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `7d5c3d45f5ffe82acd8a43947901273b515ae4bc2fa83888b2fa73a9e4da2be2`.

<book:entry path="docs/guarantees/analytics/landing-events-fire-once.md">
# A Visit Is Reported Exactly Once Per Page Load

Given the homepage, when it mounts, then exactly one
`landing_viewed` event fires for that page load, regardless of how many times
React re-renders or re-mounts the view. In particular, React's StrictMode
double-mount in development must not produce two events.

---

Last LLM verification:
- Date: 2026-10-05
- Reviewer: Codex
- Result: verified
- Evidence: `apps/web/src/views/LandingView.tsx` — the `viewedFired` flag is
  module-scoped rather than a `useRef`, because a ref is recreated by the
  second StrictMode mount and would not suppress anything.
  `apps/web/src/main.site.tsx` renders inside `<StrictMode>`, so this is a live
  concern in `npm run dev`, not a hypothetical. `resetViewedForTest` exists
  solely so a test can assert the property from a clean slate.
- Test coverage: `apps/web/src/views/LandingView.test.tsx`
  (`reports the visit once per load`).
- Caveat requiring human review: the flag is per JavaScript context, so a
  genuine second visit within the same SPA session (navigating away to
  `#/pricing` and back to `#/`) is deliberately *not* re-reported. That is
  correct for "how many visits did this page get" and wrong for "how many
  times did someone return to it"; if the second question ever matters it
  needs its own event rather than a change here.
</book:entry>

</book:doc>
