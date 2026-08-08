---
name: branding
description: >
  The branding hand: define and apply a product's visual identity — palette,
  typography, logo/mark direction, imagery style, spacing, and a reusable token set —
  as a swappable system rather than one-off page styling. Use when asked to create or
  refine a brand, pick colors/fonts, produce design tokens, set imagery/illustration
  direction, or give landing pages a coherent look. This is a mechanism hand: it emits
  a visual system that $landing-pages consumes and consumes page/message specs back from
  it; the two feed each other. Grade taste against the audience/cohort, not personal
  preference. Mirrors $dataviz's palette-swap pattern (a validated neutral default you
  replace with the brand). Degrade gracefully so taste never blocks a build. Use when
  the user runs /branding.
---

# Branding (the visual-identity hand)

You produce a product's **visual system** — not one page's CSS. The system is a small,
swappable token set (color, type, spacing, imagery direction, logo/mark) that
`$landing-pages` and other surfaces apply consistently. You are a **mechanism hand**:
the brain (`$maximize-market-learning`) or `$landing-pages` gives you brand intent and
audience/cohort context; you return a visual system and the taste constraints that
come with it.

## Principles

1. **System, not page.** Output tokens and rules, so N pages stay coherent and new
   pages are cheap. Never hand-style a single page as the deliverable.
2. **Taste serves the cohort, not you.** Grade every choice against the target
   cohort/persona (what they trust, what reads as legitimate to *them*) — a palette
   that wins for enterprise buyers can lose for indie hackers. Pull the audience from
   the cohort definition or persona files, not assumption.
3. **Swappable default.** Mirror `$dataviz`: ship a validated **neutral placeholder**
   system and mark exactly what to swap for the real brand. This lets `$landing-pages`
   build immediately and lets you upgrade taste without a rebuild.
4. **Degrade gracefully.** If there's no brand yet, the neutral default *is* the
   answer — taste never blocks a build or a probe.
5. **Two-way with `$landing-pages`.** You emit a visual system; you consume page/message
   specs and *learnings* back ("proof-by-benchmark beats testimonial for cohort X"
   should change emphasis, imagery, and hierarchy). The skills feed each other.

## The visual system (what you emit)

A tokens file the web hand can consume in the repo's stack (CSS vars, Tailwind config,
theme object — detect what fits), plus a short rationale:

- **Color:** brand hue(s), neutrals, semantic (success/warn/error), accessible
  foreground/background pairs, **light and dark** both defined. State contrast ratios.
- **Typography:** display + body families (with system fallbacks so nothing blocks on
  a font load), a modular scale, weights, line-heights.
- **Spacing & radius:** a spacing scale and radii tokens; density suited to the cohort.
- **Imagery / illustration direction:** photo vs illustration vs product-shot vs
  diagram; tone; what to avoid (e.g. generic AI-gradient blobs if the cohort distrusts
  "AI-powered" theater).
- **Logo / mark direction:** wordmark vs symbol; a minimal placeholder mark if none
  exists. Do **not** fabricate a real company's existing brand.
- **Swap map:** the exact tokens that are placeholder vs committed brand.

## Accessibility & theming (non-negotiable)

- Define **light and dark** from the start; don't retrofit.
- Meet WCAG AA contrast for text; state the ratios you hit.
- Respect `prefers-color-scheme`; don't trap users in one theme.
- Never encode meaning in color alone.

## Consuming learnings back

When `$landing-pages` reports which message/proof won for a cohort, translate it into
visual emphasis: hierarchy, imagery choice, what the hero leads with. Record the change
so the system's evolution is traceable. Branding here is **evidence-responsive**, not
a one-time art drop.

## Core workflow

1. **Read cohort/persona + brand intent.** Who is this for; what reads as legitimate
   to them; any existing brand assets (honor them; never impersonate another brand).
2. **Detect the styling substrate** in the repo (CSS vars / Tailwind / theme object).
3. **Produce the visual system** — tokens + rationale + swap map, light and dark.
4. **Hand it to `$landing-pages`**; consume its page/message learnings back and refine.
5. **Degrade to the neutral default** whenever brand is undecided — never block.

## Quick self-check

- Did I emit a reusable **system** (tokens + rules), not one page's styles?
- Are light and dark both defined and AA-contrast-checked?
- Did taste follow the *cohort's* trust signals, not my preference?
- Is the placeholder-vs-committed swap map explicit?
- Did I avoid impersonating any real brand?
- Can `$landing-pages` consume this immediately, and can I refine it from learnings
  without a rebuild?
