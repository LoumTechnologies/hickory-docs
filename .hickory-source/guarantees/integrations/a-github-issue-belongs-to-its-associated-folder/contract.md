<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A GitHub Issue Belongs To Its Associated Folder

For: Engineers connecting repository issues, reviews and locally installed tools.

Given a person associates `owner/repository#number` (or its GitHub URL) from a
folder's context menu, when the workspace tree is read here or from another
clone, then the issue is a child of that folder. The committed-capable
`.hick-workspace.json` record contains only provider identity, issue number,
and root-relative folder—never a token or copied provider response.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/integrations/a-github-issue-belongs-to-its-associated-folder.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
