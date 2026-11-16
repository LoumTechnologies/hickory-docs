//! Native publication of an already evaluated project value; no parsing.
use hick_lang::{EntryKind, ProjectTree};
use std::path::{Path, PathBuf};
/// Write an evaluated tree. Validate every destination and supported metadata
/// before any write; never follow a pre-existing symlink in the destination.
pub fn write_tree(root: &Path, tree: &ProjectTree) -> Result<Vec<PathBuf>, String> {
    for (path, entry) in tree.entries() {
        if path.starts_with(hick_lang::EDITOR_BUFFER_PREFIX) {
            return Err(format!(
                "Editor identity has no filesystem destination: {path}"
            ));
        }
        hick_lang::validate_project_path(path)?;
        let mut parent = root.to_path_buf();
        for part in path.split('/') {
            parent.push(part);
            if let Ok(metadata) = std::fs::symlink_metadata(&parent) {
                let destination = parent == root.join(path);
                let replace_link = destination
                    && metadata.file_type().is_symlink()
                    && entry.kind == EntryKind::Symlink;
                if (metadata.file_type().is_symlink() && !replace_link)
                    || (!metadata.is_dir() && !destination)
                {
                    return Err(format!(
                        "refusing materialization through existing non-directory or symlink: {}",
                        parent.display()
                    ));
                }
                if destination && !metadata.is_file() && !replace_link {
                    return Err(format!("occupied project output: {path}"));
                }
            }
        }
        if entry.kind == EntryKind::Symlink {
            let target = std::str::from_utf8(&entry.bytes)
                .map_err(|_| "unsupported non-UTF-8 symlink target")?;
            hick_lang::validate_project_path(target).map_err(|_|format!("native materialization does not support escaping or ambiguous symlink target: {path}"))?;
            #[cfg(not(unix))]
            return Err(format!(
                "symlink materialization is unsupported on this platform: {path}"
            ));
        }
        #[cfg(not(unix))]
        if entry.executable {
            return Err(format!(
                "executable mode materialization is unsupported on this platform: {path}"
            ));
        }
    }
    let mut written = vec![];
    for (path, entry) in tree.entries() {
        let full = root.join(path);
        if materialized_entry_matches(&full, entry) {
            written.push(full);
            continue;
        }
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        if entry.kind == EntryKind::File {
            std::fs::write(&full, &entry.bytes).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(
                    &full,
                    std::fs::Permissions::from_mode(if entry.executable { 0o755 } else { 0o644 }),
                )
                .map_err(|e| e.to_string())?;
            }
        } else {
            #[cfg(unix)]
            {
                if std::fs::symlink_metadata(&full).is_ok() {
                    std::fs::remove_file(&full).map_err(|e| e.to_string())?;
                }
                std::os::unix::fs::symlink(
                    std::str::from_utf8(&entry.bytes).map_err(|e| e.to_string())?,
                    &full,
                )
                .map_err(|e| e.to_string())?;
            }
        }
        written.push(full);
    }
    Ok(written)
}
fn materialized_entry_matches(path: &Path, entry: &hick_lang::ProjectEntry) -> bool {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    match entry.kind {
        EntryKind::File => {
            if !metadata.is_file() {
                return false;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let expected = if entry.executable { 0o755 } else { 0o644 };
                if metadata.permissions().mode() & 0o7777 != expected {
                    return false;
                }
            }
            std::fs::read(path).is_ok_and(|bytes| bytes.as_slice() == entry.bytes.as_ref())
        }
        EntryKind::Symlink => {
            if !metadata.file_type().is_symlink() {
                return false;
            }
            let Ok(expected) = std::str::from_utf8(&entry.bytes) else {
                return false;
            };
            std::fs::read_link(path)
                .is_ok_and(|target| target.as_os_str() == std::ffi::OsStr::new(expected))
        }
    }
}
#[cfg(test)]
mod stable_materialization_tests {
    use super::*;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    fn old_time(path: &Path) -> SystemTime {
        let expected = UNIX_EPOCH + Duration::from_secs(1_234_567_890);
        std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(expected))
            .unwrap();
        expected
    }
    #[test]
    fn unchanged_tree_keeps_output_times_and_changed_bytes_still_publish() {
        let root = tempfile::tempdir().unwrap();
        let mut tree = ProjectTree::default();
        tree.insert(
            "source.txt",
            hick_lang::ProjectEntry::file(b"first".to_vec()),
        )
        .unwrap();
        let paths = write_tree(root.path(), &tree).unwrap();
        let old = old_time(&paths[0]);
        assert_eq!(write_tree(root.path(), &tree).unwrap(), paths);
        assert_eq!(
            std::fs::metadata(&paths[0]).unwrap().modified().unwrap(),
            old
        );
        tree.insert(
            "source.txt",
            hick_lang::ProjectEntry::file(b"second".to_vec()),
        )
        .unwrap();
        write_tree(root.path(), &tree).unwrap();
        assert_eq!(std::fs::read(&paths[0]).unwrap(), b"second");
        assert_ne!(
            std::fs::metadata(&paths[0]).unwrap().modified().unwrap(),
            old
        );
    }
    #[cfg(unix)]
    #[test]
    fn equal_bytes_do_not_hide_a_wrong_mode_or_leaf_symlink() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let mut tree = ProjectTree::default();
        let mut entry = hick_lang::ProjectEntry::file(b"same".to_vec());
        entry.executable = true;
        tree.insert("run", entry).unwrap();
        write_tree(root.path(), &tree).unwrap();
        std::fs::set_permissions(
            root.path().join("run"),
            std::fs::Permissions::from_mode(0o644),
        )
        .unwrap();
        write_tree(root.path(), &tree).unwrap();
        assert_eq!(
            std::fs::metadata(root.path().join("run"))
                .unwrap()
                .permissions()
                .mode()
                & 0o7777,
            0o755
        );
        let outside = tempfile::NamedTempFile::new().unwrap();
        std::fs::remove_file(root.path().join("run")).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("run")).unwrap();
        assert!(write_tree(root.path(), &tree).is_err());
        assert_eq!(std::fs::read(outside.path()).unwrap(), b"");
    }
    #[test]
    fn unchanged_fast_path_does_not_bypass_plan_validation() {
        let root = tempfile::tempdir().unwrap();
        let mut tree = ProjectTree::default();
        tree.insert("a", hick_lang::ProjectEntry::file(b"same".to_vec()))
            .unwrap();
        write_tree(root.path(), &tree).unwrap();
        let old = old_time(&root.path().join("a"));
        let identity = format!("{}unsafe", hick_lang::EDITOR_BUFFER_PREFIX);
        tree.insert(
            &identity,
            hick_lang::ProjectEntry::file(b"not a filesystem output".to_vec()),
        )
        .unwrap();
        assert!(write_tree(root.path(), &tree).is_err());
        assert_eq!(
            std::fs::metadata(root.path().join("a"))
                .unwrap()
                .modified()
                .unwrap(),
            old
        );
    }
}
