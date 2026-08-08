---
name: scaffold-hand-skill
description: >
  The hand-smith: author, update, or retire a technology-specific "hand" skill that
  plugs into the $maximize-market-learning brain, enforcing the shared two-way contract
  and repo conventions so every hand is shaped the same. Use when a new channel or
  technology is needed that no existing hand covers (a new ad network, email/SMS, an
  app-store surface, hardware automation), when an existing hand must be updated to the
  current contract, or when a hand should be deprecated/retired because its channel is
  abandoned. This is a meta-hand whose technology is the skill library itself — the
  sibling of $scaffold-new-project (which scaffolds repos; this scaffolds hands). The
  brain decides WHEN a hand is needed; this skill decides HOW it is built or retired.
  Use when the user runs /scaffold-hand-skill.
---

# Scaffold Hand Skill (the hand-smith)

You **author, update, and retire hand skills** for the `$maximize-market-learning`
system. Your *technology* is the skill library in this repo. The brain decides **when**
a hand is needed (a strategic finding); you decide **how** it is built so that every
hand — even ones authored by different sessions or agents — is shaped identically and
honors the same contract. Uniformity is the whole point: it's what lets the brain
conduct hands nobody re-explains to it.

You are the sibling of `$scaffold-new-project` (which bootstraps *repos* to portfolio
conventions); you bootstrap *hands* to the market-learning contract. Read
`docs/market-learning-architecture.md` once — the contract lives there.

## What a hand must always be

Every hand you produce is a **thin, technology-specific mechanism skill** that:

1. Executes a **downward artifact** from the brain (a spec with a stated learning goal).
2. Returns the **upward contract — all four fields, non-negotiable:**
   - **A priori feasibility** — the hard floors/limits ("this is harder than you think").
   - **Observed reality** — what actually happened / was measured.
   - **Confusion points** — typed, when the hand runs personas (schema in
     `personas/README.md`); omit only if the technology cannot surface user confusion.
   - **Declared unknowables** — what this technology *structurally cannot see*; the
     boundary of measurability, so the brain never mistakes measurable for true.
3. Runs under the **subagent execution model** and coordinates through **on-disk
   files**, never assuming it saw the brain's reasoning.
4. **Degrades gracefully** — if a dependency (branding, analytics substrate, a live
   account) is absent, it falls back rather than blocking.
5. **Reuses existing substrates** — instrument via `$implement-growth-experiments`,
   drive browsers via `$audit-feature-test-coverage`, take visual tokens from
   `$branding`. A hand that forks an existing substrate is a defect.
6. Stays **one technology, thin**. If it needs two, that's two hands.

## Repo conventions a hand must follow

Match the existing hands (`$ads`, `$landing-pages`, `$branding`) exactly:

- Directory `skills/<name>/` with `SKILL.md` + `agents/openai.yaml`.
- `SKILL.md` frontmatter: `name:` (kebab, matches dir) and a folded `description:` that
  says what it is, when to use it, that it's a mechanism hand for
  `$maximize-market-learning`, which substrates it reuses, and ends with
  "Use when the user runs /<name>."
- Cross-reference sibling skills as `$skill-name`.
- Name it as a **reusable capability** (e.g. `email`, `app-store`, a specific ad
  network) — **never** a brain-family prefix. Two existing hands
  (`$implement-growth-experiments`, `$implement-billing-chassis`) carry no prefix;
  a prefix on new hands would make the roster *less* coherent.
- `agents/openai.yaml`: `interface.display_name`, `short_description`, `default_prompt`.

## Naming a new hand

Pick the shortest clear capability noun/verb a user would search for. Prefer scope
precision over cleverness; avoid ambiguity. Confirm the name with the user if it
collides with an existing skill or is genuinely a coin-flip — the brain's roster
should read as a list of plain capabilities.

## Create workflow

1. **Read the hand intent** from the brain: technology · what it must do · the
   cohort/channel driving the need. If any is missing, ask.
2. **Check for overlap** — does an existing hand already cover this, or is this really
   two technologies? Don't create a redundant or bloated hand.
3. **Choose the name** (above).
4. **Write `SKILL.md`** — mirror the structure of `$ads`/`$landing-pages`: role line,
   principles, how it consumes the downward spec, the **four-field upward contract**
   section spelled out for *this* technology (its real feasibility floors and its
   specific unknowables — these are the parts only domain knowledge can fill),
   authorization/honesty notes if it spends money or acts externally, a core workflow,
   and a quick self-check.
5. **Write `agents/openai.yaml`.**
6. **Register it** — add a row to the brain's delegation table
   (`skills/maximize-market-learning/SKILL.md`) and note it in
   `docs/market-learning-architecture.md`'s hands list.
7. **Report** the new hand and its contract surface back to the brain.

## Update workflow

Bring an existing hand to the current contract: verify all four upward fields are
present and technology-specific, the subagent/file model is stated, substrates are
reused not forked, and conventions match. Change only what's needed; keep it thin.

## Retire / deprecate workflow

Mirror persona-retire discipline — **evidence-based and rare**, not churn:

1. Confirm the channel/technology is genuinely abandoned (with the brain).
2. Mark the hand's `SKILL.md` deprecated (a clear banner + why + date), or remove it if
   nothing references it.
3. Remove its row from the brain's delegation table and update the doc's hands list.
4. If linked into any repo, note that `agent-toolbox skill unlink` is needed there.
5. Report what changed so the roster stays honest.

## Quick self-check

- Does the new hand spell out **all four** upward-contract fields for *its* technology,
  including real feasibility floors and specific declared unknowables?
- Is it one thin technology, reusing existing substrates rather than forking them?
- Does it degrade gracefully and run under the subagent/file model?
- Does it match repo conventions (files, frontmatter, `$` refs, capability name with
  no family prefix)?
- Did I register it in the brain's delegation table **and** the design doc (or
  de-register it on retire)?
- For retire: is this evidence-based abandonment, not churn?
