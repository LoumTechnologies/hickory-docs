<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A New Project Is A Recipe Commit, Made As One Act

For: Authors editing executable documents and the files those documents produce.

Given a `dotnet new` template chosen in File → New Project, a **location**
— any folder on this machine — and a folder name for its files, when the
project is created, then the scaffolder runs in a terminal, in a scratch
directory, and what it wrote is committed the moment it exits zero — in one
act, with nothing a person can do in between — as a commit whose last
paragraph carries the recipe:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)
- [Requirements 3](requirements/03.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-new-project-is-a-recipe-commit.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />
- [Requirement detail](requirements/03.md)
<storycode:include file="requirements/03.md" />

## Implementation

</storycode:doc>
