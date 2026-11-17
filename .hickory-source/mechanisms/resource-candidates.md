<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Analyze construction ownership after a resource proposal

For maintainers connecting namespace proposals to source authority without coupling them to an editor host.

Reacquire the complete before state first, retaining the reviewed identities only for the pure ordered transition. Build two candidate layers: changed paths over saved construction input, and those same changes over supplied unsaved construction drafts. Explicit absence outranks pinned and disk fallbacks. Surviving entries follow their source identity to a new address; changed documents declaring projects introduce additional compiler roots. A new construction fragment must be consumed by at least one candidate layer rather than gaining a guessed standalone context.

The snapshot pins verified saved dependencies, bounds unique and cumulative source work, and checks every saved dependency again after analysis. The response reports each layer’s actual compiler output paths and any proposal targets that would become generated. Entries in the result state the exact compiler scope; this is not workspace-wide project discovery. A fingerprint binds the plan, root scope and these facts for later review; it is not a signed write receipt or publication permission. One optional analysis runs at a time on a blocking worker, independent of source opening and the filesystem write gate. No source file, generated output or index is modified, and this endpoint exposes no save operation. A later publisher must repeat admission checks, validate current UI drafts and handle per-path publication/recovery explicitly.

A restoration can return an older saved baseline while retaining a different
unsaved draft. The internal saved-layer seam accepts a complete, fingerprinted
saved namespace independently of the reviewed draft transition. File presence
must agree; saved text and drafts are compiled in their own layers and their
output claims remain a union. The projection enters the consequence fingerprint.
The public ordinary-save request has no override: a caller implementing recovery
must bind this internal projection to retained evidence before writing. Invalid
construction layers still refuse analysis; this change does not yet authorize
restoring an invalid historical draft or expose an undo route.

Focused evidence (2026-11-15): the separate saved/draft projection case passes
with distinct output inventories and hashes, their protected-target union, bound
consequence fingerprints, incomplete/contradictory/false-hash refusal and no
saved source changes. Consuming older saved text cannot validate a different
unconsumed draft at the same address: consumption must match the candidate text
hash. Seven existing candidate cases pass unchanged. This is internal analysis
for later restoration admission, not a working undo command.

Receipt adoption needs the exact reviewed saved states, including creation
permissions chosen by the engine's host. Return those states with the consequences
and bind them into its fingerprint. A browser or phone must not guess the server's
filesystem permissions. Ordinary saves derive them through the recovery manifest
constructor; restoration uses its independently validated saved projection.

Focused verification (2026-11-15): the new engine-metadata candidate witness
passed creation with host-selected private permissions, inherited move metadata,
exact hashes and absent materialization. The independent saved/draft projection
witness checks the returned complete saved map. All 47 selected resource CLI
cases passed. The rebuilt service witness confirmed equality between reviewed
saved states and the later complete publication receipt; this is server protocol
evidence, not frontend adoption acceptance.

A candidate can relocate a volume folder without producing the new file yet.
Protect targets covered by those exact saved/draft declarations as well as actual
compiler outputs. Keep the declared reservation out of `output_paths`: it is
ownership policy, not evidence that compilation produced a file. Recheck each
parsed source body and folder path through the same snapshot and path authority.

