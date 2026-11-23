<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# New Document Creates Another Unnamed Buffer

For: Authors editing executable documents and the files those documents produce.

Given the app has any number of unnamed documents open, when a person chooses
File → New Document, its keyboard shortcut, or the tree's New document action,
then another blank unnamed tab opens and takes focus. Existing tabs and their
contents remain open. Tabs have distinct identities and captions (Untitled,
Untitled 2, and so on), like a conventional text editor.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/new-document-is-an-act.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-519678d8b434"></a>
## `crates/hickory-cli/src/serve/api/create_new.rs`

Related contracts: [New Document Creates Another Unnamed Buffer](contract.md).

<a id="f-519678d8b434-write"></a>
### `write`

<storycode:copy id="f-519678d8b434-write">
use std::io::Write;
use std::path::Path;

use super::{ApiError, ApiResult};

/// Naming a new document never grants permission to truncate an existing file.
/// create_new closes the gap between target inspection and the actual write.
pub(super) fn write(path:&Path,source:&str)->ApiResult<()>{write_observed(path,source).map(|_|())}
pub(super) fn write_observed(path: &Path, source: &str) -> ApiResult<String> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                ApiError::conflict(format!(
                    "{} already exists. Open it, or choose a new filename; no existing bytes were overwritten.",
                    path.display()
                ))
            } else {
                ApiError::internal(format!("could not create {}: {error}", path.display()))
            }
        })?;
    file.write_all(source.as_bytes()).and_then(|_|file.sync_all()).map_err(|error| {
        ApiError::internal(format!(
            "created {} but could not finish writing it: {error}. The new file may contain partial bytes; inspect it before retrying.",
            path.display()
        ))
    })?;
    #[cfg(unix)]std::fs::File::open(path.parent().ok_or_else(||ApiError::internal("New source has no parent"))?).and_then(|directory|directory.sync_all()).map_err(|e|ApiError::internal(e.to_string()))?;
    let identity=super::super::source_history::identity_for_metadata(&file.metadata().map_err(|e|ApiError::internal(e.to_string()))?).map_err(ApiError::internal)?;
    let held=same_file::Handle::from_file(file).map_err(|e|ApiError::internal(e.to_string()))?;
    if same_file::Handle::from_path(path).map_err(|e|ApiError::internal(e.to_string()))?!=held{return Err(ApiError::conflict("Created source name was independently replaced; inspect it before retrying."))}
    Ok(identity)
}

</storycode:copy>
<a id="f-519678d8b434-tests"></a>
### `tests`

<storycode:copy id="f-519678d8b434-tests">
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    // Guarantee: docs/guarantees/authoring/new-document-is-an-act.md
    #[test]
    fn racing_new_document_writes_have_one_winner_and_never_truncate() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("new.md");
        let gate = Arc::new(Barrier::new(2));
        let attempts: Vec<_> = ["first source", "second source"]
            .into_iter()
            .map(|source| {
                let gate = gate.clone();
                let path = path.clone();
                std::thread::spawn(move || {
                    gate.wait();
                    (source, write(&path, source))
                })
            })
            .collect();
        let results: Vec<_> = attempts
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect();
        assert_eq!(
            results.iter().filter(|(_, result)| result.is_ok()).count(),
            1
        );
        let winner = results.iter().find(|(_, result)| result.is_ok()).unwrap().0;
        assert_eq!(std::fs::read_to_string(&path).unwrap(), winner);
        let refused = &results
            .iter()
            .find(|(_, result)| result.is_err())
            .unwrap()
            .1
            .as_ref()
            .unwrap_err();
        assert!(
            refused
                .message()
                .contains("no existing bytes were overwritten")
        );
        assert!(write(&path, "replacement").is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), winner);
    }

    #[cfg(unix)]
    // Guarantee: docs/guarantees/authoring/new-document-is-an-act.md
    #[test]
    fn new_document_refuses_existing_and_dangling_final_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.md");
        std::fs::write(&target, "original").unwrap();
        let link = dir.path().join("link.md");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(write(&link, "overwrite").is_err());
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "original");
        let absent = dir.path().join("absent.md");
        let dangling = dir.path().join("dangling.md");
        std::os::unix::fs::symlink(&absent, &dangling).unwrap();
        assert!(write(&dangling, "new").is_err());
        assert!(!absent.exists());
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/api/create_new.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/api/create_new.rs"><storycode:paste select="#f-519678d8b434-write"/><storycode:paste select="#f-519678d8b434-tests"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/lib/editorBufferTargets.ts](../../../mechanisms/apps/web/src/lib/editorBufferTargets.ts.md#output-17f526449e3c)
- [apps/web/src/views/WorkspaceChat.tsx](../../agent/unsent-responses-survive-restart/contract.md#output-c910953d9e7f)
- apps/web/src/views/WorkspaceView.tsx
- apps/web/src/views/uiWorkspaceMenuActions.ts
- apps/web/src/views/useNewDocument.ts
- apps/web/src/views/useUnsavedLifecycle.ts
- apps/web/src/views/useUntitledRecovery.ts
- apps/web/src/views/useUntitledSave.ts
- apps/web/src/views/workspaceMenuActions.ts
- apps/web/src/views/workspaceState.ts
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)
- [crates/hickory-cli/src/serve/api/create_new.rs](contract.md#output-519678d8b434)

### Tests and fixtures

- apps/web/src/lib/newDoc.test.ts
- [apps/web/src/router.test.ts](../../../mechanisms/apps/web/src/router.test.ts.md#output-415811f3ab59)
- apps/web/src/views/uiWorkspaceMenuActions.test.tsx
- apps/web/src/views/useUnsavedLifecycle.test.tsx
- apps/web/src/views/useUntitledSave.test.tsx
- apps/web/src/views/workspaceMenuActions.test.ts
- apps/web/src/views/workspaceTabs.test.tsx

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
