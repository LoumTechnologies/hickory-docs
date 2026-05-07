//! MergeOrchestrator: three-way merge algorithm for pipeline snapshots.

use std::collections::{BTreeMap, HashMap};

use log::{debug, info};

use hick_store::{FileProvenance, Snapshot, SnapshotId, StoreError, VersionStore};

use crate::error::MergeError;
use crate::merge_strategy::MergeStrategy;

/// Result of a merge operation.
pub struct MergeResult {
    /// Merged file contents: path → content.
    pub files: HashMap<String, Vec<u8>>,
    /// Files that had true three-way conflicts (resolved by strategy).
    pub conflicts_resolved: Vec<String>,
}

/// Orchestrates three-way merge between pipeline output and user edits.
pub struct MergeOrchestrator<'a> {
    store: &'a dyn VersionStore,
    strategy: &'a dyn MergeStrategy,
}

impl<'a> MergeOrchestrator<'a> {
    pub fn new(store: &'a dyn VersionStore, strategy: &'a dyn MergeStrategy) -> Self {
        Self { store, strategy }
    }

    /// Perform a merge.
    ///
    /// - `branch`: the branch to read the base snapshot from
    /// - `pipeline_files`: generated files from the pipeline (path → content)
    /// - `pipeline_provenance`: provenance for each generated file
    /// - `disk_files`: current files on disk (path → content)
    ///
    /// Returns the merged files and creates a new snapshot.
    pub async fn merge(
        &self,
        branch: &str,
        pipeline_files: &HashMap<String, Vec<u8>>,
        pipeline_provenance: &HashMap<String, FileProvenance>,
        disk_files: &HashMap<String, Vec<u8>>,
    ) -> Result<MergeResult, MergeError> {
        // Get the base snapshot (last snapshot on this branch)
        let base_snapshot = self.get_base_snapshot(branch).await?;

        let mut merged_files = HashMap::new();
        let mut conflicts_resolved = Vec::new();

        // Process each pipeline output file
        for (path, generated) in pipeline_files {
            let provenance = pipeline_provenance.get(path);
            let disk_version = disk_files.get(path);

            let merged = self
                .merge_file(path, generated, provenance, disk_version, &base_snapshot)
                .await?;

            if let MergeFileResult::ConflictResolved = merged.kind {
                conflicts_resolved.push(path.clone());
            }

            merged_files.insert(path.clone(), merged.content);
        }

        // Carry forward UserCreated files from previous snapshot
        if let Some(ref base) = base_snapshot {
            for path in base.files.keys() {
                if !pipeline_files.contains_key(path)
                    && let Some(prov) = base.provenance.get(path)
                    && matches!(prov, FileProvenance::UserCreated)
                {
                    if let Some(disk) = disk_files.get(path) {
                        merged_files.insert(path.clone(), disk.clone());
                    } else {
                        // File deleted by user, don't carry forward
                        debug!("UserCreated file '{path}' no longer on disk, skipping");
                    }
                }
            }
        }

        info!(
            "Merge complete: {} files, {} conflicts resolved",
            merged_files.len(),
            conflicts_resolved.len()
        );

        Ok(MergeResult {
            files: merged_files,
            conflicts_resolved,
        })
    }

    /// Create a snapshot from merge results and advance the branch.
    pub async fn commit(
        &self,
        branch: &str,
        files: &HashMap<String, Vec<u8>>,
        provenance: &HashMap<String, FileProvenance>,
        message: Option<&str>,
    ) -> Result<SnapshotId, MergeError> {
        // Store all blobs
        let mut file_hashes = BTreeMap::new();
        for (path, content) in files {
            let hash = self.store.put_blob(content).await?;
            file_hashes.insert(path.clone(), hash);
        }

        // Get parent
        let parent = self.store.get_branch(branch).await?;
        let parents = parent.into_iter().collect();

        let prov_btree: BTreeMap<String, FileProvenance> = provenance
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let snapshot = Snapshot {
            id: SnapshotId(String::new()),
            parents,
            files: file_hashes,
            provenance: prov_btree,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            message: message.map(|s| s.to_string()),
        };

        let id = self.store.put_snapshot(&snapshot).await?;
        self.store.set_branch(branch, &id).await?;

        info!("Created snapshot {id} on branch '{branch}'");
        Ok(id)
    }

    async fn get_base_snapshot(&self, branch: &str) -> Result<Option<Snapshot>, MergeError> {
        match self.store.get_branch(branch).await? {
            Some(id) => {
                let snap = self.store.get_snapshot(&id).await?;
                Ok(Some(snap))
            }
            None => Ok(None),
        }
    }

