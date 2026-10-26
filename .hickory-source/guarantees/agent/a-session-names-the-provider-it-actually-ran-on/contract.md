<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Session Names the Provider It Actually Ran On

For: Authors using an agent to change documents while retaining review control.

Given an agent run configured with a provider selector, when that selector is
not one this build supports, or the selected provider has no API key, then the
run refuses before the first request — naming the accepted selectors, or the
exact environment variable that is unset. It never falls back to a different
provider.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/a-session-names-the-provider-it-actually-ran-on.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hickory-agent/src/provider.rs](../../../mechanisms/crates/hickory-agent/src/provider.rs.md#output-2be2d28e151b)
- [crates/hickory-agent/src/usage.rs](../../../mechanisms/crates/hickory-agent/src/usage.rs.md#output-f4b085d673b2)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
