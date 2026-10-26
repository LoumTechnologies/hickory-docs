<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
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


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/the-desktop-app-is-its-own-download.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Implementation review

### Implementation

- [scripts/build-python-agent-runtime.sh](../../../mechanisms/scripts/build-python-agent-runtime.sh.md#output-5f2e12b18746)
- [scripts/dist-desktop.sh](../../../mechanisms/scripts/dist-desktop.sh.md#output-928dfd6fa282)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
