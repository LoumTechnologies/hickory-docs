<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Empty Response Never Kills a Session

For: Authors using an agent to change documents while retaining review control.

Given a model that returns an empty or whitespace-only response, when the
agent loop re-prompts it, then the correction turn succeeds: the blank turn is
recorded as `(no output)` rather than replayed verbatim, and no request built
by either client ever carries an empty content block.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/an-empty-response-never-kills-a-session.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hickory-agent/src/llm_anthropic.rs](../../../mechanisms/crates/hickory-agent/src/llm_anthropic.rs.md#output-ad06d96de78a)
- [crates/hickory-agent/src/llm_openai.rs](../../../mechanisms/crates/hickory-agent/src/llm_openai.rs.md#output-0150d11c8f60)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve; no-existing-code-or-test-reference-resolved.

</storycode:doc>
