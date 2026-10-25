<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Generator Is Written In The Team's Language, Not The Model Server's

For: Teams choosing a toolchain and writing generators in their existing language.

Given a language with a code model server, when somebody writes a generator
against it, then the generator may be written in **any** language: the server
speaks GraphQL over a pipe and has no opinion about what is on the other end
of it. `hick code-model --client --target <lang>` emits the types, and
`hick code-model --runtime --target <lang>` emits the transport that carries
them — so a project asking in two languages gets two clients and vendors
neither.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/languages/a-generator-is-written-in-the-team-s-language.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
