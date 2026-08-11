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

## The three demos

Above the interest sections the page runs the product, in the visitor's own
browser, with no account:

1. **Knowledge work** — a six-step walkthrough in which meeting notes become
   Jira issues, the lineage between note and ticket is drawn as a Sankey, and
   an edit made on the ticket is carried back into the note it came from.
2. **Software** — a document that explains an algorithm and *is* the program,
   tangled into a module and its test from one shared fragment.
3. **Collaboration** — two live CRDT clients on one document, with a git strip
   underneath.

They are the page's main claim and its main risk, so two rules govern them:

- **The mechanism is real.** The demos parse, weave, derive provenance and
  draw ribbons with the same modules the signed-in workspace uses
  (`lib/weave`, `lib/ribbons`, `lib/ribbonGeometry`) — see
  `docs/guarantees/landing/home-demos-run-the-real-mechanism.md`. A recorded
  animation would be cheaper and would make the page a liar the first time a
  visitor tried to reproduce what they saw.
- **What is faked says so on screen.** No demo contacts a third-party
  service: the Jira half and the git remote are simulated, and both are
  labelled next to the thing that is faked rather than in a footnote.

The walkthrough can also **play itself**. A visitor who will not click
anything still gets the story, and the ones who take the wheel are the signal
— which is why `demo_engaged` carries `autoplay` and autoplay stops before the
hands-on steps rather than clicking past the part it is trying to get someone
to try.

Their instrumentation value is different in kind from the interest sections':
opening a section costs a click, driving a demo costs effort. `demo_engaged`
with `autoplay: false` is therefore the page's strongest revealed-interest
signal, and *which* demo someone drives is a segment hypothesis in itself — the knowledge-work
walkthrough and the literate-programming one describe the same mechanism to
two populations that may share nothing else.

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
| `demo_engaged` | a home-page demo was driven | `demo_id`, `step`, `autoplay` |
| `segment_corrected` | *(confirmation posture — no UI yet)* | `from_segment`, `to_segment` |

Every event additionally carries `intended_segment`, `declared_segment`,
`referring_domain`, and the current `utm_*` values.

### Two deliveries, and why the beacon stopped being the only one

*Revised 2026-08-11 (`local-first.md`): the marketing site is a static file
with no server behind it, so a same-origin beacon has nothing to post to.*

Delivery is chosen by configuration, not by a branch in the page:

- **Direct to PostHog** when `VITE_POSTHOG_KEY` is set — a project write key
  (`phc_…`) can only send events, so inlining it is safe. This is what a static
  deployment uses.
- **The server's beacon** (`/api/analytics/capture`) when it is not, which is
  how the hosted app has always worked.

The original argument for the beacon alone was that `import.meta.env.VITE_*` is
inlined at build time, so a key would bake one environment's project into an
image the promote path ships to another. With a single production environment
there is nothing to promote, and that objection no longer holds. The two
arguments that survive, and are the reason the beacon is kept rather than
deleted:

- **It keeps measuring.** A third-party analytics request is among the
  most-blocked on the web; a same-origin `/api` POST is not. A static site pays
  this cost — some visitors will simply not be counted.
- **A deployment that prefers no key in its bundle should not have to have
  one.** The key is public by construction, which is fine for a write-only
  project key and is still a choice worth leaving open.

Guarded in two places, because a *personal* API key (`phx_…`) can create and
destroy projects: `just site` refuses to build with one, and `src/config.ts`
refuses to use one — logging and disabling capture rather than throwing, since
a white-screened marketing page is a worse outcome than a missing event.

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
- **Whether a demo taught anything.** `demo_engaged` says how far someone
  got, not whether they understood it. Someone who clicks to step 6 and
  leaves confused is indistinguishable here from someone who got it — that
  question belongs to `$persona-walkthrough`, not to this funnel.
- **Phones see a different page.** Below 950px the three-column Sankey stacks
  and the ribbon layer is hidden, because a diagram drawn across 40px
  channels would misrepresent the product. Mobile `demo_engaged` counts are
  therefore not comparable with desktop ones.
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
