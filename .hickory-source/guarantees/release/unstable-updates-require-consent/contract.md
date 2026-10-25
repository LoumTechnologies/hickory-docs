<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Unstable Updates Require Consent

For: Maintainers publishing installers and people checking what a download contains.

Given a packaged release build for Apple Silicon macOS whose SemVer prerelease
channel is `unstable`, when its editor window has opened, then the app checks
the configured HTTPS GitHub Unstable manifest and checks again every 30 minutes.
Stable builds, development builds, and other platforms do not start this loop.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/unstable-updates-require-consent.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
