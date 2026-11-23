<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Unstable Updates Require Consent

## Simple frontend cutover

Download the next Simple build from the Unstable CI release and replace the
installed app explicitly. The in-app signed updater described below was part
of the retired desktop bundle.

## Earlier release design and evidence

For: Maintainers publishing installers and people checking what a download contains.

An Apple Silicon macOS build with the Unstable version and bundled updater
configuration checks the HTTPS manifest after the editor opens and again every
30 minutes. The unsigned CI bundle has no updater configuration.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/unstable-updates-require-consent.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

</storycode:doc>
