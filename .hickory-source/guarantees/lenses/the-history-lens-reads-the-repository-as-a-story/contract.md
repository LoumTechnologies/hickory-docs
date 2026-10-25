<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The History Lens Reads The Repository As A Story

For: Authors reviewing history and choosing where an edit should land.

Given a folder under version control, when its history is opened as a story
(the Git pane's *Read as a story*, or *History as a story* from the command
bar), then the commits are drawn **oldest first** as cards: an ordinary
commit as prose (its subject, its body, its files behind one click), a
commit whose trailers carry a recipe (`Hick-Recipe`, `Hick-Image`,
`Hick-Output`) as a **cell** whose command is shown and whose output is the
commit's own diff, and the working tree as the last card. The past folds by
default and the view opens at the tail. The publication floor is drawn as a
line between the records and the drafts. A recipe card, once expanded, says
which of its files a later commit changed and in which commit — or that
none has.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/lenses/the-history-lens-reads-the-repository-as-a-story.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
