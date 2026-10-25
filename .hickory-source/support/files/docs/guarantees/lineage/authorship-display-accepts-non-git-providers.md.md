<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/lineage/authorship-display-accepts-non-git-providers.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `6a3c7ec67c0b2def8138cfe6f8ef3ccba770c9eaa6788742c761c5c3adf92ae4`.

<book:entry path="docs/guarantees/lineage/authorship-display-accepts-non-git-providers.md">
# Authorship display accepts non-Git providers

Given an attribution provider supplying line metadata, when author colors are
turned on in Lineage settings, then the editor distinguishes credited authors
without replacing syntax colors. Git blame is the default provider. A host can
install a replacement provider without Git. Recorded agent writes supplement
Git attribution using the existing strong session-write fingerprints. Session
user and assistant elements expose their recorded roles. Absent origin evidence
remains unknown; a Git identity alone never classifies text as human or agent.

An edit clears color attribution for that buffer until refreshed. A late request
for earlier editor bytes cannot reinstall stale attribution. Provider records
include the attributed line text, and records that differ from the editor are
omitted, including when colors are enabled over an unsaved edit. Color is accompanied
by a provider/author/origin tooltip; agent origin uses a dashed stripe and
imported origin uses a dotted stripe.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified
- Originating requirement: Git-backed authorship coloring with a non-Git seam.
- Evidence: `lib/attribution.ts`, `editor/blameGutter.ts`, `editor/useBlame.ts`,
  `components/LineageSettings.tsx`, `serve/api.rs::blame`.
- Test coverage: `lib/attribution.test.ts`, existing `blameGutter.test.ts`.
- Assumptions and exclusions: line-granular; mixed-origin lines cannot provide
  span-level identity. Recorded roles are declarations, not authenticated people.
  Missing sessions cannot establish agent origin. Uncommitted is a Git state.
- Witnesses and challenges: non-Git provider installation/restoration; edit
  invalidation; stable author colors. Agent origin remains separate from credit.
- Executed checks: attribution and existing blame-gutter unit tests passed;
  non-Git human/agent display passed in Chromium, Firefox and WebKit.
  `serve_blame` HTTP tests verify recorded conversation roles without Git.
- Human judgment: not recorded; visual accessibility review remains necessary.

- Dependency fingerprint: `apps/web/src/editor/useBlame.ts` SHA-256 `1c9d803dc560607ac94cc0f4ec5dace515a555d59d4cf9fab542ce3caa22bbaa`; changes to this implementation or its cited integration paths require review.


## 2026-10-22 shared display reading

LLM review: partial. Displayed attribution has explicit bounded line and byte
readers through the installed editor-language facet. Both layers clear on source
change; provider source text is excluded. Twenty-three focused attribution/gutter
cases and changed-entrypoint TypeScript passed. Exact dependencies, evidence and
limitations are in the issue-72 handoff. No human judgment supplied.
</book:entry>

</book:doc>
