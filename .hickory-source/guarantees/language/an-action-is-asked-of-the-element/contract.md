<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
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
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/language/an-action-is-asked-of-the-element.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
