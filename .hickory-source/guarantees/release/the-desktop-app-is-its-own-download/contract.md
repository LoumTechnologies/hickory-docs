<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# The Desktop App Is Its Own Download

For: Maintainers publishing installers and people checking what a download contains.

Given a person who wants a window rather than a terminal, when they go to the
releases page, then they find an installer in their platform's own shape —
`.dmg` on macOS, `.msi` on Windows, `.deb` and `.AppImage` on Linux — built
from the same commit and carrying the same version as the CLI archives beside
it, and installing it requires no Rust toolchain, no Node, and no checkout.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/the-desktop-app-is-its-own-download.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
