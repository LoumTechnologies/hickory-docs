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

## The one demo, and the section above it

*Revised 2026-08-13. The page previously carried three demos; two of them
showed capabilities this product does not have (a hosted issue-tracker
integration, two people editing one document) and were removed rather than
relabelled — see `local-only.md`. The page's focus also moved: the primary
claim is now the tool surface a visitor's **own** coding agent drives, because
that is the reason to install this rather than a thing it also does.*

Above the interest sections the page makes its case in three parts — why,
then how, then proof:

1. **The claim** (`components/Convergence.tsx`) — three muted cards (the
   session, the reasoning, the edit) collapsing into one accented card (a
   `.hick` document in your repository). The shape carries the argument on its
   own, so a reader who only skims the headings still gets it.
2. **The mechanism** (`components/AgentToolSurface.tsx`) — static, not
   interactive: `hick init`, `hick mcp`, the five `hick doc` tools, and the
   artifact they leave behind (`HICKORY_SESSION`, `hick ingest --from session`). A fake
   terminal pretending to run an agent would only obscure how small the real
   surface is.
3. **The proof** (`landing/demos/ProgramDemo.tsx`) — one document, its woven
   files, and its executable cells, driven in the visitor's own browser with
   no account. It exists because parts 1 and 2 are promises nobody should take
   on trust: this is where a visitor checks them.

*Revised again 2026-08-13, and this is the more important of the day's two
revisions.* The first pass led with content-hash edit anchoring as though it
were the innovation. It is not — applying an edit to the right span is a
problem every serious harness solved years ago, and pitching it as a
breakthrough reads as years late to exactly the reader who would install this.
The anchors are now an aside at the end of part 2: load-bearing, unremarkable,
and present only because a record whose edits landed somewhere other than where
they claim is worth nothing. What the page leads with instead is the
convergence, which is genuinely not on offer anywhere else, and
`docs/guarantees/landing/the-page-leads-with-the-claim-not-the-mechanism.md`
holds that ordering in place.

The demo is the page's main claim and its main risk, so two rules govern it:

- **The mechanism is real.** It parses, weaves, derives provenance and draws
  ribbons with the same modules the desktop app uses (`lib/weave`,
  `lib/ribbons`, `lib/ribbonGeometry`) — see
  `docs/guarantees/landing/home-demos-run-the-real-mechanism.md`. A recorded
  animation would be cheaper and would make the page a liar the first time a
  visitor tried to reproduce what they saw.
- **What is faked says so on screen.** The execution transcript is a
  recording, because a browser tab cannot start a container, and it says so
  next to itself along with what `hick run` and `hick test` do with the same
  cells on the visitor's machine. Nothing on the page contacts a third-party
  service.

A third rule now sits beside them and covers the whole page rather than the
demo: every command, flag and hick fragment shown must be true of the shipped
binary — `docs/guarantees/landing/home-page-claims-are-true-of-the-binary.md`.
That guarantee exists because this page spent months describing a hosted
workspace that had been deleted, and nobody noticed.

The demo does **not** play itself. Autoplay existed for the six-step
walkthrough, where a visitor who would not click anything still needed the
story; a single split editor shows its picture on arrival with nothing to
advance through. So `demo_engaged` no longer carries a meaningful `autoplay`,
and its instrumentation value simplifies: opening an interest section costs a
click, driving the demo costs effort, and the first interaction is reported
once.

## Instrumentation

Events, all defined in `apps/web/src/analytics/events.ts`. There is no
server-side allowlist any more — `apps/server` went with the hosted product
and the site posts to PostHog directly (`local-only.md`):

| Event | Fires when | Key properties |
|---|---|---|
| `landing_viewed` | page load (once) | `path` |
| `interest_expanded` | section opened / closed | `interest_id`, `phase`, `open_index`, `dwell_ms` |
| `interest_clicked` | link inside a section followed | `interest_id`, `link_id` |
| `segment_declared` | identity anchor answered | `declared_segment` |
| `cta_clicked` | a call to action taken | `cta_id` |
| `demo_engaged` | the home-page demo was driven (first interaction only) | `demo_id`, `step` |
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
