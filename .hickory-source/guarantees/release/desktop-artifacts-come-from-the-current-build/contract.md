<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Desktop collection excludes unchanged cached installers

## Simple frontend cutover

Each desktop package comes from the selected revision. The reusable workflow
builds and checks the macOS bundle, KDE installation tree and WinForms
workspace side by side with the CLI archives, then uploads their checksums.

## Earlier release design and evidence

For: Maintainers publishing installers and people checking what a download contains.

Given an isolated desktop bundle build, when `scripts/dist-desktop.sh` collects
installable files, then it selects only files whose existence or observed
identity changed since the invocation's pre-build snapshot. An unchanged cached
installer is never renamed as the current version merely because it remains in
the bundle directory. No changed installable file, or multiple changed files of
one format, refuses collection before replacing the previous dist selection.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/desktop-artifacts-come-from-the-current-build.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

</storycode:doc>
