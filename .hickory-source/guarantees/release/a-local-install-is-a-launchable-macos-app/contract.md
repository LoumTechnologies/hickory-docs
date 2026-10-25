<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Local Install Is a Launchable macOS App

For: Maintainers publishing installers and people checking what a download contains.

Given a checkout on an Apple Silicon Mac with the desktop build prerequisites
and write access to `/Applications`, when `just local-install` succeeds, then
`/Applications/Hickory Docs.app` contains a native release build with the UI
embedded and is registered with macOS as **Hickory Docs**. It can be launched
from Applications without a running development server or this checkout. If
the per-user LaunchServices service is unavailable, the installer warns but
still succeeds with the valid app in Applications; macOS registers it when the
service returns.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/a-local-install-is-a-launchable-macos-app.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
