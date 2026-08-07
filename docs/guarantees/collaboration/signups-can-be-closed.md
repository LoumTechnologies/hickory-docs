# Signups Can Be Closed

Given `SIGNUP_ALLOWLIST` is set, when someone posts to `/api/auth/signup`
with an address it does not admit, then no account is created and the response
is 403 without revealing why. Given it is unset or blank, signup is open, so
existing deployments are unaffected.

An entry beginning with `@` admits a whole domain; anything else must match the
address exactly. Both sides are compared lowercased.

A public deployment of this product has three properties that are individually
defensible and jointly an invitation: signup is open, email addresses are never
verified (there is no mail capability in the codebase at all), and any account
can execute arbitrary code in containers. "We have no users yet" addresses the
wrong risk — the exposure is strangers scanning for exactly this shape, and
free compute running unverified code attracts miners within days.

This is deliberately the crudest gate that works. The real fix is email
verification, which does not exist and which team invitations will need
anyway. Until then an allowlist is one environment variable, and one
environment variable is much better than nothing.

The refusal is deliberately uninformative. A closed instance should not
confirm which addresses *would* be accepted, so the message names no address
and gives no reason.

The domain check splits on the LAST `@`, so `nate@example.com.evil.dev` does
not match an allowlisted `@example.com` — there is a test for that specific
lookalike, because the naive `ends_with` version of this check is wrong in
exactly that way.

---

Last LLM verification:
- Date: 2026-08-07
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/server/src/config.rs` — `parse_allowlist` lowercases and
  drops blanks, so an unset or whitespace value yields an empty list;
  `Config::signup_allowlist` carries it. `apps/server/src/routes/auth.rs` —
  `signup_allowed` returns true on an empty list, matches a `@domain` entry
  against the part after the final `@`, and otherwise compares the whole
  address; `signup` checks it after password validation and before the
  INSERT, returning `ApiError::forbidden` with a message naming nothing.
- Test coverage: `apps/server/tests/integration.rs`
  `the_signup_allowlist_admits_exactly_who_it_should` covers exact match,
  domain match, both lookalike near-misses, a non-member of an allowed
  domain, and the unset/blank open cases.