<ui:copy id="resource-candidate-ownership">
//! Evaluate the saved and draft construction layers of a proposed resource namespace.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    resource_changes::{ResourcePlan, SourceState},
    resource_inputs::{Inputs, Request as Capture},
};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub inputs: Inputs,
    pub capture: Capture,
    pub plan: ResourcePlan,
}
#[derive(Debug, Serialize)]
pub struct Layer {
    pub entries: Vec<String>,
    pub removed_entries: Vec<String>,
    pub source_hashes: BTreeMap<String, String>,
    pub output_paths: BTreeSet<String>,
}
#[derive(Debug, Serialize)]
pub struct Facts {
    pub version: u32,
    pub workspace: String,
    pub context: String,
    pub fingerprint: String,
    pub saved: Layer,
    pub draft: Layer,
    pub saved_sources: BTreeMap<String, super::resource_inputs::SavedSource>,
    pub checked_saved_sources: BTreeMap<String, String>,
    pub protected_targets: Vec<String>,
    pub source_files_modified: bool,
}
fn hash(text: &str) -> String {
    super::workspace_fs::view::hash(text.as_bytes())
}
struct Snapshot<'a> {
    root: &'a Path,
    pinned: BTreeMap<String, String>,
    checks: BTreeMap<String, String>,
    bytes: usize,
    cost: usize,
}
impl Snapshot<'_> {
    fn saved(&mut self, path: &str) -> Result<String, String> {
        if let Some(text) = self.pinned.get(path) {
            return Ok(text.clone());
        }
        let text = super::source_tests::saved_source(self.root, path)?;
        self.remember(path, &text)?;
        Ok(text)
    }
    fn remember(&mut self, path: &str, text: &str) -> Result<(), String> {
        if let Some(old) = self.pinned.get(path) {
            if old != text {
                return Err(
                    "Construction source changed during candidate analysis. Prepare again.".into(),
                );
            }
            return Ok(());
        }
        self.bytes = self.bytes.saturating_add(text.len());
        if self.bytes > 128 * 1024 * 1024 || self.pinned.len() >= 10_000 {
            return Err("Candidate construction source snapshot exceeds its interactive budget. Narrow the change.".into());
        }
        self.checks.insert(path.into(), hash(text));
        self.pinned.insert(path.into(), text.into());
        Ok(())
    }
}
fn construction(text: &str) -> Result<(bool, bool), String> {
    let doc=hick_lang::parse(text).map_err(|error|format!("Candidate construction source cannot be parsed: {error}. Correct it before resource changes."))?;
    Ok((
        !doc.find_tags("project").is_empty(),
        doc.all_tags().iter().any(|tag| {
            matches!(
                tag.name.as_str(),
                "file" | "entry" | "patch" | "scope" | "exec" | "volume"
            )
        }),
    ))
}
fn markdown(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    path.ends_with(".md") || path.ends_with(".markdown")
}
fn mapped_entries(inputs: &Inputs, after: &BTreeMap<String, SourceState>) -> BTreeSet<String> {
    inputs
        .ownership
        .protected_entries
        .iter()
        .filter_map(|entry| match after.get(entry) {
            Some(SourceState::Missing { .. }) => {
                let identity = match inputs.sources.get(entry) {
                    Some(SourceState::File { identity, .. }) => identity,
                    _ => return None,
                };
                after.iter().find_map(|(path, state)| match state {
                    SourceState::File {
                        identity: current, ..
                    } if current == identity => Some(path.clone()),
                    _ => None,
                })
            }
            _ => Some(entry.clone()),
        })
        .collect()
}
fn layer(
    snapshot: &mut Snapshot<'_>,
    inputs: &Inputs,
    after: &BTreeMap<String, SourceState>,
    overlays: &BTreeMap<String, String>,
    missing: &BTreeSet<String>,
    targets: &[String],
    claimed: &mut BTreeSet<String>,
) -> Result<Layer, String> {
    let mut roots = mapped_entries(inputs, after);
    for (path, text) in overlays {
        if markdown(path) && construction(text)?.0 {
            roots.insert(path.clone());
        }
    }
    let mut result = Layer {
        entries: vec![],
        removed_entries: vec![],
        source_hashes: BTreeMap::new(),
        output_paths: BTreeSet::new(),
    };
    for entry in roots {
        let source = if let Some(source) = overlays.get(&entry) {
            source.clone()
        } else {
            snapshot.saved(&entry)?
        };
        if !construction(&source)?.0 {
            result.removed_entries.push(entry);
            continue;
        }
        let scan = super::source_tests::scan_with_namespace(
            snapshot.root,
            &super::source_tests::Request {
                path: entry.clone(),
                source: source.clone(),
                project_entry: Some(entry.clone()),
                sources: overlays.clone(),
                capture: false,
            },
            Some(&snapshot.pinned),
            missing,
        )?;
        snapshot.cost = snapshot
            .cost
            .saturating_add(scan.source_texts.values().map(String::len).sum::<usize>());
        if snapshot.cost > 256 * 1024 * 1024 {
            return Err("Candidate construction analysis exceeds its cumulative source budget. Narrow the change.".into());
        }
        claimed.extend(super::resource_inputs::declared_volume_targets(snapshot.root, &scan.source_texts, targets)?);
        for (path, text) in &scan.source_texts {
            if !overlays.contains_key(path) {
                snapshot.remember(path, text)?;
            }
            if result
                .source_hashes
                .insert(path.clone(), hash(text))
                .is_some_and(|previous| previous != hash(text))
            {
                return Err(
                    "Candidate source layers disagreed about one source. Prepare again.".into(),
                );
            }
        }
        let tree = scan
            .tree
            .as_ref()
            .ok_or("Candidate construction did not produce a complete output tree.")?;
        let base = Path::new(&entry).parent().unwrap_or(Path::new(""));
        for path in tree.entries().keys() {
            result
                .output_paths
                .insert(super::source_tests::relative(base, path)?);
        }
        if result.output_paths.len() > 10_000 {
            return Err(
                "Candidate construction output inventory exceeds 10,000 paths. Narrow the change."
                    .into(),
            );
        }
        result.entries.push(entry);
    }
    for entry in &inputs.ownership.protected_entries {
        if missing.contains(entry) && !result.entries.contains(entry) {
            result.removed_entries.push(entry.clone());
        }
    }
    result.removed_entries.sort();
    result.removed_entries.dedup();
    Ok(result)
}
pub(crate) fn analyze(
    root: &Path,
    inputs: &Inputs,
    fresh: &Inputs,
    capture: &Capture,
    plan: &ResourcePlan,
) -> Result<Facts, String> {
    analyze_with_saved(root, inputs, fresh, capture, plan, None)
}
/// Internal restoration seam: the caller must bind saved states to retained recovery.
/// Ordinary resource saves always use the reviewed final draft as their saved state.
pub(crate) fn analyze_with_saved(
    root: &Path,
    inputs: &Inputs,
    fresh: &Inputs,
    capture: &Capture,
    plan: &ResourcePlan,
    saved_after: Option<&BTreeMap<String, super::resource_inputs::SavedSource>>,
) -> Result<Facts, String> {
    let after = super::resource_inputs::validate_candidate(inputs, fresh, plan)?;
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let mut snapshot = Snapshot {
        root: &root,
        pinned: BTreeMap::new(),
        checks: BTreeMap::new(),
        bytes: 0,
        cost: 0,
    };
    for (path, expected) in &fresh.ownership.saved_sources {
        let text = snapshot.saved(path)?;
        if hash(&text) != *expected {
            return Err(format!(
                "Saved construction input changed: {path}. Prepare again."
            ));
        }
    }
    let mut changed = BTreeMap::new();
    let missing: BTreeSet<_> = after
        .iter()
        .filter_map(|(path, state)| {
            matches!(state, SourceState::Missing { .. }).then_some(path.clone())
        })
        .collect();
    for patch in &plan.patches {
        if let SourceState::File { text, .. } = &patch.after {
            changed.insert(patch.path.clone(), text.clone());
        }
    }
    let saved_changed = if let Some(states) = saved_after {
        let paths: BTreeSet<_> = plan
            .patches
            .iter()
            .map(|patch| patch.path.clone())
            .collect();
        if states.keys().cloned().collect::<BTreeSet<_>>() != paths {
            return Err(
                "Restoration must retain the complete saved namespace separately from its drafts."
                    .into(),
            );
        }
        let mut overlays = BTreeMap::new();
        for (path, saved) in states {
            match (saved, after.get(path)) {
                (
                    super::resource_inputs::SavedSource::Missing,
                    Some(SourceState::Missing { .. }),
                ) => {}
                (
                    super::resource_inputs::SavedSource::File { text, sha256, .. },
                    Some(SourceState::File { .. }),
                ) => {
                    if text.len() > 8 * 1024 * 1024 || &hash(text) != sha256 {
                        return Err(
                            "Restored saved source has an invalid content fingerprint or size."
                                .into(),
                        );
                    }
                    overlays.insert(path.clone(), text.clone());
                }
                _ => {
                    return Err(
                        "Restored saved and draft namespaces disagree about source presence."
                            .into(),
                    );
                }
            }
        }
        overlays
    } else {
        changed.clone()
    };
    let mut saved = saved_changed.clone();
    let mut draft = capture.ownership_sources.clone();
    for (path, value) in &capture.drafts {
        draft.insert(path.clone(), value.text.clone());
    }
    draft.extend(changed.clone());
    for path in &missing {
        saved.remove(path);
        draft.remove(path);
    }
    let targets: Vec<_> = plan.patches.iter().map(|patch| patch.path.clone()).collect();
    let mut claimed = BTreeSet::new();
    let saved = layer(&mut snapshot, inputs, &after, &saved, &missing, &targets, &mut claimed)?;
    let draft = layer(&mut snapshot, inputs, &after, &draft, &missing, &targets, &mut claimed)?;
    for (path, text) in changed.iter().chain(saved_changed.iter()) {
        if markdown(path)
            && construction(text)?.1
            && saved.source_hashes.get(path) != Some(&hash(text))
            && draft.source_hashes.get(path) != Some(&hash(text))
        {
            return Err(format!(
                "Candidate construction source {path} has no selected project context. Link it to a construction entry in this proposal before saving resource changes."
            ));
        }
    }
    // Every saved dependency read by either candidate layer remains an admission check.
    super::source_tests::check_saved_sources(&root, &snapshot.checks)
        .map_err(|error| error.message().to_string())?;
    let protected_targets: Vec<_> = plan
        .patches
        .iter()
        .filter_map(|patch| {
            (saved.output_paths.contains(&patch.path) || draft.output_paths.contains(&patch.path) || claimed.contains(&patch.path))
                .then_some(patch.path.clone())
        })
        .collect();
    let mut facts = Facts {
        version: 1,
        workspace: fresh.workspace.clone(),
        context: fresh.context.clone(),
        fingerprint: String::new(),
        saved,
        draft,
        saved_sources: match saved_after {
            Some(states) => states.clone(),
            None => super::resource_journal::Manifest::from_review(inputs, plan)?.after,
        },
        checked_saved_sources: snapshot.checks,
        protected_targets,
        source_files_modified: false,
    };
    let bytes = if let Some(saved_after) = saved_after {
        serde_json::to_vec(&(plan, saved_after, &fresh.saved_guards, &facts))
    } else {
        serde_json::to_vec(&(plan, &fresh.saved_guards, &facts))
    }
    .map_err(|error| error.to_string())?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(
            "Candidate consequence response exceeds its interactive budget. Narrow the change."
                .into(),
        );
    }
    facts.fingerprint = super::workspace_fs::view::hash(&bytes);
    Ok(facts)
}
static ANALYSES: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);
pub async fn inspect(
    State(state): State<LocalState>,
    Json(request): Json<Request>,
) -> ApiResult<Json<Facts>> {
    let _permit=ANALYSES.try_acquire().map_err(|_|ApiError::conflict("Resource consequence analysis is busy. Let it finish before preparing another proposal."))?;
    let fresh =
        super::resource_inputs::inspect(State(state.clone()), Json(request.capture.clone()))
            .await?
            .0;
    let root = state.index.root().to_path_buf();
    let identity = state.resource_root.clone();
    let workspace = fresh.workspace.clone();
    let facts = tokio::task::spawn_blocking(move || {
        if identity.check(&root)? != workspace {
            return Err("Workspace changed. Prepare again.".into());
        }
        let facts = analyze(
            &root,
            &request.inputs,
            &fresh,
            &request.capture,
            &request.plan,
        )?;
        if identity.check(&root)? != workspace {
            return Err("Workspace changed during resource analysis. Prepare again.".into());
        }
        Ok::<_, String>(facts)
    })
    .await
    .map_err(|error| {
        ApiError::internal(format!(
            "Candidate analysis failed: {error}. Prepare again."
        ))
    })?
    .map_err(ApiError::conflict)?;
    Ok(Json(facts))
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/resource_candidates.rs"><ui:paste select="#resource-candidate-ownership"/></ui:file>
</ui:doc>
