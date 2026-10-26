<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Action Is Asked Of The Element, And Carried Out By The Host

For: Authors embedding code in Markdown and maintainers changing the document grammar.

Given a block the app draws, when a client asks for an action on it —
`POST /api/docs/:id/blocks/:at/:action`, `:at` being the byte its tag
starts at — then the element that declared that tag answers, and it
answers with a *request* (`hick_blocks::ActionOutcome`: an answer, a run
of these cells, an edit of this span) which the server carries out with
the machinery its older routes already use. A cell's `run` starts the
same run `POST /run` starts and answers `202` with the same run id; an
`edit` writes the document the way `PUT` writes it and tells the live
room; an `answer` is returned as is. An action the element does not
declare is `404` naming the element and the action; one it refuses is
`422` in its own words; a byte where no element starts is `422`. And
`GET /api/elements` lists every element with its kind, attributes and
actions, read from the same registry `/render` draws from.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/language/an-action-is-asked-of-the-element.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [crates/hick-blocks/src/lib.rs](../../../mechanisms/crates/hick-blocks/src/lib.rs.md#output-5260198b0a7b)
- [crates/hick-literate/src/render.rs](../../../mechanisms/crates/hick-literate/src/render.rs.md#output-4a7f39f68eb8)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)

### Tests and fixtures

- [crates/hickory-cli/tests/block_actions.rs](../../../mechanisms/crates/hickory-cli/tests/block_actions.rs.md#output-fa0f3e293d18)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
