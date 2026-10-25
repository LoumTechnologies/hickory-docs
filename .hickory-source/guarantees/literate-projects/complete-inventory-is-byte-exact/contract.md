<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Check every source file, including assets and permissions

For: Reviewers deciding whether a reorganized source still constructs the same program.

Changing one byte of an image, removing a migration, or adding an unexplained lockfile must fail equality even when every Rust file matches.

## What must happen

Given generated and expected trees under the declared inventory policy, equality compares the complete path inventory, exact content bytes, entry kinds, executable status, and symlink targets. Missing, extra, or differing entries fail. Only the source closure, declared reading targets, fixed metadata directories, and explicit exclusions are omitted. Unknown kinds and ambiguous portable paths fail; Git ignore rules and filename extensions do not decide source membership.


## Limits of the evidence

Executable versus ordinary is the portable permission model. Ownership, timestamps, empty directories and all POSIX permission bits are outside it. The captured repository has no symlink entries; the focused inventory test exercises virtual symlink metadata.

## Inspect implementation and tests

[Follow the actual construction, named witnesses and evidence gaps](review.md).

<book:include file="review.md" />

Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/complete-inventory-is-byte-exact.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Keep complete source equality separate from test success and source-edit reversibility.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

</book:doc>
