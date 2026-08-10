# Persona — OSS maintainer / individual

> **Unvalidated hypothesis.** Identity has not been through the brain's gate and
> this cohort has never been observed. Only the recon-owned sections below are
> populated. See `personas/README.md`.

## Alternatives set — what they use today

*Recon 2026-08-09.*

| Alternative | Kind | Note |
|---|---|---|
| GitHub README + Pages | Generic substitute | Free, zero-config, already there. The true default |
| mdBook / MkDocs / Docusaurus | Generic substitute | Self-hosted, owned, no account |
| Language-native doctests (`cargo test --doc`, `doctest`, `mdBook test`) | Generic substitute | **Already solves the verification job well enough** for single-language projects, in CI they already run |
| Quarto · Jupyter Book / MyST · R Markdown | Direct rival (documents that run) | Free, mature, large communities |
| Runme (Stateful, OSS) | Direct rival | Executable markdown runbooks — "Jupyter with a shell kernel." No public commercial pricing found |
| Mintlify free tier | Direct rival (hosting) | As of June 2026 there is effectively no reason for this cohort to pay anyone for hosting |
| **Do nothing** | Do-nothing | Docs rot; a user opens an issue; it gets fixed. Free |

**Constraint that dominates this cohort:** willingness to pay is **$0 by
design** (`pricing-strategy.md`), and tolerance for a hosted dependency is low.
A load-bearing feature behind a hosted account converts this cohort from
distribution into detractors.

## Watering holes

*Recon 2026-08-09. Medium confidence except where cited.*

| Channel | Norms |
|---|---|
| Hacker News (Show HN) | The single highest-leverage channel for this cohort. Self-hostable + open core must be visible above the fold or the thread turns hostile |
| Lobsters | Invite-only, authored-by tag required, strong anti-marketing norm |
| Language communities — r/rust + This Week in Rust, r/Python, users.rust-lang.org, Elixir Forum | Contribution-first. TWiR-style newsletters accept a PR for a genuinely relevant project |
| GitHub Discussions / Issues on adjacent projects | Legitimate only as *participation*; drive-by "we built a thing" links are poison |
| Fosstodon / Mastodon | Where the license-conscious end of this cohort actually is |
| SustainOSS · maintainer forums | Sustainability-framed, not tool-framed. Only relevant if the pitch reduces maintainer labor |
| **Write the Docs Slack** | Published norms: help ~10× per self-promotion, `#community-showcase`, disclose involvement, "calls to action are poison." Source: writethedocs.org/slack. High confidence |

## Mistaken-identity set

*Recon 2026-08-09.*

| They'll think it's… | Why they'd reach for it | Why it falls short |
|---|---|---|
| Quarto / Jupyter Book / mdBook | "A tool that renders prose plus code, re-executing on build" | Those render and re-run; they don't assert expected output, and they carry no provenance from an output byte back to its origin. Drift produces a *different page*, not a *failed build* |
| Rust/Python doctests | "My examples are already tested" | Correct for one language's in-source snippets. Not for a document that provisions an environment, injects data, and asserts a transcript |
| Literate programming (WEB, noweb, org-babel) | It *is* that idea, and this cohort is old enough to know it | Fair comparison — and the reason `docs/market/failure-ledger.md` exists. The differentiator must be that the derived artifacts round-trip, which is precisely what killed the predecessors. **Expect this objection from the best-informed commenter in any thread; it deserves a real answer, not a dodge** |
| Runme | Executable markdown, overlapping surface | Runme targets ops runbooks and shell execution; it does not verify a published document against expectations in CI |
| "Another VC-backed devtool that will rug-pull the OSS core" | Pattern-matched reflex, earned by the category | Only answerable by the self-hostable core being real and load-bearing, which it is (`LocalExecutor`, single node, free forever) |
