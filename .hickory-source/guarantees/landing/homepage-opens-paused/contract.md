<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The public debugger demo opens paused

For: Prospective users checking what the homepage demonstrates and promises.

Given a developer visiting the public demo page at `/site.html`, when its static assets load, then
one short Markdown document opens with a real browser debug session paused
at its return statement. The breakpoint, selected frame, highlighted line
and variables come from executing that document's code.

The [read-only Simple Browser document](../read-only-homepage/contract.md) now opens at the site root. The paused debugger and engineer tour are available from its demo link.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/landing/homepage-opens-paused.md.md). The paragraph ledger records their exact ranges and dispositions.

The interactive behavior stays on the public demo page. The original homepage placement is recorded below.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/src/views/LandingView.test.tsx](../../../mechanisms/apps/web/src/views/LandingView.tsx.md#output-ca24d1670e8c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