    async fn get_base_content(
        &self,
        path: &str,
        snapshot: &Option<Snapshot>,
    ) -> Result<Option<Vec<u8>>, MergeError> {
        if let Some(snap) = snapshot
            && let Some(hash) = snap.files.get(path)
        {
            match self.store.get_blob(hash).await {
                Ok(data) => return Ok(Some(data)),
                Err(StoreError::BlobNotFound(_)) => return Ok(None),
                Err(e) => return Err(MergeError::Store(e)),
            }
        }
        Ok(None)
    }

    async fn merge_file(
        &self,
        path: &str,
        generated: &[u8],
        provenance: Option<&FileProvenance>,
        disk_version: Option<&Vec<u8>>,
        base_snapshot: &Option<Snapshot>,
    ) -> Result<MergeFileOutcome, MergeError> {
        // HickFile provenance → always use generated (user edits .hick source, not output)
        if matches!(provenance, Some(FileProvenance::HickFile { .. })) {
            return Ok(MergeFileOutcome {
                content: generated.to_vec(),
                kind: MergeFileResult::UseGenerated,
            });
        }

        // No disk version → use generated (first run or file was deleted)
        let edited = match disk_version {
            Some(v) => v.as_slice(),
            None => {
                return Ok(MergeFileOutcome {
                    content: generated.to_vec(),
                    kind: MergeFileResult::UseGenerated,
                });
            }
        };

        // Get base version
        let base = self.get_base_content(path, base_snapshot).await?;
        let base = base.as_deref().unwrap_or(&[]);

        // Three-way comparison
        if edited == base {
            // No user edits → use generated
            Ok(MergeFileOutcome {
                content: generated.to_vec(),
                kind: MergeFileResult::UseGenerated,
            })
        } else if generated == base {
            // No pipeline changes → keep edited
            Ok(MergeFileOutcome {
                content: edited.to_vec(),
                kind: MergeFileResult::KeepEdited,
            })
        } else if edited == generated {
            // Converged → use either
            Ok(MergeFileOutcome {
                content: generated.to_vec(),
                kind: MergeFileResult::Converged,
            })
        } else {
            // True three-way conflict → delegate to strategy
            let merged = self.strategy.merge(path, base, generated, edited).await?;
            Ok(MergeFileOutcome {
                content: merged,
                kind: MergeFileResult::ConflictResolved,
            })
        }
    }
}

struct MergeFileOutcome {
    content: Vec<u8>,
    kind: MergeFileResult,
}

enum MergeFileResult {
    UseGenerated,
    KeepEdited,
    Converged,
    ConflictResolved,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::trivial_strategy::TakeGenerated;
    use hick_store::{BuiltinVersionStore, InMemoryObjectStore};

    fn make_store() -> Arc<BuiltinVersionStore> {
        Arc::new(BuiltinVersionStore::new(Arc::new(
            InMemoryObjectStore::new(),
        )))
    }

    #[tokio::test]
    async fn first_run_no_base() {
        let store = make_store();
        let strategy = TakeGenerated;
        let orch = MergeOrchestrator::new(store.as_ref(), &strategy);

        let pipeline_files = HashMap::from([
            ("a.txt".to_string(), b"aaa".to_vec()),
            ("b.txt".to_string(), b"bbb".to_vec()),
        ]);
        let provenance = HashMap::from([(
            "a.txt".to_string(),
            FileProvenance::HickFile {
                source: "main.hick".to_string(),
            },
        )]);

        let result = orch
            .merge("main", &pipeline_files, &provenance, &HashMap::new())
            .await
            .unwrap();

        assert_eq!(result.files.len(), 2);
        assert_eq!(result.files["a.txt"], b"aaa");
        assert_eq!(result.files["b.txt"], b"bbb");
        assert!(result.conflicts_resolved.is_empty());
    }

    #[tokio::test]
    async fn no_user_edits_uses_generated() {
        let store = make_store();
        let strategy = TakeGenerated;
        let orch = MergeOrchestrator::new(store.as_ref(), &strategy);

        // Create initial snapshot
        let initial = HashMap::from([("file.txt".to_string(), b"v1".to_vec())]);
        let prov = HashMap::from([(
            "file.txt".to_string(),
            FileProvenance::ContainerOutput {
                container: "build".to_string(),
                volume: "out".to_string(),
            },
        )]);
        orch.commit("main", &initial, &prov, Some("initial"))
            .await
            .unwrap();

        // Second run: pipeline produces v2, disk still has v1 (no edits)
        let pipeline = HashMap::from([("file.txt".to_string(), b"v2".to_vec())]);
        let disk = HashMap::from([("file.txt".to_string(), b"v1".to_vec())]);

        let result = orch.merge("main", &pipeline, &prov, &disk).await.unwrap();
        assert_eq!(result.files["file.txt"], b"v2"); // Uses generated
        assert!(result.conflicts_resolved.is_empty());
    }

