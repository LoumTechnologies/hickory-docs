<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Missing Merge Driver Is A Button, Not A Command To Go And Type

For: Engineers using local sessions or pairing their own machines.

Given a clone whose `.gitattributes` routes `*.md` at the hick merge
driver but whose git config does not define it, when the app opens that
folder and shows the banner saying so, then the banner carries a **Run
hick init** button that runs `hick init` on the folder inside the engine
— the same `run_init` the CLI runs: hook, `.gitignore`, `.gitattributes`,
the driver definition, the editor and agent files — and then says what it
wrote and that `.md` documents now merge through hick in this clone. If
it cannot (the folder is not a git work tree, git is missing), the warning
stays and the reason is beside it, in the engine's words.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/a-missing-merge-driver-is-a-button.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
