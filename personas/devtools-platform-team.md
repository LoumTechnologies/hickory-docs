# Persona — Devtools / platform team whose docs are product surface

> **Unvalidated hypothesis.** Identity has not been through the brain's gate and
> this cohort has never been observed. Only the recon-owned sections below are
> populated. See `personas/README.md`.

## Alternatives set — what they use today

*Recon 2026-08-09.*

| Alternative | Kind | Note |
|---|---|---|
| Mintlify | Direct rival (hosting) | Free as of June 2026 — Starter and Pro both $0 base, unlimited seats on Pro. Verifies nothing |
| ReadMe | Direct rival (hosting) | $0 / $250 / $3,000+. Proves this cohort pays for docs infrastructure. Verifies nothing |
| Docusaurus / mdBook / MkDocs (self-hosted) | Generic substitute | Free, owned, boring. The default for teams that resent a vendor |
| Rust doctests · Python `doctest` · `mdBook test` · Sphinx doctest | Generic substitute | **This is the real competitor for the verification job.** Free, in-toolchain, already in their CI |
| Dredd · Schemathesis | Generic substitute | Diff live API responses against the OpenAPI spec. Covers the API, not the prose |
| A `docs-updated` label gate in GitHub Actions | Do-nothing-plus | Blocks merge if route handlers changed without a spec change. Cheap, crude, widely used |
| **Do nothing** | Do-nothing | Ship it; find out from a user or an abandoned evaluation. Currently the market leader |

**The honest framing:** for *hosting*, they have excellent free options. For
*verification*, they have a free per-language option that mostly works. What
neither gives them is a document that fails the build — which is the only wedge.

## Watering holes

*Recon 2026-08-09. Norms sourced where cited; otherwise general knowledge —
medium confidence, and worth confirming before posting.*

| Channel | Norms |
|---|---|
| Hacker News (Show HN) | One shot, no second chance. Title carries everything; the author must be in the thread answering. A live demo that runs in the reader's browser is the strongest asset — the landing page already has three |
| Lobsters | Invite-only, low tolerance for marketing. Authored-by tag required. Self-promotion is capped by convention |
| **Write the Docs Slack** | Explicit, published norms — **help others ~10× for every self-promotion**; use the most specific channel or `#community-showcase`; **always disclose involvement**; "**calls to action are poison**"; don't re-share without saying why. Source: writethedocs.org/slack. High confidence |
| r/devops, r/ExperiencedDevs, r/programming | Heavily anti-promotional. Value must land before the link; most subs require prior comment history |
| dev.to | Permissive, low signal |
| Platform Engineering / CNCF Slacks | Practitioner-dense; vendor pitches get ignored, war stories don't |

## Mistaken-identity set — what they'll think this is

*Recon 2026-08-09. This cohort has a strong prior that "docs tool" means
"docs hosting," which is the single largest positioning risk for this product.*

| They'll think it's… | Why they'd reach for it | Why it falls short |
|---|---|---|
| Mintlify / ReadMe with extra steps | Same words on the page: docs, projects, publishing | Neither executes an example or fails a build when one drifts; they publish whatever you wrote, correct or not |
| A doctest runner | "We already test our examples" | doctests check examples *in source*, per language. They can't run a document that spins up an environment, feeds it data, and asserts the transcript |
| A Jupyter/notebook thing | Cells, output, execution | The artifact is a versioned document in the repo whose output is asserted in CI, not a session whose state lives in a kernel |
| An API schema-drift checker (Dredd, Schemathesis) | "Drift detection — we have that" | Those compare a live API to a spec. They say nothing about whether the quickstart a human follows still works |

**Positioning consequence (inference):** leading with "documentation" puts this
product into a category that was just repriced to $0 and where it looks like a
weaker copy. Leading with **"your quickstart fails CI the day it breaks"**
describes a job none of the four above do.
