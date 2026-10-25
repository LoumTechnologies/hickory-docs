<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# hick Opens The App While Explicit Verbs Keep Their CLI Behavior

For: Authors editing executable documents and the files those documents produce.

Given an installed desktop app, when `hick` runs without arguments, then it
launches a blank editor window and returns. `hick <path>` and `hick open <path>`
open the named folder or document with an absolute path; a missing path is refused.
`hick open` defaults to the current folder. Existing subcommands and aliases,
`--help`, and `--version` keep their CLI behavior. A command name wins over a
bare folder name; `./test` or `hick open test` disambiguates that folder.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/hick-open-hands-a-folder-to-the-app.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
