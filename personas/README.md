# Persona library

Personas are durable, **product-agnostic** cohort assets owned by the
`$maximize-market-learning` brain. Identity is defined once and the product is
bound at run time; hands *run* personas and write confusion reports and
learnings back.

## Current state (2026-08-09) — read this before trusting anything here

**No persona in this directory has been through the brain's identity gate, and
none has been observed.** These files were created by `$market-recon` during
the first recon run, and they contain **only the three sections recon owns**:

- **Alternatives set** — what the cohort uses today instead of us (rivals,
  generic substitutes, spreadsheets, do-nothing).
- **Watering holes** — where the cohort congregates, and each channel's
  self-promotion norms. `$community-engagement` reads this.
- **Mistaken-identity set** — adjacent things the cohort will reflexively
  compare this product to *even though they don't use them for this job*, with
  the one-line reason each falls short. `$audience-first-docs` reads this for
  positioning; `$community-engagement` reads it to anticipate "isn't this just
  X?" objections.

Everything else a persona needs — identity, goals, constraints, the learnings
log — is **deliberately absent**. Persona definition is a collaborative gate
between the brain and the operator, and recon does not get to invent a human.

The six cohorts here are **hypotheses from `docs/specs/freeform/pricing-strategy.md`
plus two added by the operator**, not discovered clusters. The discovery landing
page has never been driven by traffic, so no cluster has ever been observed.
Treat every file here as a research note until a probe promotes it.

| File | Cohort | Origin |
|---|---|---|
| `devtools-platform-team.md` | Devtools/platform teams whose docs are product surface | pricing-strategy.md buyer map |
| `oss-maintainer.md` | OSS maintainers and individuals | pricing-strategy.md buyer map |
| `data-research-team.md` | Data and research teams needing reproducibility | pricing-strategy.md buyer map |
| `agent-output-owner.md` | People whose agent sessions become artifacts in git | product thesis |
| `compliance-provenance-enterprise.md` | Enterprises needing auditable lineage for compliance | operator, 2026-08-09 |
| **`delivery-owner.md`** | **Tech lead / eng manager / PM who owns "did we build what we said"** | **operator, 2026-08-09 (addendum)** |

**`delivery-owner.md` carries an explicit split hypothesis** (internal product
org vs contract-bound delivery) with a recorded trigger for forking it. Read
that section before treating it as one cohort.
