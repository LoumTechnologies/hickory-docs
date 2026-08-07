# An Unproven Address Cannot Run Code

Given a deployment with email configured, when an account whose address has
not been confirmed tries to run or check a document or start an agent, then it
is refused with a 403 naming the endpoint that sends a new link. Reading and
writing documents stay open to that account.

The gate is on **execution**, not on login. Someone waiting on a slow mail
should be able to sign in, look around, and write — those cost nothing and
reach nobody. Running code costs compute, and compute is the lever an abuser
actually wants, so that is where proof of the address is required.

A deployment with no `SENDGRID_API_KEY` cannot require what it cannot send, so
the check is a no-op there. One unset environment variable must not make the
product unusable, which is the same way Stripe and PostHog already degrade.

Three details carry more weight than they look:

**Tokens are stored as SHA-256, never in plaintext.** A database read cannot
be replayed as a verification or a password reset, exactly as with
`password_hash`. SHA-256 rather than argon2 because these are 256 bits of
CSPRNG output — there is nothing to brute-force, and argon2's deliberate
slowness would only make redemption slow.

**Redemption is a single atomic UPDATE ... RETURNING**, conditioned on unused
and unexpired. SELECT-then-UPDATE would leave a race in which two concurrent
redemptions of one reset link both succeed — two people setting a password on
one account.

**Purpose is part of the lookup.** A verification link cannot be redeemed as a
password reset. Without that, anyone who obtained a verification link — from a
forwarded message, a mail archive, a shared inbox — could take the account.

Password reset always answers 200 whether or not the address exists, because
anything else turns it into an account-existence oracle. Sends are capped at 5
per hour per user per purpose: uncapped, the endpoint is a free mail cannon
aimed at any address an attacker names, and a sending domain's reputation does
not recover on a useful timescale.

---

Last LLM verification:
- Date: 2026-08-07
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/server/src/runs.rs` `check_email_verified` returns Ok when
  the mailer is unconfigured or the user is verified, and is called before
  `check_exec_quota` on both the run/check path (`start_run`) and the agent
  route. `apps/server/src/email_tokens.rs` — `hash_token` (SHA-256 hex),
  `issue` expires outstanding tokens of the same purpose in one transaction,
  `redeem` is a single conditional `UPDATE ... RETURNING` keyed on hash AND
  purpose. `apps/server/src/routes/auth.rs` — `request_reset` returns the same
  body for known and unknown addresses; `MAX_SENDS_PER_HOUR = 5`.
  Migration `0005_email_verification.sql` grandfathers existing accounts as
  verified, since they predate the claim.
- Test coverage: `apps/server/tests/integration.rs` —
  `an_unverified_account_can_read_and_write_but_not_run`,
  `verifying_an_address_unlocks_execution` (including the second redemption
  failing), `a_verification_token_cannot_reset_a_password`,
  `password_reset_round_trips_and_never_reveals_who_has_an_account` (asserts
  the known and unknown responses are byte-identical and that only one mail
  goes out), `verification_sends_are_rate_limited`. All run against a
  `CapturingMailer`, so the suite never contacts a provider.
