<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Publish the committed Unstable build chosen by the maintainer

For: Maintainers publishing installers and people checking what a download contains.

Publish Unstable from the configured Linux-to-Mac workflow. The payloads must come from the committed revision; the tool must not invent a green-CI gate that this workflow explicitly removed.

## What must happen

Manual Unstable publication builds the exact committed HEAD present on GitHub through the configured Apple Silicon SSH builder. It uploads the DMG and signed app archive, then switches the updater manifest only after payload uploads succeed. Existing assets stay downloadable. Publishers using that builder share its lock. The command neither dispatches GitHub Actions nor gates or monitors regression results; the maintainer chooses publication timing. Stable release behavior is separate.

## Limits of the evidence

No publication is authorized or performed by this rewrite. Builder/signing configuration and real installability remain unverified. Manifest replacement is not atomic across GitHub failures, and separate builders can race.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/only-a-green-commit-becomes-downloadable.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: The explicit October 25 manual-publication requirement supersedes the older automatic green-CI-only Unstable contract. Keep old stable-channel rules historical until separately reviewed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Source constructed here

[Open this behavior’s construction inputs](construction.md).

<book:include file="construction.md" />

</book:doc>
