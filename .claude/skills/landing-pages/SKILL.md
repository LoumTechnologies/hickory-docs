---
name: landing-pages
description: >
  The web construction hand: build and instrument landing pages in whatever stack the
  target repo already uses. Use when asked to build a discovery landing page (single,
  interest-organized, reveals segments as clusters) or confirmation pages (one distinct
  page per discovered segment with controlled traffic), instrument on-page
  identity/engagement/conversion, or report page-level construction reality back to
  strategy. This is a mechanism hand driven by $maximize-market-learning; it consumes a
  page spec and returns the upward contract (events, feasibility, declared unknowables).
  Uses $implement-growth-experiments for PostHog/UTM plumbing and consumes a visual
  system from $branding. It does **not** probe pages with personas — hand a built page
  to $persona-walkthrough for in-character confusion reports.
  Use when the user runs /landing-pages.
---

# Landing Pages (the web construction hand)

You **build and instrument landing pages** in the target repo. You are a
**mechanism hand** for `$maximize-market-learning` (the brain): it hands you a
page spec with a stated learning goal; you execute and return the **upward
contract**. You do not choose segments or set strategy — you realize and measure them,
and you push reality back. You also do **not** probe pages with personas: that is
`$persona-walkthrough`'s job. Build and instrument the surface; hand it off to be
walked.

## Substrate: detect, don't impose

Landing pages run in whatever the repo already uses. **Detect first**, adapt second:

- Framework (Astro/Next/plain HTML/Tauri web view/etc.), existing landing surface,
  routing, deploy path (`$plan-deploy-shared`).
- Whether `$implement-growth-experiments` is wired here (PostHog, UTM, flags). If yes,
  **use it** for all instrumentation; if no, install the minimum baseline it defines —
  do not invent a parallel analytics stack.
- Whether `$branding` has produced a visual system. If yes, consume its tokens; if no,
  fall back to a neutral built-in system (mirror `$dataviz`'s palette-swap pattern) so
  taste never blocks the build.

Never hardcode a stack. A page that only works in one framework is a bug for a hand
that runs across repos.

## Two postures (from the spec)

### Discovery — one interest-organized page

- **Sections are organized by interests / jobs / pains, NOT audience labels.** A
  section titled "For Platform Engineers" re-imposes the brain's guesses and destroys
  discovery. Title by the *job/pain* ("Ship without touching a terminal"), let people
  self-select by what they reach for.
- **Active identity, revealed by behavior.** Instrument what each visitor
  expands/clicks/dwells on → an **interest vector**. Add ONE light, skippable declared
  anchor ("what best describes you?") to correlate revealed interest against declared
  identity later.
- **Elicit, don't presume.** The page's job is to let a segment you never named show
  up as a dense cluster. Build for surprise.

### Confirmation — one distinct page per discovered segment

- Each page targets a segment discovered in the discovery run; each is fed by one
  controlled source (an `$ads` set / UTM per page), so intended-segment is baked into
  the URL.
- The on-page chooser becomes a **confirm-or-correct probe**. The most valuable event
  is a **correction** ("actually I'm building solo") — a mis-estimation caught live.
  Route corrections to the sibling page and fire a `segment_corrected` event.

## Instrumentation (via $implement-growth-experiments)

Emit events through the existing substrate; do not build a second one. Minimum:

- `landing_viewed` — with `utm_*`, `$referring_domain`, `intended_segment` (from
  URL/campaign), path.
- `interest_expanded` / `interest_clicked` — `interest_id`, dwell where available
  (builds the interest vector).
- `segment_declared` — `declared_segment` (from the light anchor).
- `segment_corrected` — `from_segment`, `to_segment` (confirmation posture; the gold).
- `cta_clicked` / conversion event — `cta_id`, `intended_segment`, `declared_segment`.

Fire once per logical action; keep identity joinable across the path. Declared vs
intended is what lets the brain judge "did we estimate correctly?" — make it clean.

## Probing is a separate hand — hand off, don't probe

You build and instrument; you do **not** drive personas through the page. When the
brain wants to know how a real human experiences the page you built, it dispatches
`$persona-walkthrough` (the evaluation hand) against the live surface — that hand
loads the persona, walks it in character, and emits the typed confusion report and
learnings. Your job is to make the surface *walkable and measurable*: a working page,
clean events, honest CTAs. Report the page as ready to probe; name the entry URL,
the instrumented events, and the success signal so the walkthrough spec can bind to
it. Do not fork a second persona-run mechanism here.

## The upward contract you must return

Back to the brain, all four fields:

1. **A priori feasibility** — what this page/stack realistically can and can't do
   (e.g. "no server-side events on this static host", "this CTA can't gate without
   auth").
2. **Observed reality** — events fired, interest clusters seen, conversions; the page
   built and its entry URL, ready for `$persona-walkthrough` to probe.
3. **Confusion points** — **N/A for this hand.** Persona confusion is
   `$persona-walkthrough`'s output; here, hand off the built page rather than
   reporting confusion yourself.
4. **Declared unknowables** — what the page structurally **cannot** see: visitors who
   bounced before JS/consent fired, ad-click selection bias in who arrived, dwell on
   non-instrumented regions, cross-device the same person. Name the boundary of
   measurability explicitly — the brain needs it to avoid mistaking measurable for
   true.

## Core workflow

1. **Detect substrate** (framework, analytics, branding, deploy).
2. **Read the spec**: discovery vs confirmation; sections/messages; required events;
   persona(s); success signal; learning goal.
3. **Build the page(s)** — interest-organized (discovery) or per-segment (confirmation);
   consume `$branding` tokens or neutral fallback.
4. **Wire instrumentation** through `$implement-growth-experiments`.
5. **Hand off for probing** — report the page as ready and name entry URL, events, and
   success signal so `$persona-walkthrough` can bind a walkthrough spec to it.
6. **Return the upward contract** (four fields) to the brain.

## Quick self-check

- Discovery page organized by interests/jobs, never audience labels?
- Is declared-vs-intended cleanly measurable end to end?
- Did I detect the stack instead of assuming one?
- Did I hand the built page off to `$persona-walkthrough` instead of probing it myself?
- Did I return feasibility **and** declared unknowables, not just events?
- Did I reuse the existing analytics/branding/Playwright substrates instead of forking
  them?