    #[tokio::test]
    async fn user_edits_no_pipeline_changes() {
        let store = make_store();
        let strategy = TakeGenerated;
        let orch = MergeOrchestrator::new(store.as_ref(), &strategy);

        // Create initial snapshot
        let initial = HashMap::from([("file.txt".to_string(), b"v1".to_vec())]);
        let prov = HashMap::from([(
            "file.txt".to_string(),
            FileProvenance::ContainerOutput {
                container: "build".to_string(),
                volume: "out".to_string(),
            },
        )]);
        orch.commit("main", &initial, &prov, Some("initial"))
            .await
            .unwrap();

        // Second run: pipeline still produces v1, but user edited to v1-edited
        let pipeline = HashMap::from([("file.txt".to_string(), b"v1".to_vec())]);
        let disk = HashMap::from([("file.txt".to_string(), b"v1-edited".to_vec())]);

        let result = orch.merge("main", &pipeline, &prov, &disk).await.unwrap();
        assert_eq!(result.files["file.txt"], b"v1-edited"); // Keeps user edits
    }

    #[tokio::test]
    async fn true_conflict_calls_strategy() {
        let store = make_store();
        let strategy = TakeGenerated; // Will take the generated version
        let orch = MergeOrchestrator::new(store.as_ref(), &strategy);

        // Create initial snapshot
        let initial = HashMap::from([("file.txt".to_string(), b"v1".to_vec())]);
        let prov = HashMap::from([(
            "file.txt".to_string(),
            FileProvenance::ContainerOutput {
                container: "build".to_string(),
                volume: "out".to_string(),
            },
        )]);
        orch.commit("main", &initial, &prov, Some("initial"))
            .await
            .unwrap();

        // Both pipeline AND user changed the file
        let pipeline = HashMap::from([("file.txt".to_string(), b"v2-pipeline".to_vec())]);
        let disk = HashMap::from([("file.txt".to_string(), b"v1-edited".to_vec())]);

        let result = orch.merge("main", &pipeline, &prov, &disk).await.unwrap();
        assert_eq!(result.files["file.txt"], b"v2-pipeline"); // Strategy chose generated
        assert_eq!(result.conflicts_resolved, vec!["file.txt"]);
    }

    #[tokio::test]
    async fn hick_file_always_regenerated() {
        let store = make_store();
        let strategy = TakeGenerated;
        let orch = MergeOrchestrator::new(store.as_ref(), &strategy);

        // Create initial snapshot
        let initial = HashMap::from([("output.html".to_string(), b"v1".to_vec())]);
        let prov = HashMap::from([(
            "output.html".to_string(),
            FileProvenance::HickFile {
                source: "main.hick".to_string(),
            },
        )]);
        orch.commit("main", &initial, &prov, Some("initial"))
            .await
            .unwrap();

        // Even though user "edited" the file, HickFile provenance means use generated
        let pipeline = HashMap::from([("output.html".to_string(), b"v2-generated".to_vec())]);
        let disk = HashMap::from([("output.html".to_string(), b"v1-user-edited".to_vec())]);

        let result = orch.merge("main", &pipeline, &prov, &disk).await.unwrap();
        assert_eq!(result.files["output.html"], b"v2-generated");
        assert!(result.conflicts_resolved.is_empty()); // Not counted as conflict
    }

    #[tokio::test]
    async fn user_created_files_carried_forward() {
        let store = make_store();
        let strategy = TakeGenerated;
        let orch = MergeOrchestrator::new(store.as_ref(), &strategy);

        // Create initial snapshot with a user-created file
        let initial = HashMap::from([
            ("generated.txt".to_string(), b"gen".to_vec()),
            ("user-notes.txt".to_string(), b"my notes".to_vec()),
        ]);
        let prov = HashMap::from([
            (
                "generated.txt".to_string(),
                FileProvenance::ContainerOutput {
                    container: "build".to_string(),
                    volume: "out".to_string(),
                },
            ),
            ("user-notes.txt".to_string(), FileProvenance::UserCreated),
        ]);
        orch.commit("main", &initial, &prov, Some("initial"))
            .await
            .unwrap();

        // Pipeline only produces generated.txt, not user-notes.txt
        let pipeline = HashMap::from([("generated.txt".to_string(), b"gen-v2".to_vec())]);
        let disk = HashMap::from([
            ("generated.txt".to_string(), b"gen".to_vec()),
            ("user-notes.txt".to_string(), b"updated notes".to_vec()),
        ]);

        let result = orch.merge("main", &pipeline, &prov, &disk).await.unwrap();
        assert_eq!(result.files["generated.txt"], b"gen-v2");
        assert_eq!(result.files["user-notes.txt"], b"updated notes"); // Carried forward
    }
}
