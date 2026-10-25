<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A desktop app bundles its own Python agent

For: Authors using an agent to change documents while retaining review control.

Given a macOS desktop build produced by `just local-install`, `just
build-desktop-app`, or `scripts/dist-desktop.sh`, when the app is installed
and opened, then the built-in Python agent is available without naming a
path: the app carries a self-contained Python runtime at
`Contents/Resources/python-agent`, discovered from the running executable's own
location. **Settings → Agents** says it is using that runtime, the Agent pane
offers **Hickory Python** as available, and no value is written
into `python.json`.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/a-desktop-app-bundles-its-python-agent.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
