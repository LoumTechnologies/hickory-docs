# Third-Party Integration Degradation Matrix

For each third-party integration the app uses, decide and document the
behavior in all three columns below. "Log/no-op" is the default; a mock is
an explicit upgrade from that default, made only when justified.

| Integration | Credential present | Credential absent (has account, not configured locally) | No internet at all |
|---|---|---|---|
| SendGrid (email) | Sends for real | Logs the would-be email (subject + recipient) at info level, returns success to the caller | Same as absent — never blocks on a network call that isn't there |
| Stripe (billing) | Talks to Stripe (sandbox key in dev, per `config-and-environments`) | Billing-gated features degrade to their free/default behavior; log what would have happened | Same as absent |
| Twilio (SMS) | Sends for real (sandbox number) | Logs the would-be message | Same as absent |
| PostHog (analytics) | Captures events for real | No-ops the capture call | Same as absent |
| S3-compatible storage | Talks to the real bucket | **Candidate for a deliberate mock** (see below) rather than log/no-op, because "no-op" would mean uploaded files silently don't exist, breaking any feature that reads them back | Same as absent — the mock (if chosen) doesn't need network either |

## Why S3 is usually the exception

Most integrations are fire-and-forget from the app's perspective (send an
email, fire an event) — a log/no-op is a complete, sufficient
implementation of "gracefully degrade." Storage is different: something
downstream (an image tag, a download link, a re-processing job) usually
needs to **read back** what was written. A pure no-op breaks that read path
even when nothing about the current task cares about S3 itself. That's the
concrete justification the instruction module's "deliberate decision"
requirement is asking for — write it down when it applies, don't assume it
generalizes to every integration.

## Local-folder-backed S3 mock

When the justification above applies, prefer a mock that writes to a local
directory (e.g. `.dev-env/s3-mock/<bucket>/<key>`) rather than an in-memory
fake or a containerized S3-compatible service:
- A developer can `ls`/open the file directly to see what got "uploaded" —
  no separate tool or API call needed to inspect state.
- No extra container, so it doesn't add anything to `docker-compose.yml` or
  the dependency list.
- Cleaned up by `just dev-clean` like any other local dev state.
- Implement it behind the same interface/client the app uses for the real
  SDK, selected by whether credentials are configured — the app code should
  not know it's talking to a mock.

## Adding a new integration

When wiring a new third-party integration into the dev environment, add a
row to this table (or the project's own copy of it) before writing the
integration code — deciding present/absent/offline behavior up front is
cheaper than retrofitting graceful degradation after the fact.
