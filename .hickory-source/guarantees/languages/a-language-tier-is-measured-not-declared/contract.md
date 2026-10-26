<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Language Tier Is Measured, Not Declared

For: Teams choosing a toolchain and writing generators in their existing language.

Given a language Hickory can route, when `hick lang` reports its tier, then
every column is computed from the catalogue or discovery path that would
actually serve the request — never from a list written down beside it. A
language is reported as having a language server exactly when discovery knows
a candidate for it or a catalogue can install one; as debuggable exactly when
the same is true of an adapter; and as Gold only when a code model server
exists for it, which is true of no language today.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/languages/a-language-tier-is-measured-not-declared.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hickory-cli/src/language_tier.rs](../../../mechanisms/crates/hickory-cli/src/language_tier.rs.md#output-3bcda72be8eb)

### Tests and fixtures

- [apps/web/src/editor/languages.test.ts](../../../mechanisms/apps/web/src/editor/languages.test.ts.md#output-773fde4e8dc2)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
