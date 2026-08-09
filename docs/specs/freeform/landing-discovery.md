# The discovery landing page

Status: built, not yet driven by traffic (2026-08-08).

One public page at the site root, whose job is **not** to convert as much as
possible. Its job is to let a segment nobody has named show up as a dense
cluster — so the shape of the page is dictated by what it needs to *learn*,
and conversion is one signal among several rather than the objective.

## Posture

**Discovery**, not confirmation. No segment has been discovered yet: there is
no persona library in this repo, no analytics history, and `experiments.toml`
is still empty. Confirmation pages — one per discovered segment, each fed by
its own campaign, whose identity chooser becomes a confirm-or-correct probe —
are the *next* posture, and are worth building only once discovery has
produced clusters worth confirming. The server already allowlists the
`segment_corrected` event so that step does not require touching the beacon.

## Why sections are titled by job, not audience

A section headed "For Platform Engineers" re-imposes our own guess about who
cares. A visitor who does not recognise the label skips a section that was
actually about their problem, and the resulting data confirms the guess we
started with. Titling by the job or pain — "Your README's examples stopped
working and nobody noticed" — lets people self-select by what they reach for,
and the set of sections a visitor opens becomes an interest vector we did not
predetermine.

The one identity question on the page is the deliberate exception: it asks
what the visitor claims to be, which is only worth collecting *because* it can
be compared against what they actually opened. It is quiet, one click, and
gates nothing — the moment it starts buying answers by pressure, the
comparison it exists for stops meaning anything.

## Instrumentation

Events, all defined in `apps/web/src/analytics/events.ts` and allowlisted in
`apps/server/src/routes/analytics.rs`:

| Event | Fires when | Key properties |
|---|---|---|
| `landing_viewed` | page load (once) | `path` |
| `interest_expanded` | section opened / closed | `interest_id`, `phase`, `open_index`, `dwell_ms` |
| `interest_clicked` | link inside a section followed | `interest_id`, `link_id` |
| `segment_declared` | identity anchor answered | `declared_segment` |
| `cta_clicked` | a call to action taken | `cta_id` |
| `segment_corrected` | *(confirmation posture — no UI yet)* | `from_segment`, `to_segment` |

Every event additionally carries `intended_segment`, `declared_segment`,
`referring_domain`, and the current `utm_*` values.

### Why a first-party beacon rather than a vendor SDK

The browser holds no analytics key. It posts to `/api/analytics/capture`, and
the server forwards using the `POSTHOG_API_KEY` it already has.

- **The deploy image stays promotable.** `import.meta.env.VITE_*` is inlined
  at `docker build` time (the `web` stage of `Dockerfile`), so a build-time
  key would bake one environment's project id into the image that the promote
  path then ships to the other. A runtime-configured server does not have
  this problem.
- **One canonical variable name set.** No `VITE_POSTHOG_KEY` twin of
  `POSTHOG_API_KEY` to keep in sync across GitHub Environments — which is what
  `.instructions/config-and-environments.md` asks for.
- **It keeps measuring.** A third-party analytics script is among the
  most-blocked requests on the web; a same-origin `/api` POST is not.

The cost is that the page gets no autocapture, no session replay, and no
client-side feature flags for free. None of those are needed to answer the
question this page exists to answer.

## What this page structurally cannot see

Naming these matters more than the numbers, because every one of them is a way
to mistake *measurable* for *true*.

- **Bounces before JavaScript runs.** Someone who opens the page and leaves
  inside a second may never fire `landing_viewed` at all. The denominator is
  systematically too small, so every rate computed from it is optimistic.
- **Do Not Track.** `apps/web/src/analytics/sink.ts` honours DNT and sends
  nothing for those visitors. That population is invisible by choice and is
  not distributed randomly — it skews technical, which is exactly our
  audience.
- **Ad and script blocking.** Same-origin posting survives more blockers than
  a vendor SDK would, but not all of them.
- **Selection bias in who arrives.** Paid or community traffic is a sample of
  whoever a channel's ranking chose to show it to. A dense cluster is
  evidence about *that channel's* audience before it is evidence about the
  market.
- **Dwell outside instrumented regions.** Time on the hero, on the pricing
  comparison, or scrolling past a section without opening it is not measured.
  A section with zero opens might have been read via its teaser.
- **The same person across devices.** Identity is a `localStorage` UUID, so a
  phone and a laptop are two visitors and clearing storage creates a third.
  First-touch attribution has the same boundary.
- **Why someone left.** Nothing here captures intent, only behaviour.
  Confusion is diagnosed by walking a persona through the page
  (`$persona-walkthrough`), not by reading this funnel.
- **No rate limiting yet** on the public beacon, so event counts are
  inflatable by an untrusted caller. Acceptable pre-launch; not acceptable
  once these numbers inform spend.

## Ready to probe

Entry URL: the site root (`/`, hash route `#/`) of whichever environment —
`just dev` locally, or the staging domain. Success signal for the discovery
run: a set of `interest_expanded` opens whose co-occurrence clusters do *not*
line up with the `declared_segment` options we guessed. That disagreement is
the finding; agreement mostly means the anchor's options taught the visitor
what to say.

This page has not been walked by a persona. Building and instrumenting it is
one hand's job; experiencing it as a human is `$persona-walkthrough`'s, and it
should be pointed at this URL before any spend goes behind it.
