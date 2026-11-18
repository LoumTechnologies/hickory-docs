//! Read historical interrupted material without reconstructing live authority.
use super::{EntryChange, EntryStamp};
use crate::serve::{
    api::{ApiError, ApiResult},
    LocalState,
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Attempt {
    pub request_id: String,
    pub context: String,
    pub before_entry: EntryStamp,
    pub after_entry: EntryStamp,
    pub reviewed_fingerprint: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PathReading {
    pub path: String,
    pub saved_kind: String,
    pub save_guard: Option<String>,
    pub source_hash: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Material {
    pub version: u32,
    pub workspace: String,
    pub namespace: String,
    pub current_entry: EntryStamp,
    pub request_id: String,
    pub manifest_sha256: String,
    pub change: EntryChange,
    pub namespace_evidence: Value,
    pub paths: Vec<PathReading>,
    pub effective_entry: Option<String>,
    pub reservation_floor: u32,
    pub attempts: Vec<Attempt>,
    pub can_complete: bool,
    pub notice: Option<String>,
    pub source_files_modified: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Reading {
    pub material: Material,
    pub fingerprint: String,
}
#[derive(Clone,Default,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedCapture {
    #[serde(default)]pub drafts:std::collections::BTreeMap<String,crate::serve::resource_inputs::Draft>,
    #[serde(default)]pub ownership_sources:std::collections::BTreeMap<String,String>,
    #[serde(default)]pub unaddressed_sources:std::collections::BTreeMap<String,String>,
    #[serde(default)]pub saved_guards:std::collections::BTreeMap<String,String>,
}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AttemptSelector {pub request_id:String,pub context:String}
#[derive(Clone,Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IntentRequest {
    pub version:u32,pub reading:Reading,pub retained_capture:RetainedCapture,
    #[serde(default)]pub supersede_request:Option<AttemptSelector>,
}
#[derive(Clone,Serialize)]
pub(crate) struct IntentExtra {
    pub interrupted_reading_fingerprint:String,
    pub supersedes:Option<Attempt>,
}
#[derive(Serialize)]
pub(crate) struct Envelope {
    version: u32,
    workspace: String,
    namespace: String,
    interrupted: Option<Reading>,
    source_files_modified: bool,
}
#[cfg(unix)]
mod unix {
    use super::*;
    use crate::serve::resource_inputs::SavedSource;
    use crate::serve::resource_journal::{Manifest, read_recovery};
    use std::collections::BTreeMap;
    use std::path::Path;
    use crate::serve::workspace_entry_authority::{leases, storage};
    const DOMAIN: &[u8] = b"hickory.workspace-entry.interrupted-reading.v1";
    fn exact_manifest(root: &Path, request: &str, sha256: &str) -> Result<Manifest, String> {
        let record = read_recovery(root, request)?;
        if record.manifest_sha256 != sha256 {
            return Err("Interrupted source journal differs from its original sealed manifest".into());
        }
        Ok(record.manifest)
    }

    pub(in crate::serve::workspace_entry_authority) struct Namespace {
        pub original: Manifest,
        pub saved: BTreeMap<String, SavedSource>,
        pub original_guard: Option<String>,
    }

    fn read_state(
        state: &LocalState,
        bound: &storage::Bound,
    ) -> Result<(Reading, Option<Namespace>), String> {
        let root = state.index.root();
        let workspace = state.resource_root.check(root)?.to_owned();
        let namespace = root
            .canonicalize()
            .map_err(|error| error.to_string())?
            .to_str()
            .ok_or("Workspace namespace is not exact UTF-8")?
            .to_owned();
        let pending = bound
            .payload
            .pending
            .as_ref()
            .ok_or("There is no interrupted entry transition to review")?;
        let active = exact_manifest(root, &pending.request_id, &pending.manifest_sha256)?;
        let origin = pending.interrupted_origin();
        let change = origin.as_ref().map(|origin| &origin.change).unwrap_or(&pending.change);
        let failure = pending.failure.as_ref();
        let current = (|| {
            let origin = origin.as_ref().map_err(Clone::clone)?;
            if pending.namespace_guards.is_some() {
                return Err("The complete source publication awaits retained entry confirmation. Refresh the workspace entry.".into());
            }
            let original = if origin.request_id == pending.request_id {
                active.clone()
            } else {
                exact_manifest(root, &origin.request_id, &origin.manifest_sha256)?
            };
            change.validate(&original.reviewed_plan)?;
            if active.before.keys().ne(original.before.keys()) || active.after != original.after {
                return Err("Interrupted attempt differs from the sealed Move namespace".into());
            }
            let saved = failure
                .ok_or("This interrupted transition has no retained producer checkpoints. Its saved namespace cannot be inferred.")?
                .current(root, &active)?;
            Ok(Namespace {
                original,
                saved,
                original_guard: origin.before_guard.clone(),
            })
        })();
        let notice = current.as_ref().err().cloned();
        let paths = active
            .before
            .keys()
            .map(
                |path| match current.as_ref().ok().and_then(|current| current.saved.get(path)) {
                    Some(SavedSource::File { sha256, .. }) => Ok(PathReading {
                        path: path.clone(),
                        saved_kind: "file".into(),
                        save_guard: Some(
                            failure
                                .ok_or("Interrupted source evidence disappeared")?
                                .guard(path)?
                                .ok_or("Interrupted file has no physical guard")?,
                        ),
                        source_hash: Some(format!("sha256:{sha256}")),
                    }),
                    Some(SavedSource::Missing) => Ok(PathReading {
                        path: path.clone(),
                        saved_kind: "missing".into(),
                        save_guard: None,
                        source_hash: None,
                    }),
                    None => Ok(PathReading {
                        path: path.clone(),
                        saved_kind: "unknown".into(),
                        save_guard: None,
                        source_hash: None,
                    }),
                },
            )
            .collect::<Result<Vec<_>, String>>()?;
        let effective_entry = [
            change.before.path.as_ref(),
            change.after.path.as_ref(),
        ]
        .into_iter()
        .flatten()
        .find(|path| {
            current
                .as_ref()
                .ok()
                .and_then(|current| current.saved.get(*path))
                .is_some_and(|saved| matches!(saved, SavedSource::File { .. }))
        })
        .cloned();
        let material = Material {
            version: 1,
            workspace,
            namespace,
            current_entry: bound.payload.current.clone(),
            request_id: origin.as_ref().map(|origin| &origin.request_id).unwrap_or(&pending.request_id).clone(),
            manifest_sha256: origin.as_ref().map(|origin| &origin.manifest_sha256).unwrap_or(&pending.manifest_sha256).clone(),
            change: change.clone(),
            namespace_evidence: serde_json::to_value(failure).map_err(|error| error.to_string())?,
            paths,
            effective_entry,
            reservation_floor: pending.reservation_floor(),
            attempts: pending.attempt_relations(root)?,
            can_complete: notice.is_none(),
            notice,
            source_files_modified: false,
        };
        let bytes = serde_json::to_vec(&material).map_err(|error| error.to_string())?;
        if bytes.len() > 2 * 1024 * 1024 {
            return Err("Interrupted reading exceeds its bounded metadata budget".into());
        }
        let fingerprint =
            crate::serve::private_record::seal(&bound.held.key, Some(DOMAIN), &bytes)?;
        bound.unchanged()?;
        Ok((Reading { material, fingerprint }, current.ok()))
    }
    pub(in crate::serve::workspace_entry_authority) fn validate_locked(
        state: &LocalState,
        reading: &Reading,
        bound: &storage::Bound,
    ) -> Result<Namespace, String> {
        crate::serve::private_record::verify(
            &bound.held.key,
            Some(DOMAIN),
            &serde_json::to_vec(&reading.material).map_err(|error| error.to_string())?,
            &reading.fingerprint,
        )?;
        let (fresh, namespace) = read_state(state, bound)?;
        if fresh.fingerprint != reading.fingerprint
            || serde_json::to_vec(&fresh.material).map_err(|error| error.to_string())?
                != serde_json::to_vec(&reading.material).map_err(|error| error.to_string())?
        {
            return Err("Interrupted entry reading changed after original capture. Preserve drafts and prepare a new complete review".into());
        }
        namespace.ok_or_else(|| fresh.material.notice.unwrap_or_else(|| {
            "Interrupted source producer remains unknown".into()
        }))
    }
    pub(super) fn inspect(state: LocalState) -> Result<Envelope, String> {
        super::super::ordinary(&state)?;
        leases::ordinary_root(state.index.root())?;
        let lease = leases::coordination(state.index.root(), false)?;
        let base = hickory_workspace::data_root().map_err(|error| error.to_string())?;
        let bound = storage::Bound::optional_under(&base, state.index.root())?;
        let workspace = state.resource_root.check(state.index.root())?.to_owned();
        let namespace = state
            .index
            .root()
            .canonicalize()
            .map_err(|error| error.to_string())?
            .to_str()
            .ok_or("Workspace namespace is not exact UTF-8")?
            .to_owned();
        let interrupted = match &bound {
            Some(bound) if bound.payload.pending.is_some() => Some(read_state(&state, bound)?.0),
            _ => None,
        };
        lease.check()?;
        Ok(Envelope {
            version: 1,
            workspace,
            namespace,
            interrupted,
            source_files_modified: false,
        })
    }
}
#[cfg(unix)]
pub(super) use unix::{Namespace, validate_locked};
pub(crate) async fn inspect(State(state): State<LocalState>) -> ApiResult<Json<Envelope>> {
    #[cfg(unix)]
    {
        state
            .resource_root
            .ready()
            .await
            .map_err(ApiError::conflict)?;
        tokio::task::spawn_blocking(move || unix::inspect(state))
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?
            .map(Json)
            .map_err(ApiError::conflict)
    }
    #[cfg(not(unix))]
    {
        let _ = state;
        Err(ApiError::unavailable(
            "This engine cannot retain interrupted entry authority",
        ))
    }
}
