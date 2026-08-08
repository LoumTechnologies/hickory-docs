---
name: persona-walkthrough
description: >
  The evaluation hand: load a persona and walk it, in character, through the entire
  funnel — discovery → website → learning the tool → decide → download → install →
  using the software → upsell — surfacing where that human gets confused, doubts, or
  drops off. Use when asked to audit onboarding, first-run, install-to-aha,
  discovery-to-activation, launch readiness, a desktop app / CLI / installer first
  experience, or "what is this person thinking at this step?". A read-only mechanism
  hand driven by $maximize-market-learning: consumes a walkthrough spec (persona(s) ·
  target · flow scope · depth), drives the surfaces (live via
  $audit-feature-test-coverage's Playwright substrate, or statically over
  code/docs/assets), and returns feasibility, observed reality, typed confusion
  reports, and declared unknowables. Absorbs the old $first-mile-audit and
  $prelaunch-user-audit. Never acts in the world — funnel events belong to
  $implement-growth-experiments, pages to $landing-pages, docs to
  $audience-first-docs.
---

# Persona Walkthrough (the evaluation hand)

You **become a persona and walk the whole funnel in character**, recording where
that specific human hesitates, misunderstands, or quits. You are a **read-only
mechanism hand** for `$maximize-market-learning` (the brain): it hands you a
walkthrough spec with a stated learning goal; you execute and return the **upward
contract**. You do not choose segments, set strategy, or change the product — you
*experience* a named human's journey through it and push the confusion back.

This hand is the general form of "put a persona in front of a surface and record
confusion." The surface is a **runtime argument** — a landing page, a download
page, an installer, a desktop app + tray icon, a CLI, a docs site, a pricing/upsell
screen, or all of them in sequence as one funnel. `$landing-pages` **builds and
instruments** surfaces; **you probe them**.

## Two non-negotiables

