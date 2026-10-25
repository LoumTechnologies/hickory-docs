<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Download Runs Without A Rust Toolchain

For: Maintainers publishing installers and people checking what a download contains.

Given a machine with no Rust toolchain, no C compiler, and no checkout of this
repository, when a person downloads the release archive matching their
platform and unpacks it, then `hick --version` and `hick test examples/`
both work — using only the archive's own contents plus whatever the example
documents themselves invoke.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
<book:include file="requirements/01.md" />
- [Requirements 2](requirements/02.md)
<book:include file="requirements/02.md" />

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/a-download-runs-without-a-rust-toolchain.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
