//! Workspace inventories and editor identities adapted to the portable tree verifier.
// Guarantee: docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md
use crate::serve::agent_context::EditorBuffer;
use anyhow::{Context, Result, ensure};
use hick_lang::{ProjectEntry, ProjectInventory, ProjectTree};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

/// Reserved evaluator namespace. These entries are editor addresses, never disk destinations.
pub const BUFFERS: &str = hick_lang::EDITOR_BUFFER_PREFIX;
#[derive(Clone, Serialize, Deserialize)]
pub struct WorkspaceCapture {
    pub root: Option<PathBuf>,
    pub identities: BTreeMap<String, Option<String>>,
    #[serde(default)]
    pub provenance: BTreeMap<String, Option<String>>,
}
impl WorkspaceCapture {
    pub fn freeze(root: Option<PathBuf>, buffers: &[EditorBuffer]) -> Result<(Self, ProjectTree)> {
        let mut scope = Self {
            root,
            identities: BTreeMap::new(),
            provenance: BTreeMap::new(),
        };
        for buffer in buffers {
            let id = buffer
                .id
                .as_ref()
                .context("Editor capture needs stable buffer IDs; reopen the editor and retry.")?;
            ensure!(
                !id.is_empty() && !scope.identities.contains_key(id),
                "Duplicate or empty editor ID: {id}"
            );
            scope.identities.insert(id.clone(), buffer.path.clone());
            scope
                .provenance
                .insert(id.clone(), buffer.provenance_uri.clone());
        }
        let tree = scope.current(buffers)?;
        Ok((scope, tree))
    }
    pub fn current(&self, buffers: &[EditorBuffer]) -> Result<ProjectTree> {
        let identities: BTreeMap<_, _> = buffers
            .iter()
            .map(|b| (b.id.clone().unwrap_or_default(), b.path.clone()))
            .collect();
        ensure!(
            identities.len() == buffers.len() && identities == self.identities,
            "Editor identity changed (opened, closed, saved or renamed). Restore the original editors and paths, then retry; start a new turn to use a new boundary."
        );
        if !self.provenance.is_empty() {
            let provenance: BTreeMap<_, _> = buffers
                .iter()
                .map(|b| (b.id.clone().unwrap_or_default(), b.provenance_uri.clone()))
                .collect();
            ensure!(
                provenance == self.provenance,
                "Editor identity changed (window or buffer instance). Reopen the original captured window before retrying."
            );
        }
        let mut tree = if let Some(root) = &self.root {
            {
                let first =
                    crate::project_revision::directory(root, Some(&ProjectInventory::default()))?;
                let second =
                    crate::project_revision::directory(root, Some(&ProjectInventory::default()))?;
                ensure!(
                    first.compare(&second).is_empty(),
                    "Workspace changed while capturing. Pause other writers and retry changeset_status."
                );
                second
            }
        } else {
            ProjectTree::default()
        };
        ensure!(
            !tree
                .entries()
                .keys()
                .any(|p| p == BUFFERS.trim_end_matches('/') || p.starts_with(BUFFERS)),
            "Workspace uses reserved editor namespace {BUFFERS}; move it before starting a capture."
        );
        let mut destinations = BTreeMap::new();
        for buffer in buffers {
            let id = buffer.id.as_ref().context("Missing editor ID")?;
            let path = if let Some(name) = &buffer.path {
                let root = self.root.as_ref().context("A filesystem-backed editor needs an open folder; open its folder before capturing.")?;
                let absolute = if std::path::Path::new(name).is_absolute() {
                    PathBuf::from(name)
                } else {
                    root.join(name)
                };
                let relative = absolute.strip_prefix(root).context(
                    "Editor is outside the captured workspace; open its folder before capturing.",
                )?;
                let path = relative
                    .to_str()
                    .context("Editor path must be UTF-8")?
                    .replace('\\', "/");
                hick_lang::validate_project_path(&path).map_err(anyhow::Error::msg)?;
                ensure!(
                    ProjectInventory::default().contains(&path),
                    "Editor {name} is in an excluded metadata/session directory; move it before capturing."
                );
                let mut parent = root.clone();
                let components: Vec<_> = path.split('/').collect();
                for (at, component) in components.iter().enumerate() {
                    parent.push(component);
                    match std::fs::symlink_metadata(&parent) {
                        Ok(meta) => ensure!(
                            !meta.file_type().is_symlink()
                                && if at + 1 == components.len() {
                                    meta.is_file()
                                } else {
                                    meta.is_dir()
                                },
                            "Editor {name} crosses a symlink or non-file destination; open its real workspace path before capturing."
                        ),
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
                        Err(error) => {
                            return Err(error).with_context(|| {
                                format!("Cannot inspect editor destination {name}")
                            });
                        }
                    }
                }
                // A deleted backing file and its still-open buffer are independent
                // captured states. No disk bytes are synthesized from the buffer.
                path
            } else {
                let encoded = id
                    .as_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>();
                format!("{BUFFERS}{encoded}")
            };
            if let Some(previous) = destinations.insert(path.clone(), buffer.content.clone()) {
                ensure!(
                    previous == buffer.content,
                    "Editors disagree about {path}; reconcile them before capturing."
                );
            }
            // Disk and editor are separate states. Keeping both prevents a dirty
            // editor from concealing a native write to its backing file.
            let encoded = id
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            let path = format!("{BUFFERS}{encoded}");
            let mut entry = ProjectEntry::file(Vec::new());
            entry.bytes = buffer.content.as_bytes().to_vec().into();
            tree.insert(&path, entry).map_err(anyhow::Error::msg)?;
        }
        Ok(tree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn buffers(path: Option<&str>) -> Vec<EditorBuffer> {
        serde_json::from_value(json!([
            {"id":"saved","name":"Note","path":path,"content":"dirty A\n","focused":true},
            {"id":"untitled:/two","name":"Note","path":null,"content":"second A\n"}
        ]))
        .unwrap()
    }
    #[test]
    fn workspace_and_editor_channels_capture_complete_changes_and_repair() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("note.txt"), "disk A\n").unwrap();
        std::fs::write(dir.path().join("response.md"), "ordinary markdown\n").unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join(".git/ignored"), "metadata").unwrap();
        let mut buffers = buffers(Some("note.txt"));
        let (scope, a) =
            WorkspaceCapture::freeze(Some(dir.path().to_path_buf()), &buffers).unwrap();
        assert_eq!(&*a.get("note.txt").unwrap().bytes, b"disk A\n");
        assert!(a.entries().values().any(|e| &*e.bytes == b"dirty A\n"));
        assert!(a.get(".git/ignored").is_none());
        std::fs::rename(
            dir.path().join("response.md"),
            dir.path().join("renamed.md"),
        )
        .unwrap();
        std::fs::write(dir.path().join("created.txt"), "new\n").unwrap();
        std::fs::write(dir.path().join("note.txt"), "native B\n").unwrap();
        buffers[0].content = "editor B\n".into();
        buffers[1].content = "second B\n".into();
        let b = scope.current(&buffers).unwrap();
        assert!(a.compare(&b).len() >= 6);
        let edits = std::sync::Arc::new(crate::serve::acp::edits::Edits::default());
        *edits.buffers.lock().unwrap() = buffers.clone();
        let mut contract = crate::changeset::Contract::new(
            dir.path().into(),
            ProjectInventory::default(),
            a,
            "turn".into(),
        );
        contract.workspace = Some((scope.clone(), edits.clone()));
        let status = contract.status().unwrap();
        assert_eq!(status["entry"], "_response.md");
        let mut submission = json!({"boundary":"turn","expected":status["expected"],"entry":status["entry"],"sources":{"_response.md":"<hick:project/>"}});
        assert_eq!(contract.submit(&submission).unwrap()["accepted"], false);
        submission["sources"]["_response.md"] = status["template"].clone();
        assert_eq!(contract.submit(&submission).unwrap()["accepted"], true);
        assert!(contract.completion().is_ok());
        edits.buffers.lock().unwrap()[1].content = "concurrent edit\n".into();
        assert!(contract.completion().is_err());
        let status = contract.status().unwrap();
        submission["expected"] = status["expected"].clone();
        submission["sources"]["_response.md"] = status["template"].clone();
        assert_eq!(contract.submit(&submission).unwrap()["accepted"], true);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("note.txt")).unwrap(),
            "native B\n"
        );
        assert!(!dir.path().join("_response.md").exists());
        std::fs::remove_file(dir.path().join("note.txt")).unwrap();
        let deleted = scope.current(&buffers).unwrap();
        assert!(deleted.get("note.txt").is_none());
        assert!(
            deleted
                .entries()
                .values()
                .any(|e| &*e.bytes == b"editor B\n")
        );
        assert!(contract.completion().is_err());
    }
    #[test]
    fn no_folder_uses_only_editor_identities_and_refuses_transitions() {
        let mut buffers = buffers(None);
        let (scope, a) = WorkspaceCapture::freeze(None, &buffers).unwrap();
        assert_eq!(a.entries().len(), 2);
        assert!(a.entries().keys().all(|p| p.starts_with(BUFFERS)));
        buffers[0].content = "B\n".into();
        let b = scope.current(&buffers).unwrap();
        let response = hick_lang::transition_response(&a, &b, "Edited untitled note").unwrap();
        assert!(
            crate::changeset::verify_sources(
                &a,
                &b,
                "response.md",
                &BTreeMap::from([("response.md".into(), response)])
            )
            .unwrap()
            .is_empty()
        );
        let original = buffers.clone();
        buffers[0].path = Some("invented.md".into());
        assert!(
            scope
                .current(&buffers)
                .unwrap_err()
                .to_string()
                .contains("identity changed")
        );
        buffers = original.clone();
        buffers.pop();
        assert!(scope.current(&buffers).is_err());
        buffers = original;
        let mut replacement = buffers.clone();
        replacement[0].provenance_uri = Some("new-window-buffer".into());
        assert!(scope.current(&replacement).is_err());
        buffers[0].id = Some("renamed".into());
        assert!(scope.current(&buffers).is_err());
    }
    #[test]
    fn conflicting_editors_and_outside_or_excluded_paths_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("note.txt"), "A").unwrap();
        let mut buffers = buffers(Some("note.txt"));
        buffers[1].path = Some("note.txt".into());
        assert!(WorkspaceCapture::freeze(Some(dir.path().into()), &buffers).is_err());
        buffers[1].path = None;
        for path in ["../outside", ".git/config"] {
            buffers[0].path = Some(path.into());
            assert!(WorkspaceCapture::freeze(Some(dir.path().into()), &buffers).is_err());
        }
    }
}

