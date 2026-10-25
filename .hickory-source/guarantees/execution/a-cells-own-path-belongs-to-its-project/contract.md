<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Cell's Own Path Belongs To Its Project, Not To Your Shell

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a document run through a local executor (`LocalExecutor`, and the
`SandboxedExecutor` wrapping it), when its cells execute, then the scratch
directory holding their workdirs is named after **the project the document
lives in** — so two runs of one document agree about that path no matter which
directory the person was standing in, and two different projects never share
one.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />
- [Requirements 3](requirements/03.md)
<book:include file="requirements/03.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-cells-own-path-belongs-to-its-project.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
