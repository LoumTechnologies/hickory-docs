# The Landing Beacon Forwards Only Declared Events With Scalar Properties

Given `POST /api/analytics/capture`, which is necessarily unauthenticated
(the visitors it measures have no account yet), when a request arrives, then
the server forwards it to PostHog only if the event name is on an explicit
allowlist, the anonymous `distinct_id` is 1–64 characters, there are at most
24 properties, every property value is a string, number, boolean, or null,
and no property name or string value exceeds 300 characters. Anything else is
rejected with 400 and a message naming both halves of the contract to update.
The browser never holds a PostHog credential.

When PostHog is unconfigured, capture is a no-op and the endpoint still
answers `{"accepted": true}` — a caller that read that as failure would retry
forever against a healthy server.

---

Last LLM verification:
- Date: 2026-08-08
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/server/src/routes/analytics.rs` — `ALLOWED_EVENTS`,
  `MAX_DISTINCT_ID`, `MAX_PROPERTIES`, `MAX_STRING`; `validate` and
  `check_property` enforce every clause above and their `ApiError::bad_request`
  messages name `ALLOWED_EVENTS` and the web-side `LandingEvent` union.
  Forwarding goes through `crate::analytics::Analytics::capture`, which is a
  no-op when `POSTHOG_API_KEY` is unset (`apps/server/src/config.rs`), while
  the handler returns `accepted: true` regardless. The route is registered in
  `apps/server/src/lib.rs` outside any auth extractor, and in
  `apps/server/src/openapi.rs` so `just codegen` types it for the web client.
  The browser side (`apps/web/src/analytics/sink.ts`) posts same-origin and
  reads no key.
- Test coverage: `apps/server/src/routes/analytics.rs::tests`
  (`accepts_a_declared_event_with_scalar_properties`,
  `rejects_an_event_name_that_is_not_on_the_allowlist`,
  `rejects_nested_property_values`, `rejects_an_oversized_property_or_id`,
  `rejects_a_property_bag_wider_than_the_cap`).
- Caveat requiring human review: the allowlist bounds *what* an untrusted
  caller can write, not *how much*. There is no rate limit on this endpoint,
  so a determined caller can inflate event counts for events that are on the
  list. That is acceptable pre-launch (see `.instructions/pre-launch.md`) but
  must be revisited before the funnel numbers inform spend — the mail
  endpoints' `ApiError::too_many_requests` path is the pattern to copy.