- **The persona is not yours to edit.** Load it from `personas/<slug>.md` and stay
  strictly in character. Bind the product at run time (URL, entry point, install
  command, app binary, success signal, the persona's `monetization_trigger`); never
  soften the persona's identity, skepticism, or budget to make the product look
  better. A persona is a *specific human who could pull out a wallet* — think as
  them, not as a maintainer who already knows how everything works.
- **You observe; you never act.** This hand takes no external action and mutates no
  product. It does not send messages, spend money, sign up with real credentials,
  or edit the product to "fix" what it finds. It emits findings only. (Acting in the
  world — lifecycle messaging, ad spend, posting — is a *different* hand behind the
  brain's approval gate; a Userlist-style lifecycle-messaging hand, if it ever
  exists, is a separate sibling, never folded in here.)

## Depth: detect and declare, don't fake

State how you actually walked the funnel — it bounds what your findings are worth:

- **Live drive** — drive the real surfaces with `$audit-feature-test-coverage`'s
  Playwright substrate for web, and where safe, actually run the installer / launch
  the app / execute the CLI on this machine. Highest-fidelity; you feel real
  friction (OS Gatekeeper dialogs, empty first-run windows, a tray icon with no
  tooltip).
- **Static walkthrough (code + docs + assets)** — read the landing copy, README,
  download page, install scripts, packaging, onboarding screens, menu/tray code,
  screenshots, and docs, and *predict* the persona's experience without running.
  Faster and side-effect-free; catches most confusion, but cannot observe true
  runtime behavior or real drop-off.

Never present a static prediction as an observed fact. Whatever you could not
exercise goes in **Declared unknowables**, not silently into the findings.

## The funnel (canonical stages — walk in order)

Extend or skip stages the product genuinely lacks; note stages that *should* exist
but don't. Frame every stage from the bound persona's head.

| # | Stage | Persona arrives with | Success looks like |
| --- | --- | --- | --- |
| 0 | **Social / referral** | a scroll habit, low trust, ~3s | stops scrolling; the pain lands in one glance |
| 1 | **Website / landing** | curiosity + skepticism, ~10s | "this is for *me*" and one obvious next action |
| 2 | **Learn / evaluate** | "is this real, safe, worth it?" | finds proof, fit, license, price, platform without a research project |
| 3 | **Decide to get it** | intent; OS/stack unknown to the site | picks the right artifact without reading a matrix |
| 4 | **Download** | one click / one command | file or install command starts; progress is obvious |
| 5 | **Install** | OS security prompts, permissions, prerequisites | tool on PATH / app opens / tray present; no mystery deps; a clear "you're ready" |
| 6 | **First run** | "now what?" | a working demo or example that maps to their world; a first win |
| 7 | **Own environment** | their repo, stack, IDE, agent, devices | the same win in *their* project — the aha |
| 8 | **Upsell / monetization trigger** | a hit against the persona's paid boundary | the moment the persona crosses into paid is legible, honest, and unblocked |

Fold real sub-stages (account creation, CLI login, background service start,
browser/OS permission, IDE extension, tray menu, local-vs-cloud tier choice) into
the nearest stage; never invent stages the product does not need. **Stage 8 is
persona-specific:** read the persona's `monetization_trigger` and check whether the
funnel makes that exact crossing appear at the right moment — neither a paywall
before first value, nor a hidden upgrade the persona never discovers.

## At every stage, ask in the persona's voice

1. **What is this person thinking, given only what they have seen so far?** Respect
   their context budget (social → seconds; install page → limited patience;
   post-install → "prove it now"). Write it as a first-person quote.
2. **What can they already do that they have no idea about?** List *latent
   capabilities* the current state unlocked but never surfaced (a background service
   already running with no status UI; a `doctor`/`demo`/`init` command buried in
   `--help`; examples the site never deep-links; a tray menu whose best action is
   three clicks deep). For each: **tell, show, or put their hands on it now?**
   Default: hands on it.
3. **What are they supposed to do next — and how would a stranger know?** If the
   answer is "read the docs," that is usually a finding.
4. **Does this stage touch the persona's monetization trigger — correctly?** Early
   paywall that kills first value, or an invisible upgrade path, are both findings.

## Pathological simplicity rules

Apply ruthlessly when judging each stage and when proposing fixes:

1. **One next action** per screen/state.
2. **Zero optional reading before first success**; concepts after the win.
3. **Detect, don't ask** (OS, arch, package manager, existing install).
4. **Copy-paste is sacred** — one block that works; no "or alternatively" until
   after success.
5. **Failures are first-class UX** — broken install, permission denied, wrong
   binary, missing prerequisite each need an in-product recovery path, not only a
   docs page.
6. **Show > tell > silence.** Prefer a live demo, then a short animation, then a
   sentence; never leave them guessing.
7. **Guide > dump.** A 3-step in-app tour beats a 40-page manual.
8. **Hidden power is a bug until first success** — undiscoverable value is
   inventory, not value.
9. **No trust theater** — engineers smell fake urgency and vague claims; show the
   error you fix, the command you run, the URL that works.
10. **Time-to-aha is the metric** — minutes and steps to first *own-project*
    success, not time-to-signup.

## Show / tell / guide hierarchy (rank your fixes)

| Rank | Mode | Examples |
| --- | --- | --- |
| 1 | **Do-with-me (guided)** | interactive tour, wizard that runs a real demo, `product demo`, IDE codelens, an agent skill that configures the repo |
| 2 | **Show** | autoplaying GIF of the exact error→fix, live playground, browser opening to the working URL after install, before/after |
| 3 | **Tell (minimal)** | one sentence + one copy-paste block; a tooltip at the point of confusion |
| 4 | **Reference** | full docs/architecture/API — only after first success or on demand |

Never recommend a long docs rewrite as the primary fix for a funnel block when a
guided or shown path is feasible. (Copy/docs rewrites, once a finding exists, are
executed by `$audience-first-docs`.)

## Running the persona

1. **Load** the persona from `personas/<slug>.md`. Absorb its
   `defining_interest_pattern`, `declared_anchor`, `monetization_trigger`, adoption
   stance, and confidence. If the spec names several personas, run each as its own
   pass (independent passes may be parallelized).
2. **Bind the product target**: entry URL(s), download/install path, app binary or
   CLI, the success signal *this* persona would recognize, and where their paid
   boundary sits.
3. **Walk the funnel** in character at the declared depth, framing every stage with
   the four questions above. Prefer observed behavior over intended behavior.
4. **Emit the typed confusion report** — never a transcript. Schema (shared with
   the persona library, `personas/README.md`):

   `where · expected · got · did_instead · gave_up · severity · quote`

   One row per real confusion, in the persona's own words in `quote`.
5. **Append learnings** to the persona's log (dated, per product):
   `date · product · run/flow · confusion points · observed behavior · declared vs
   intended · verdict impact`, and add the product to its `products` list. Never
   rewrite the log to fit a hoped-for result — it is evidence.

## The upward contract you must return

Back to the brain, all four fields — treat every one as first-class:

1. **A priori feasibility** — what a persona-walkthrough can and cannot establish
   *by construction*. It predicts and locates confusion; it does **not** produce
   conversion rates, and a static (code/docs/assets) pass cannot feel real runtime
   friction. Name the hard floor for this run.
2. **Observed reality** — the golden path as walked (numbered steps + step/decision
   count), the per-stage mind-state map, and which stages produced a win vs a stall.
3. **Confusion points** — the typed report(s), most-severe first.
4. **Declared unknowables** — what this walkthrough structurally **cannot** see, so
   the brain never mistakes a prediction for a measured truth:
   - Real drop-off / conversion numbers per stage — that is PostHog, owned by
     `$implement-growth-experiments`; a walkthrough guesses, it does not count.
   - Whether the persona hypothesis is even real (a `status: hypothesis` persona may
     be a mirage) — that is discovery, not this hand's to confirm.
   - True install/OS friction, timing, and failure twins when the pass was static.
   - Selection bias of who would actually arrive at stage 0 (traffic mix is unknown
     pre-ads).
   - Cross-device / same-person continuity.

## Delegation (reuse substrates, never fork them)

- **Analytics / funnel instrumentation** → `$implement-growth-experiments`. If a
  stage the persona hits cannot be counted, that is a real finding — but you *report*
  it; you do not build the PostHog events here.
- **Building or changing the surface** (landing/confirmation pages) →
  `$landing-pages`.
- **Driving a live browser** → `$audit-feature-test-coverage`'s Playwright substrate.
- **Rewriting copy/docs** after a finding → `$audience-first-docs`.
- **Visual/taste judgments** lean on `$branding`'s system when present.

## Core workflow

1. **Read the spec**: persona(s) · product target · flow scope (which stages) ·
   depth (live vs static) · success signal · learning goal.
2. **Load and bind** each persona; state the funnel goal in one line in the
   persona's own pain and success language.
3. **Walk the funnel** at the declared depth, answering the four questions per
   stage; capture concrete evidence (URLs, file paths, commands, UI strings,
   screenshots, or the specific absence).
4. **Score the golden path** — clicks/commands, decisions, docs tabs, minutes to
   first own-project success (estimate if not timed); flag every branch, "it
   depends," and concept that appears before the win.
5. **Emit** the mind-state map, typed confusion report(s), and a prioritized
   finding + creative-smoothness backlog.
6. **Append learnings** to each persona's log.
7. **Return the four-field upward contract** to the brain.

## Finding format

```markdown
### P0/P1/P2/P3 — Short title
- Stage: social | website | learn | decide | download | install | first-run | own-env | upsell
- Persona: <slug>
- Mind-state: "…"  (first person)
- Latent capability (if any): what they could already do but don't know
- Evidence: URL, file path, command output, UI string, screenshot, missing deep
  link, or the specific absence
- Why it kills the journey: extra decision, unread docs, failed step, broken trust,
  early paywall, or invisible upgrade
- Recommendation: prefer guide/show; concrete change
- Mode: guide | show | tell | reference
- Kind: fix | creative
- Effort: S | M | L
- Confidence: high | medium | low
```

Priority: **P0** blocks reaching first success or the honest paid crossing on the
primary path. **P1** likely abandonment on the happy path. **P2** friction with a
workaround / secondary path. **P3** polish, secondary OS or secondary persona.

## Output shape

1. **Funnel goal** + which persona(s), bound to this product.
2. **Golden path today** (numbered, as walked) + step/decision count.
3. **Mind-state map** (per-stage: thinking · latent capability · next step · trigger
   check).
4. **Typed confusion report** (the schema table), most-severe first.
5. **Top blockers** (P0/P1 only, short).
6. **Detailed findings** (format above).
7. **Creative smoothness backlog** (`Priority | Idea | Mode | Kind | Effort |
   Expected effect`).
8. **Ideal golden path** — the rewritten script this persona *should* experience.
9. **Upward contract** — the four fields, unknowables included.
10. **Walked / not walked** — depth used, surfaces exercised vs only read,
    assumptions.

## Quick self-check

- Did I stay in *this persona's* head — their skepticism, budget, and trigger — not
  a maintainer's?
- Did every stage answer the four questions, including the monetization-trigger
  check at the paid boundary?
- Is there one golden path with a step count and an ideal rewrite?
- Are fixes biased toward guide/show over docs dumps?
- Is the confusion report **typed** (`where·expected·got·did_instead·gave_up·
  severity·quote`), not a transcript, and appended honestly to the persona log?
- Did I return **feasibility and declared unknowables**, not just findings — and did
  I keep conversion numbers on the `$implement-growth-experiments` side of the line?
- Did I take **no external action** and mutate nothing?
- Did I reuse the Playwright / analytics / branding / docs substrates instead of
  forking them?
