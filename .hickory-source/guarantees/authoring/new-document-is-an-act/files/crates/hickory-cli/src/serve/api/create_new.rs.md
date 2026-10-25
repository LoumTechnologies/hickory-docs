<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/api/create_new.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [New Document Creates Another Unnamed Buffer](../../../../../../contract.md)

Captured SHA-256: `516e58a44d1cc9c3a1cf083169ebb4513fede55aacb33d135b9a73b803428e8d`.

<book:entry path="crates/hickory-cli/src/serve/api/create_new.rs">
use std::io::Write;
use std::path::Path;

use super::{ApiError, ApiResult};

/// Naming a new document never grants permission to truncate an existing file.
/// create_new closes the gap between target inspection and the actual write.
pub(super) fn write(path: &Path, source: &str) -> ApiResult<()> {
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
    file.write_all(source.as_bytes()).map_err(|error| {
        ApiError::internal(format!(
            "created {} but could not finish writing it: {error}. The new file may contain partial bytes; inspect it before retrying.",
            path.display()
        ))
    })
}

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
</book:entry>

</book:doc>
