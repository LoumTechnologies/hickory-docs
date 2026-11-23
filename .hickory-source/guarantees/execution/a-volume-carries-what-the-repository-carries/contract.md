<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Apply repository exclusions when seeding execution inputs

For: Authors running documents on their own machine and reviewers assessing their side effects.

Mount the repository as a volume. Ignored build outputs must not become accidental inputs, and a cell's own transcript must not make its next recording unreachable.

## What must happen

Repository volume seeding applies the repository's intended ignore filtering. Recording input digests additionally exclude the document's unstable generated products and weave targets. Literal authored inputs remain inputs. Removing the old self-mount warning does not remove ignore filtering or authorize a blanket exclusion of everything under an output directory.

## Limits of the evidence

Volume input filtering and executor confinement are separate boundaries. This does not police arbitrary filesystem writes by an unconfined process.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Keep the original ignore-filter promise and fold in the later removal of the obsolete self-mount warning. Delegate exact digest policy to the recording-key contract.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="f-23d11bf7cb82-a_sibling_documents_kept_recordings_are_not_in_the_key"></a>
### `a_sibling_documents_kept_recordings_are_not_in_the_key`

<storycode:copy id="f-23d11bf7cb82-a_sibling_documents_kept_recordings_are_not_in_the_key">
use super::*;

// Protects docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md
// (the cache-digest half — `mounted_inputs_digest` must apply the same
// `.gitignore` filter `seed_from_directory` already applies at seed time).
#[test]
fn a_sibling_documents_kept_recordings_are_not_in_the_key() {
    let with = b"<h:doc>\n<h:exec container=\"c\">cmd<h:ingested key=\"k\" sha256=\"s\" at=\"d\">\nout\n</h:ingested></h:exec>\n</h:doc>\n";
    let without = b"<h:doc>\n<h:exec container=\"c\">cmd</h:exec>\n</h:doc>\n";
    assert_eq!(strip_kept_recordings(with), without.to_vec());
    assert_eq!(strip_kept_recordings(without), without.to_vec());
}

</storycode:copy>

<a id="f-5a76df8fd4d1-start"></a>
### `start`

A folder with one document in it, a git repository, and a `.gitignore`
that ignores what a .NET restore writes.
The `.gitignore` is not scene-setting: it is the ingest's filter, and
without it `obj/` lands in the document. See
`docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md`.

<storycode:copy id="f-5a76df8fd4d1-start">
/// A folder with one document in it, a git repository, and a `.gitignore`
/// that ignores what a .NET restore writes.
///
/// The `.gitignore` is not scene-setting: it is the ingest's filter, and
/// without it `obj/` lands in the document. See
/// `docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md`.
async fn start() -> Session {
    start_inner(false).await
}

</storycode:copy>

</storycode:doc>
