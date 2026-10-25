<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Desktop Build Carries The UI

For: Authors editing executable documents and the files those documents produce.

Given `just local-install` or a Tauri desktop build, when the frontend is built
and the desktop binary is compiled, then the binary embeds that frontend's
index, scripts, and styles. Rebuilding the frontend invalidates Cargo's cached
desktop build even when no Rust source changes. A release build without a built
frontend fails before it can replace the installed app.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-desktop-build-carries-the-ui.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
