# Persona — Data / research team needing reproducibility

> **Unvalidated hypothesis.** Identity has not been through the brain's gate and
> this cohort has never been observed. Only the recon-owned sections below are
> populated. See `personas/README.md`.

## Alternatives set — what they use today

*Recon 2026-08-09. This is the most crowded alternatives set of the five
cohorts — see niche N4 in `docs/market/niche-map.md`.*

| Alternative | Kind | Note |
|---|---|---|
| Jupyter | Generic substitute | 40M+ monthly PyPI downloads. The water this cohort swims in. Free |
| **marimo** | **Direct rival — the closest one in this entire recon** | Reactive Python notebook, stored as **pure Python**, git-versionable, runnable as a script, deployable as an app. $5M seed (Nov 2024, AIX Ventures / Anthony Goldbloom; angels incl. Jeff Dean, Clem Delangue). Has taken the "reproducible + git-friendly + file-is-the-truth" position with funding |
| Hex | Direct rival | ~$36–$75/editor/mo + metered compute ($0.32–$6.70/hr) |
| Deepnote | Direct rival | ~$39/editor/mo annual, $49 monthly (as of 2026-08-01); compute bundled |
| Code Ocean | Direct rival, compliance-flavored | "Compute Capsule" = code + data + environment. $16.5M Series B. v4 pitches "agentic workflows that generate reproducible, **compliant** results" |
| Quarto · R Markdown · `bookdown` · Jupyter Book/MyST | Generic substitute | Free, mature, entrenched in academia |
| Nextjournal | Direct rival | **Alive** — checked specifically; site and GitHub active through Dec 2025 |
| Docker + a Makefile + a README | Generic substitute | What reproducibility actually looks like in most teams |
| **Do nothing** | Do-nothing | Re-run it and hope; if it breaks, fix it by hand. Overwhelmingly the incumbent |

**WTP is proven here and nowhere else in this recon:** the category has settled
at **$36–$75 per editor per month**. Note what that implies for the Team tier —
$149/mo for 10 editors is $14.90/editor, roughly a third of the category floor.

## Watering holes

*Recon 2026-08-09. Medium confidence — general knowledge, not sourced norms.*

| Channel | Norms |
|---|---|
| r/datascience, r/Python, r/MachineLearning | Tool posts tolerated when framed as a workflow problem, not a launch |
| Jupyter Discourse · marimo Discord | Practitioner-dense; a rival's Discord is for listening, not posting |
| Locally Optimistic Slack · dbt Community Slack | Analytics-engineering culture; strong anti-vendor norms, `#tools` channels exist |
| PyData / SciPy / posit::conf | Talk-driven. SciPy has an established track for exactly this class of tool (marimo presented there in 2025) |
| Academic/reproducibility venues — VISxAI, rOpenSci, ReScience | Where the Distill diaspora went after the 2021 hiatus. Slow, credibility-driven, non-commercial |
| Twitter/X + Bluesky data community | Fragmented since 2023; still where notebook launches get amplified |

## Mistaken-identity set

*Recon 2026-08-09.*

| They'll think it's… | Why they'd reach for it | Why it falls short |
|---|---|---|
| Another Jupyter | Cells, prose, output, execution | Jupyter's reproducibility is aspirational: hidden kernel state and out-of-order execution are the field's standing joke. Hickory's document is re-derived from source with asserted expectations |
| marimo | Genuinely close — reactive, reproducible, a plain file in git | **Not a mistake; a real competitor.** The remaining distinction is expectation-assertions and byte-level provenance, which is a *feature-level* difference against a funded incumbent — the weakest kind of moat |
| Hex / Deepnote | Hosted collaborative notebooks with compute | Those sell a workspace priced per editor; Hickory sells an artifact that lives in the user's git repo and fails CI when it drifts |
| Quarto / R Markdown | "Prose and code, rendered together" | Re-execution without assertion. A drifted result silently renders a different, wrong document |
| Code Ocean | Reproducible capsules, compliance framing | Capsules package an environment for re-running a *published* result; they don't sit in a team's CI failing builds |