#[cfg(test)]
mod native_guards {
    use super::*;
    #[cfg(unix)]
    use hick_lang::EntryKind;
    #[test]
    fn editor_addresses_cannot_be_materialized() {
        let mut tree = ProjectTree::default();
        tree.insert("ordinary.txt", ProjectEntry::file(b"do not write".to_vec()))
            .unwrap();
        tree.insert(
            &format!("{BUFFERS}1234"),
            ProjectEntry::file(b"editor only".to_vec()),
        )
        .unwrap();
        let dir = tempfile::tempdir().unwrap();
        assert!(
            hick_project_env::write_tree(dir.path(), &tree)
                .unwrap_err()
                .contains("no filesystem destination")
        );
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }
    #[cfg(unix)]
    #[test]
    fn workspace_metadata_changes_participate_without_following_links() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("run.sh"), "echo hi\n").unwrap();
        let (scope, a) = WorkspaceCapture::freeze(Some(dir.path().into()), &[]).unwrap();
        std::fs::set_permissions(
            dir.path().join("run.sh"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        symlink("run.sh", dir.path().join("link")).unwrap();
        let b = scope.current(&[]).unwrap();
        assert_eq!(a.compare(&b).len(), 2);
        assert!(b.get("run.sh").unwrap().executable);
        assert_eq!(b.get("link").unwrap().kind, EntryKind::Symlink);
        assert!(hick_lang::transition_response(&a, &b, "metadata").is_err());
        symlink(dir.path(), dir.path().join("alias")).unwrap();
        let buffers: Vec<EditorBuffer> = serde_json::from_value(serde_json::json!([{"id":"alias","name":"Alias","path":"alias/run.sh","content":"bytes"}])).unwrap();
        assert!(WorkspaceCapture::freeze(Some(dir.path().into()), &buffers).is_err());
        std::fs::remove_file(dir.path().join("alias")).unwrap();
        let sources = BTreeMap::from([("response.md".into(), "<hick:project/><hick:patch id=\"remove\">\n--- a/run.sh\n+++ /dev/null\n@@ -1 +0,0 @@\n-echo hi\n</hick:patch><hick:file path=\"run.sh\" executable=\"true\" after=\"remove\">echo hi\n</hick:file><hick:entry path=\"link\" kind=\"symlink\">run.sh</hick:entry>".into())]);
        assert!(
            crate::changeset::verify_sources(&a, &b, "response.md", &sources)
                .unwrap()
                .is_empty()
        );
    }
}
