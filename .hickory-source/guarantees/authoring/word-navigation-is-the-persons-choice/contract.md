<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Word Navigation Is The Person's Choice

For: Authors editing executable documents and the files those documents produce.

Given any editor in the app — a `.md` document, a plain file, a woven
output — when Ctrl+← or Ctrl+→ is pressed (Option+← / Option+→ on macOS), then
the caret moves by whichever definition of "word" the person picked in
Settings → Editing, and Shift extends the selection by the same unit:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/word-navigation-is-the-persons-choice.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
