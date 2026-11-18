<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Move the retained main through the shared resource publisher

For maintainers following an explicit main-construction address change from its original reading to durable source evidence. The intent is a live server object, not a capability flag or a deserialized historical row. The existing resource acquisition, compiler consequences, source publisher and editor receipts remain the path of execution. Focused real-engine and host acceptance must be recorded separately; constructors or pure models alone do not establish usable Move.

## Describe the original reading and exact operation

Keep request decoding strict. A Move names one vacant relative construction address; restoration names one retained record and fingerprint. Both use the original EntrySelection and a single engine-issued context. Resource URIs escape literal filenames rather than treating a percent or hash as metadata.

<ui:copy id="workspace-entry-transition-wire">
//! Explicit retained-main transitions use the existing resource review/publisher.
use super::{EntryChange, EntrySelection, EntryStamp, TransitionKind};
use crate::serve::{
    LocalState,
    api::{ApiError, ApiResult},
    resource_changes::{ResourcePlan, SourceState},
    resource_inputs::{Draft, Inputs, Request as Capture, SavedSource},
};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::{BTreeMap, BTreeSet}, path::Path};
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IntentRequest {
    version: u32,
    entry_selection: EntrySelection,
    operation: Operation,
}
#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Operation {
    Move {
        destination: String,
    },
    Restoration {
        recovery_id: String,
        recovery_fingerprint: String,
    },
    InterruptedCompletion,
}
#[derive(Clone, Serialize)]
pub(crate) struct IntentResponse {
    version: u32,
    workspace: String,
    namespace: String,
    token: String,
    context: String,
    kind: String,
    title: String,
    before_entry: EntryStamp,
    after_entry: EntryStamp,
    edit: Value,
    source_files_modified: bool,
    #[serde(flatten)]
    extra:Option<super::recovery::IntentExtra>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InputRequest {
    version: u32,
    token: String,
    capture: Capture,
}
fn hash(value: &impl Serialize) -> Result<String, String> {
    Ok(crate::serve::workspace_fs::view::hash(
        &serde_json::to_vec(value).map_err(|error| error.to_string())?,
    ))
}
fn uri(path: &str) -> Result<String, String> {
    let mut value =
        tower_lsp::lsp_types::Url::parse("hick:///").map_err(|error| error.to_string())?;
    {
        let mut segments = value
            .path_segments_mut()
            .map_err(|_| "Workspace URI cannot hold source segments")?;
        segments.clear();
        for segment in path.split('/') {
            segments.push(segment);
        }
    }
    Ok(value.to_string())
}
</ui:copy>

## Preserve original source material while acquisition expands

The bounded live catalogue retains immutable selection and parent identities. Initial source observations may gain newly discovered context, but already captured text, versions, writable state and physical guards cannot drift. Signed context names remain recognizable after cache expiry or restart only so the engine can refuse them: recognizing a historical name never recreates a live intent. Source reads remain available when this capability refuses.

An inverse intent binds the retained producer to the current entry. If an
exact newer inverse already canceled its descendants, retain the sealed
producer-chain proof with that intent so Preview and Save use the same frame.
Creation and other immediate role inverses keep their existing strict
current-producer validation.

<ui:copy id="workspace-entry-transition-intent">
#[cfg(unix)]
mod unix {
    use super::super::{Binding, leases, storage};
    use super::*;
    use std::{
        sync::{Arc, LazyLock, Mutex},
        time::{Duration, Instant},
    };
    const MAX_INTENTS: usize = 32;
    const MAX_KEYS: usize = 16384;
    const LIFETIME: Duration = Duration::from_secs(15 * 60);
    #[derive(Clone)]
    struct Parent {
        path: String,
        identity: String,
    }
    #[derive(Clone, Default)]
    struct Originals {
        drafts: BTreeMap<String, String>,
        ownership: BTreeMap<String, String>,
        unaddressed: BTreeMap<String, String>,
        guards: BTreeMap<String, String>,
    }
    impl Originals {
        fn extend_map<T: Serialize>(
            original: &mut BTreeMap<String, String>,
            current: &BTreeMap<String, T>,
        ) -> Result<(), String> {
            for (key, expected) in original.iter() {
                if current.get(key).map(hash).transpose()?.as_ref() != Some(expected) {
                    return Err(format!(
                        "Original retained source material changed or disappeared: {key}. Prepare a new entry review."
                    ));
                }
            }
            for (key, value) in current {
                original.insert(key.clone(), hash(value)?);
            }
            Ok(())
        }
        fn extend(&mut self, capture: &Capture) -> Result<(), String> {
            Self::extend_map(&mut self.drafts, &capture.drafts)?;
            Self::extend_map(&mut self.ownership, &capture.ownership_sources)?;
            Self::extend_map(&mut self.unaddressed, &capture.unaddressed_sources)?;
            Self::extend_map(&mut self.guards, &capture.saved_guards)?;
            if self.drafts.len() + self.ownership.len() + self.unaddressed.len() + self.guards.len()
                > MAX_KEYS
            {
                return Err("Entry review exceeds its retained source identity budget.".into());
            }
            Ok(())
        }
    }
    #[derive(Clone)]
    enum Admission {
        Selected(EntrySelection),
        Interrupted {
            reading: Arc<super::super::recovery::Reading>,
            retained_paths: BTreeSet<String>,
        },
    }
    impl Admission {
        fn before(&self) -> &EntryStamp {
            match self {
                Self::Selected(proof) => &proof.entry,
                Self::Interrupted { reading, .. } => &reading.material.current_entry,
            }
        }
        fn compiler_entry(&self) -> Option<&String> {
            match self {
                Self::Selected(proof) => proof.entry.path.as_ref(),
                Self::Interrupted { reading, .. } => reading.material.effective_entry.as_ref(),
            }
        }
        fn original_guard(&self) -> Option<String> {
            match self {
                Self::Selected(proof) => proof.source_guard.clone(),
                Self::Interrupted { reading, .. } => reading.material.paths.iter()
                    .find(|path| Some(&path.path) == reading.material.current_entry.path.as_ref())
                    .and_then(|path| path.save_guard.clone()),
            }
        }
        fn interrupted(&self) -> Option<&super::super::recovery::Reading> {
            match self {
                Self::Interrupted { reading, .. } => Some(reading),
                Self::Selected(_) => None,
            }
        }
    }
    #[derive(Clone)]
    struct LiveIntent {
        response: IntentResponse,
        admission: Admission,
        parents: Vec<Parent>,
        operation: Operation,
        entry_lineage: Option<Arc<super::super::EntryLineageProof>>,
        created: Instant,
        originals: Originals,
        plan: Option<String>,
        saved: Option<String>,
        sources:Option<String>,
    }
    static INTENTS: LazyLock<Mutex<BTreeMap<String, Arc<Mutex<LiveIntent>>>>> =
        LazyLock::new(|| Mutex::new(BTreeMap::new()));
    const INTENT_DOMAIN: &[u8] = b"hickory.workspace-entry.live-intent-name.v1";
    fn intent_mac(key: &[u8], nonce: &[u8]) -> Result<hmac::Hmac<sha2::Sha256>, String> {
        use hmac::Mac;
        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(key)
            .map_err(|_| "Entry intent seal unavailable")?;
        mac.update(&(INTENT_DOMAIN.len() as u64).to_be_bytes());
        mac.update(INTENT_DOMAIN);
        mac.update(nonce);
        Ok(mac)
    }
    fn token(key: &[u8]) -> Result<String, String> {
        use hmac::Mac;
        let mut nonce = [0u8; 16];
        getrandom::fill(&mut nonce).map_err(|error| error.to_string())?;
        let tag = intent_mac(key, &nonce)?.finalize().into_bytes();
        Ok(format!("{}{}", hex::encode(nonce), hex::encode(&tag[..16])))
    }
    fn reserved_context(key: &[u8], context: &str) -> Result<bool, String> {
        use hmac::Mac;
        if context.len() != 64 {
            return Ok(false);
        }
        let Ok(bytes) = hex::decode(context) else {
            return Ok(false);
        };
        Ok(intent_mac(key, &bytes[..16])?
            .verify_truncated_left(&bytes[16..])
            .is_ok())
    }
    fn obtain(token: &str) -> Result<Arc<Mutex<LiveIntent>>, String> {
        if !crate::serve::resource_saves::hex(token, 64) {
            return Err("Entry review token is invalid.".into());
        }
        let mut values = INTENTS
            .lock()
            .map_err(|_| "Entry reviews are unavailable")?;
        values.retain(|_, value| {
            value
                .lock()
                .map(|value| value.created.elapsed() < LIFETIME)
                .unwrap_or(false)
        });
        values.get(token).cloned().ok_or("This entry review expired or belongs to another engine. Prepare again; no historical token was applied.".into())
    }
    fn retain(value: LiveIntent) -> Result<(), String> {
        let mut values = INTENTS
            .lock()
            .map_err(|_| "Entry reviews are unavailable")?;
        values.retain(|_, value| {
            value
                .lock()
                .map(|value| value.created.elapsed() < LIFETIME)
                .unwrap_or(false)
        });
        if values.len() >= MAX_INTENTS {
            let oldest = values
                .iter()
                .filter_map(|(key, value)| {
                    value.lock().ok().map(|value| (key.clone(), value.created))
                })
                .min_by_key(|(_, created)| *created)
                .map(|(key, _)| key)
                .ok_or("Entry review retention is unavailable")?;
            values.remove(&oldest);
        }
        values.insert(value.response.token.clone(), Arc::new(Mutex::new(value)));
        Ok(())
    }
    fn parents(
        root: &Path,
        paths: impl IntoIterator<Item = String>,
    ) -> Result<Vec<Parent>, String> {
        let mut result = BTreeMap::new();
        for path in paths {
            crate::serve::resource_changes::validate_path(&path)?;
            let directory = path
                .rsplit_once('/')
                .map(|(directory, _)| directory)
                .unwrap_or("");
            let identity =
                crate::serve::resource_inputs::RootIdentity::fingerprint(&root.join(directory))?;
            result.insert(directory.to_owned(), identity);
        }
        Ok(result
            .into_iter()
            .map(|(path, identity)| Parent { path, identity })
            .collect())
    }
    fn check(state: &LocalState, value: &LiveIntent, bound: &storage::Bound) -> Result<(), String> {
        if value.created.elapsed() >= LIFETIME {
            return Err(
                "Entry review expired. Prepare the complete move or restoration again.".into(),
            );
        }
        match &value.admission {
            Admission::Selected(proof)=>super::super::validate_selection_snapshot(state.index.root(),&state.resource_root,proof,bound)?,
            Admission::Interrupted { reading, .. } => { super::super::recovery::validate_locked(state, reading, bound)?; },
        }
        check_parents(state.index.root(), &value.parents)
    }
    fn inverse_change(
        root: &Path,
        bound: &storage::Bound,
        record: &crate::serve::resource_journal::Recovery,
        entry_lineage: Option<&super::super::EntryLineageProof>,
    ) -> Result<EntryChange, String> {
        if let Some(proof) = entry_lineage {
            return super::super::history_transition_with_entry_proof(
                root, &bound.payload, record, proof,
            );
        }
        check_retained_namespace(root, &bound.payload, record)?;
        let proof = crate::serve::source_history::prove_resource_after(root, record)?;
        super::super::history_transition_after_lineage(root, &bound.payload, record, &proof)
    }
    fn checked_inverse(
        root: &Path,
        bound: &storage::Bound,
        intent: &LiveIntent,
        record: &crate::serve::resource_journal::Recovery,
    ) -> Result<EntryChange, String> {
        let Operation::Restoration { recovery_id, recovery_fingerprint } = &intent.operation else {
            return Err("Choose a reviewed entry restoration before preparing its inverse.".into());
        };
        if &record.id != recovery_id || &record.fingerprint != recovery_fingerprint {
            return Err("Entry restoration changed its original sealed recovery record. Prepare a fresh review.".into());
        }
        let change = inverse_change(root, bound, record, intent.entry_lineage.as_deref())?;
        if change.after != intent.response.after_entry {
            return Err("Entry restoration no longer matches its original selected generation. Prepare a fresh review.".into());
        }
        Ok(change)
    }
    fn interrupted_record(binding: &Binding, id: &str) -> bool {
        let recorded = |recovery: &super::super::RecoveryState| {
            recovery.origin.request_id == id || recovery.attempts.iter().any(|attempt| attempt.request_id == id)
        };
        binding.pending.as_ref().is_some_and(|pending| {
            pending.request_id == id || pending.recovery.as_ref().is_some_and(recorded)
        }) || binding.history.iter().any(|row| {
            (row.request_id == id && row.change.kind == TransitionKind::InterruptedCompletion)
                || row.recovered_from.as_ref().is_some_and(recorded)
        })
    }

    pub(crate) fn validate_restoration_origin(
        state: &LocalState,
        capture: &Capture,
        record: &crate::serve::resource_journal::Recovery,
    ) -> Result<(), String> {
        let root = state.index.root();
        let base = hickory_workspace::data_root().map_err(|error| error.to_string())?;
        if let Some(bound) = storage::Bound::optional_under(&base, root)? {
            if interrupted_record(&bound.payload, &record.id) {
                return Err("Interrupted Move records require their own complete recovery review. These retained snapshots grant no generic inverse authority.".into());
            }
            if reserved_context(&bound.held.key, &capture.context)? {
                let live = obtain(&capture.context)?;
                let intent = live.lock().map_err(|_| "Entry review is unavailable")?.clone();
                exact_capture(&intent, capture)?;
                check(state, &intent, &bound)?;
                checked_inverse(root, &bound, &intent, record)?;
                return bound.unchanged();
            }
            if bound.payload.history.iter().any(|row| row.request_id == record.id) {
                return Err("Prepare a fresh workspace-entry review to restore this retained transition.".into());
            }
        }
        crate::serve::source_history::validate_resource_after(root, record)
    }
    pub(super) fn intent(
        state: LocalState,
        request: IntentRequest,
    ) -> Result<IntentResponse, String> {
        if request.version != 1 {
            return Err("Workspace-entry transitions require protocol 1.".into());
        }
        super::super::ordinary(&state)?;
        let read = super::super::acquire_entry_read_lease(
            state.index.root(),
            request
                .entry_selection
                .entry
                .path
                .as_ref()
                .map(|_| &request.entry_selection.entry),
        )?;
        let bound = storage::Bound::open(state.index.root(), false)?;
        super::super::validate_selection_snapshot(
            state.index.root(),
            &state.resource_root,
            &request.entry_selection,
            &bound,
        )?;
        if bound.payload.history.len() >= 256 {
            return Err("The bounded main-transition journal has reached 256 records. Source and retained evidence remain readable; no prior identity was discarded.".into());
        }
        let before = request.entry_selection.entry.clone();
        let (root, workspace, namespace) = (
            state.index.root(),
            request.entry_selection.workspace.clone(),
            request.entry_selection.namespace.clone(),
        );
        let mut entry_lineage = None;
        let (kind, title, after, edit, affected) = match &request.operation {
            Operation::Move { destination } => {
                let path = before.path.as_ref().ok_or(
                    "No retained main construction is configured; there is no entry to move.",
                )?;
                if !request.entry_selection.movable {
                    return Err("The retained main is incomplete or read-only. Repair its authored source before moving its role.".into());
                }
                crate::serve::resource_changes::validate_path(destination)?;
                hick_lang::validate_project_path(destination).map_err(|error| {
                    format!("Shared compiler rejected this construction address: {error}")
                })?;
                if destination == path {
                    return Err("Choose a different main construction address.".into());
                }
                let (_, saved) = crate::serve::resource_inputs::saved_target(root, destination)?;
                if !matches!(saved, crate::serve::resource_inputs::SavedSource::Missing) {
                    return Err("Main construction destination already exists. Choose a vacant exact source path; no existing source was adopted.".into());
                }
                let after = before.advance(Some(destination.clone()))?;
                (
                    "move".to_owned(),
                    format!("Move main construction to {destination}"),
                    after,
                    json!({"documentChanges":[{"kind":"rename","oldUri":uri(path)?,"newUri":uri(destination)?}]}),
                    vec![path.clone(), destination.clone()],
                )
            }
            Operation::Restoration {
                recovery_id,
                recovery_fingerprint,
            } => {
                if !crate::serve::resource_saves::hex(recovery_id, 32)
                    || !crate::serve::resource_saves::hex(recovery_fingerprint, 64)
                {
                    return Err("Choose an exact retained entry recovery identity.".into());
                }
                let record = crate::serve::resource_journal::read_recovery(root, recovery_id)?;
                if record.fingerprint != *recovery_fingerprint {
                    return Err("Entry recovery evidence changed. Read it again; no old fingerprint was applied.".into());
                }
                let row = retained_transition(&bound.payload, &record)?;
                if row.change.after != bound.payload.current {
                    entry_lineage = Some(Arc::new(super::super::lineage::prove(
                        root, &bound.payload, &record,
                    )?));
                }
                let change = inverse_change(root, &bound, &record, entry_lineage.as_deref())?;
                let affected: Vec<_> = record.manifest.after.keys().cloned().collect();
                let changes = affected
                    .iter()
                    .map(|path| Ok((uri(path)?, json!([]))))
                    .collect::<Result<serde_json::Map<String, Value>, String>>()?;
                (
                    "restoration".to_owned(),
                    format!("Restore retained main construction transition {recovery_id}"),
                    change.after,
                    json!({"changes":changes}),
                    affected,
                )
            }
            Operation::InterruptedCompletion => {
                return Err("Prepare interrupted recovery from its sealed Source Recovery reading.".into());
            }
        };
        let parents = parents(root, affected)?;
        let token = token(&bound.held.key)?;
        let response = IntentResponse {
            version: 1,
            workspace,
            namespace,
            token: token.clone(),
            context: token,
            kind,
            title,
            before_entry: before,
            after_entry: after,
            edit,
            source_files_modified: false,
            extra: None,
        };
        let value = LiveIntent {
            response: response.clone(),
            admission: Admission::Selected(request.entry_selection),
            parents,
            operation: request.operation,
            entry_lineage,
            created: Instant::now(),
            originals: Originals::default(),
            plan: None,
            saved: None,
            sources:None,
        };
        check(&state, &value, &bound)?;
        read.check()?;
        retain(value)?;
        Ok(response)
    }
    fn exact_capture(intent: &LiveIntent, capture: &Capture) -> Result<(), String> {
        if capture.context != intent.response.context
            || capture.edit != intent.response.edit
            || capture.project_entry.as_ref().is_some_and(|entry| Some(entry) != intent.admission.compiler_entry())
        {
            return Err("Entry acquisition changed its original operation or construction context. Prepare a new review.".into());
        }
        intent.originals.clone().extend(capture)
    }

    // The interrupted reading grants one typed acquisition scope. Ordinary
    // resource inputs cannot sign their own sealed orphan alias. New source
    // acquisitions may add clean buffers without taking ownership of an
    // earlier, independently retained dirty draft.
    /// A scope can only be constructed beside a live, sealed interruption.
    /// Holding it keeps the exact root and binding readable through acquisition.
    pub(crate) struct AcquisitionScope {
        bound: storage::Bound,
        lease: leases::Coordination,
        intent: LiveIntent,
        namespace: super::super::recovery::Namespace,
        _ordinary: crate::engine::private_policy::Ordinary,
    }

    impl AcquisitionScope {
        pub(crate) fn canonical_entry(&self) -> &str {
            self.intent.admission.compiler_entry().expect("sealed surviving entry")
        }

        fn original_path(&self) -> &str {
            self.intent.response.before_entry.path.as_deref().expect("sealed original path")
        }

        pub(crate) fn check(&self, root: &Path, request: &Capture) -> Result<(), String> {
            exact_capture(&self.intent, request)?;
            self.lease.check()?;
            self.bound.unchanged()?;
            let Admission::Interrupted { reading, retained_paths } = &self.intent.admission else {
                return Err("No original interrupted entry scope".into());
            };
            for (path, draft) in &request.drafts {
                if request.ownership_sources.get(path) != Some(&draft.text)
                    || (!retained_paths.contains(path)
                        && !matches!(self.namespace.saved.get(path), Some(SavedSource::File { text, .. }) if &draft.text == text))
                {
                    return Err(format!("Newly acquired or inconsistent draft cannot replace an original interrupted source: {path}."));
                }
            }
            for observation in &reading.material.paths {
                let path = &observation.path;
                let (full, current) = crate::serve::resource_inputs::saved_target(root, path)?;
                if self.namespace.saved.get(path) != Some(&current) {
                    return Err(format!("Interrupted saved source changed during acquisition: {path}"));
                }
                if let Some(guard) = &observation.save_guard {
                    crate::serve::source_history::validate_observed_guard(root, &full, path, guard)?;
                }
            }
            check_parents(root, &self.intent.parents)
        }

        pub(crate) fn compiler_request(&self, request: &Capture) -> Capture {
            let mut compiler = request.clone();
            if matches!(self.namespace.saved[self.original_path()], SavedSource::Missing) {
                // The compiler first reads the surviving saved entry. Candidate
                // analysis places the retained orphan draft at the reviewed destination.
                compiler.ownership_sources.remove(self.original_path());
            }
            compiler
        }

        pub(crate) fn retained_generated_targets(&self, paths: &[String]) -> Result<BTreeSet<String>, String> {
            self.bound.payload.ownership.as_ref().map_or_else(
                || Ok(BTreeSet::new()),
                |inventory| inventory.protected_targets(paths),
            )
        }

        pub(crate) fn orphan_draft(&self, path: &str, request: &Capture, saved: &SavedSource) -> Result<Option<Draft>, String> {
            let Some(draft) = request.drafts.get(path) else { return Ok(None); };
            if self.permits_orphan_path(path)
                && matches!(saved, SavedSource::Missing)
                && draft.save_guard.is_some()
                && draft.save_guard == self.namespace.original_guard
                && matches!(self.namespace.original.before.get(path), Some(SavedSource::File { sha256, .. }) if sha256 == &draft.base_sha256)
                && !request.saved_guards.contains_key(path)
                && request.ownership_sources.get(path) == Some(&draft.text)
            {
                return Ok(Some(draft.clone()));
            }
            Err(format!("Missing source has no matching retained original editor and sealed saved baseline: {path}. Preserve its draft."))
        }

        fn permits_orphan_path(&self, path: &str) -> bool {
            let Admission::Interrupted { retained_paths, .. } = &self.intent.admission else { return false; };
            path == self.original_path()
                && retained_paths.contains(path)
                && matches!(self.namespace.saved.get(path), Some(SavedSource::Missing))
        }

        pub(crate) fn permits_retained_source(&self, path: &str, source: &SourceState, text: &str) -> bool {
            self.namespace.saved.contains_key(path)
                && matches!(source, SourceState::File { text: captured, .. } if captured == text)
        }

        pub(crate) fn validate_orphan_sources(&self, sources: &BTreeMap<String, SourceState>) -> Result<(), String> {
            if sources.keys().ne(self.namespace.saved.keys())
                || sources.iter().any(|(path, source)| matches!(source, SourceState::File { .. })
                    && matches!(self.namespace.saved[path], SavedSource::Missing)
                    && !self.permits_orphan_path(path))
            {
                return Err("Interrupted capture exceeds its exact original Move source scope.".into());
            }
            Ok(())
        }
    }

    pub(crate) fn capture_scope(state: &LocalState, request: &Capture) -> Result<Option<AcquisitionScope>, String> {
        let Ok(live) = obtain(&request.context) else { return Ok(None); };
        let intent = live.lock().map_err(|_| "Entry review is unavailable")?.clone();
        if intent.admission.interrupted().is_none() {
            return Ok(None);
        }
        let ordinary = crate::engine::private_policy::ordinary();
        super::super::ordinary(state)?;
        let root = state.index.root();
        leases::ordinary_root(root)?;
        let lease = leases::coordination(root, false)?;
        let bound = storage::Bound::open(root, false)?;
        let namespace = interrupted_namespace(state, &intent, &bound)?;
        let scope = AcquisitionScope { bound, lease, intent, namespace, _ordinary: ordinary };
        scope.check(root, request)?;
        Ok(Some(scope))
    }

    fn check_parents(root: &Path, parents: &[Parent]) -> Result<(), String> {
        for parent in parents {
            if crate::serve::resource_inputs::RootIdentity::fingerprint(&root.join(&parent.path))? != parent.identity {
                return Err("An original entry source/destination folder was replaced. No new parent was selected.".into());
            }
        }
        Ok(())
    }

    fn interrupted_namespace(
        state: &LocalState,
        intent: &LiveIntent,
        bound: &storage::Bound,
    ) -> Result<super::super::recovery::Namespace, String> {
        if intent.created.elapsed() >= LIFETIME {
            return Err("Original interrupted entry review expired. Prepare a fresh review.".into());
        }
        let reading = intent.admission.interrupted().ok_or("No sealed interrupted entry reading")?;
        let namespace = super::super::recovery::validate_locked(state, reading, bound)?;
        check_parents(state.index.root(), &intent.parents)?;
        Ok(namespace)
    }

    fn completion_plan(
        intent: &LiveIntent,
        namespace: &super::super::recovery::Namespace,
        sources: &BTreeMap<String, SourceState>,
    ) -> Result<super::super::completion::Completion, String> {
        let Admission::Interrupted { retained_paths, .. } = &intent.admission else {
            return Err("A live interrupted entry review is required".into());
        };
        super::super::completion::prepare(
            &namespace.original,
            intent.response.before_entry.path.as_deref().ok_or("Interrupted Move lost its source")?,
            intent.response.after_entry.path.as_deref().ok_or("Interrupted Move lost its destination")?,
            &namespace.saved,
            sources,
            retained_paths,
            &intent.response.context,
        )
    }

    pub(super) fn interrupted_intent(
        state: LocalState,
        request: super::super::recovery::IntentRequest,
    ) -> Result<IntentResponse, String> {
        if request.version != 1 {
            return Err("Interrupted completion requires protocol 1.".into());
        }
        super::super::ordinary(&state)?;
        let root = state.index.root();
        leases::ordinary_root(root)?;
        let lease = leases::coordination(root, false)?;
        let bound = storage::Bound::open(root, false)?;
        let namespace = super::super::recovery::validate_locked(&state, &request.reading, &bound)?;
        let pending = bound.payload.pending.as_ref().ok_or("No interrupted Move remains")?;
        let material = &request.reading.material;
        let before = material.current_entry.clone();
        let after = EntryStamp {
            generation: material.reservation_floor.checked_add(1).ok_or("Interrupted recovery generation capacity exhausted")?,
            path: material.change.after.path.clone(),
            ..before.clone()
        };
        if bound.payload.history.len() >= 256 || material.attempts.len() >= 256 {
            return Err("The retained entry journal is full. Preserve the original recovery evidence.".into());
        }
        let supersedes = request.supersede_request.as_ref().map(|selector| {
            material.attempts.iter().find(|attempt| {
                attempt.request_id == pending.request_id
                    && attempt.request_id == selector.request_id
                    && attempt.context == selector.context
            }).cloned().ok_or("Superseding recovery must identify the exact retained current attempt.")
        }).transpose()?;
        let paths: Vec<_> = namespace.saved.keys().cloned().collect();
        let parents = parents(root, paths.clone())?;
        let edit = json!({"changes": paths.iter().map(|path| Ok((uri(path)?, json!([]))))
            .collect::<Result<serde_json::Map<String, Value>, String>>()?});
        let token = token(&bound.held.key)?;
        let response = IntentResponse {
            version: 1,
            workspace: material.workspace.clone(),
            namespace: material.namespace.clone(),
            token: token.clone(),
            context: token,
            kind: "interrupted_completion".into(),
            title: super::super::completion::TITLE.into(),
            before_entry: before,
            after_entry: after,
            edit,
            source_files_modified: false,
            extra: Some(super::super::recovery::IntentExtra {
                interrupted_reading_fingerprint: request.reading.fingerprint.clone(),
                supersedes,
            }),
        };
        let retained = request.retained_capture;
        let retained_paths: BTreeSet<_> = retained.drafts.keys().cloned().collect();
        if !retained_paths.iter().all(|path| namespace.saved.contains_key(path)) {
            return Err("An original retained draft is outside the interrupted Move scope.".into());
        }
        let mut originals = Originals::default();
        originals.extend(&Capture {
            context: response.context.clone(),
            edit: response.edit.clone(),
            project_entry: None,
            drafts: retained.drafts,
            ownership_sources: retained.ownership_sources,
            unaddressed_sources: retained.unaddressed_sources,
            saved_guards: retained.saved_guards,
        })?;
        let value = LiveIntent {
            response: response.clone(),
            admission: Admission::Interrupted { reading: Arc::new(request.reading), retained_paths },
            parents,
            operation: Operation::InterruptedCompletion,
            created: Instant::now(),
            originals,
            plan: None,
            saved: None,
            sources: None,
            entry_lineage: None,
        };
        check(&state, &value, &bound)?;
        lease.check()?;
        retain(value)?;
        Ok(response)
    }

    pub(super) async fn inputs(state: LocalState, request: InputRequest) -> Result<Inputs, String> {
        super::super::ordinary(&state)?;
        if request.version != 1 {
            return Err("Workspace-entry source acquisition requires protocol 1.".into());
        }
        let live = obtain(&request.token)?;
        let original = live
            .lock()
            .map_err(|_| "Entry review is unavailable")?
            .clone();
        exact_capture(&original, &request.capture)?;
        let reading = if original.admission.interrupted().is_none() {
            let reading = super::super::acquire_entry_read_lease(
                state.index.root(),
                original.admission.before().path.as_ref().map(|_| original.admission.before()),
            )?;
            let bound = storage::Bound::open(state.index.root(), false)?;
            check(&state, &original, &bound)?;
            Some(reading)
        } else {
            None
        };
        let result = crate::serve::resource_inputs::inspect(
            State(state.clone()),
            Json(request.capture.clone()),
        )
        .await
        .map_err(|error| error.detail().to_string())?
        .0;
        let bound = storage::Bound::open(state.index.root(), false)?;
        check(&state, &original, &bound)?;
        if let Some(reading) = reading {
            reading.check()?;
        }
        let plan = if result.refusals.is_empty() {
            match &original.operation {
                Operation::Move { .. } => Some(crate::serve::resource_changes::prepare(
                    crate::serve::resource_changes::Request {
                        context: result.context.clone(),
                        title: original.response.title.clone(),
                        edit: original.response.edit.clone(),
                        sources: result.sources.clone(),
                    },
                )?),
                Operation::InterruptedCompletion => {
                    let namespace = interrupted_namespace(&state, &original, &bound)?;
                    Some(completion_plan(&original, &namespace, &result.sources)?.plan)
                }
                Operation::Restoration { .. } => None,
            }
        } else {
            None
        };
        let mut value = live.lock().map_err(|_| "Entry review is unavailable")?;
        // Another acquisition cannot erase already frozen source material.
        let mut coherent = value.originals.clone();
        coherent.extend(&request.capture)?;
        value.originals = coherent;
        if let Some(plan) = plan {
            value.saved = Some(hash(&(
                &result.saved_sources,
                &result.saved_guards,
                &result.workspace,
            ))?);
            value.plan = Some(hash(&plan)?);
            value.sources = Some(hash(&result.sources)?);
        }
        Ok(result)
    }

    pub(crate) fn plan_interrupted(
        state: &LocalState,
        request: &crate::serve::resource_changes::Request,
    ) -> Result<Option<ResourcePlan>, String> {
        let Ok(live) = obtain(&request.context) else { return Ok(None); };
        let intent = live.lock().map_err(|_| "Entry review is unavailable")?.clone();
        if intent.admission.interrupted().is_none() { return Ok(None); }
        super::super::ordinary(state)?;
        let root = state.index.root();
        leases::ordinary_root(root)?;
        let lease = leases::coordination(root, false)?;
        let bound = storage::Bound::open(root, false)?;
        let namespace = interrupted_namespace(state, &intent, &bound)?;
        if request.edit != intent.response.edit || request.title != intent.response.title
            || intent.sources.as_deref() != Some(hash(&request.sources)?.as_str())
        {
            return Err("Interrupted plan changed its captured operation or logical source. Prepare a fresh review.".into());
        }
        let plan = completion_plan(&intent, &namespace, &request.sources)?.plan;
        if intent.plan.as_deref() != Some(hash(&plan)?.as_str()) {
            return Err("Interrupted plan differs from its exact acquired source review.".into());
        }
        lease.check()?;
        Ok(Some(plan))
    }
</ui:copy>

## Persist pending authority before the first source byte changes

The shared publisher owns the exclusive physical-root provider. Retiring the known main requires the live engine intent and exact prepared plan. Before replacing an interrupted reservation, authenticate its original Move and latest producer checkpoints under that same lease. Keep the old role while the shared writer applies the reviewed saved namespace. A completion reserves the next generation beyond every retained attempt. Only complete producer observations confirm it. The finite record budget refuses before mutation and preserves the earlier identities needed to read history.

Before the first writer, the shared publisher records physical source history.
An older inverse lends its sealed frame to that preparer under this same
exclusive publication lease. The borrow ends before any file callback can run.

<ui:copy id="workspace-entry-transition-publisher">
    /// A live Move keeps saved bytes separate from its retained dirty editor.
    /// Fields remain private so ordinary resource callers cannot mint this proof.
    pub(crate) struct SavedMoveProjection {
        states: BTreeMap<String, SavedSource>,
        interrupted: bool,
    }
    impl SavedMoveProjection {
        pub(crate) fn states(
            &self,
        ) -> &BTreeMap<String, crate::serve::resource_inputs::SavedSource> {
            &self.states
        }
        pub(crate) fn manifest(self, inputs: &Inputs, plan: &ResourcePlan) -> Result<crate::serve::resource_journal::Manifest, String> {
            use crate::serve::resource_journal::Manifest;
            if self.interrupted {
                return Manifest::from_interrupted_review(inputs, plan, &self.states);
            }
            let mut manifest = Manifest::from_review(inputs, plan)?;
            manifest.after = self.states;
            Ok(manifest)
        }
    }
    pub(crate) fn saved_move_projection(
        state: &LocalState,
        inputs: &crate::serve::resource_inputs::Inputs,
        plan: &crate::serve::resource_changes::ResourcePlan,
    ) -> Result<Option<SavedMoveProjection>, String> {
        let base = hickory_workspace::data_root().map_err(|error| error.to_string())?;
        let Some(bound) = storage::Bound::optional_under(&base, state.index.root())? else {
            return Ok(None);
        };
        let live = match obtain(&inputs.context) {
            Ok(live) => live,
            Err(error) if reserved_context(&bound.held.key, &inputs.context)? => return Err(error),
            Err(_) => return Ok(None),
        };
        let intent = live
            .lock()
            .map_err(|_| "Entry review is unavailable")?
            .clone();
        check(state, &intent, &bound)?;
        if !matches!(intent.operation, Operation::Move { .. } | Operation::InterruptedCompletion) {
            return Ok(None);
        }
        if intent.plan.as_deref() != Some(hash(plan)?.as_str())
            || intent.saved.as_deref()
                != Some(
                    hash(&(
                        &inputs.saved_sources,
                        &inputs.saved_guards,
                        &inputs.workspace,
                    ))?
                    .as_str(),
                )
        {
            return Err(
                "Saved Move baseline differs from the original acquired review. Prepare again."
                    .into(),
            );
        }
        if intent.admission.interrupted().is_some() {
            let namespace = interrupted_namespace(state, &intent, &bound)?;
            let expected = completion_plan(&intent, &namespace, &inputs.sources)?;
            if expected.saved_before != inputs.saved_sources
                || hash(&expected.plan)? != hash(plan)?
                || intent.sources.as_deref() != Some(hash(&inputs.sources)?.as_str())
            {
                return Err("Completion changed its reviewed logical draft or saved namespace.".into());
            }
            return Ok(Some(SavedMoveProjection { states: expected.saved_after, interrupted: true }));
        }
        let after = plan.apply(&inputs.context, &inputs.sources)?;
        let mut states = BTreeMap::new();
        for patch in &plan.patches {
            let saved = match &after[&patch.path] {
                crate::serve::resource_changes::SourceState::Missing { .. } => {
                    crate::serve::resource_inputs::SavedSource::Missing
                }
                crate::serve::resource_changes::SourceState::File { identity, .. } => {
                    let original = inputs
                        .sources
                        .iter()
                        .find_map(|(path, source)| match source {
                            crate::serve::resource_changes::SourceState::File {
                                identity: before,
                                ..
                            } if before == identity => Some(path),
                            _ => None,
                        })
                        .ok_or("Move introduced an unknown source identity")?;
                    inputs
                        .saved_sources
                        .get(original)
                        .ok_or("Move lost its saved baseline")?
                        .clone()
                }
            };
            states.insert(patch.path.clone(), saved);
        }
        Ok(Some(SavedMoveProjection { states, interrupted: false }))
    }
    pub(crate) struct Publication {
        root: std::path::PathBuf,
        coordination: leases::Coordination,
        bound: storage::Bound,
        intent: Option<LiveIntent>,
        change: Option<EntryChange>,
        request: String,
        manifest: String,
        reviewed_fingerprint: String,
        begun: bool,
        confirmed: bool,
        failure_original: Option<super::super::failures::Evidence>,
        writers_started: bool,
        no_effect: Option<EntryStamp>,
        orphan_base: Option<(String, String)>,
        _ordinary: crate::engine::private_policy::Ordinary,
    }
    impl Publication {
        pub(crate) fn admit(
            state: &LocalState,
            manifest: &crate::serve::resource_journal::Manifest,
            transactional: bool,
            reviewed_fingerprint: &str,
        ) -> Result<Self, String> {
            super::super::ordinary(state)?;
            let ordinary = crate::engine::private_policy::ordinary();
            leases::ordinary_root(state.index.root())?;
            let root = state.index.root().to_owned();
            let coordination = leases::coordination(&root, true)?;
            coordination.check_exclusive_for(&root)?;
            let base = hickory_workspace::data_root().map_err(|error| error.to_string())?;
            let bound = storage::Bound::resolve_coordinated(&base, &root, &coordination)?;
            let reserved = reserved_context(&bound.held.key, &manifest.context)?;
            let live = match obtain(&manifest.context) {
                Ok(value) => Some(
                    value
                        .lock()
                        .map_err(|_| "Entry review is unavailable")?
                        .clone(),
                ),
                Err(error) if reserved => return Err(error),
                Err(_) => None,
            };
            let completing = live.as_ref().is_some_and(|intent| intent.admission.interrupted().is_some());
            if bound.payload.pending.is_some() != completing {
                return Err("The original interrupted entry publication changed. Read its current recovery before publishing.".into());
            }
            if live.is_none()
                && manifest.restore_of.as_ref().is_some_and(|origin| {
                    bound
                        .payload
                        .history
                        .iter()
                        .any(|row| row.request_id == origin.recovery_id)
                        || interrupted_record(&bound.payload, &origin.recovery_id)
                })
            {
                return Err("Retained entry restoration requires a fresh live entry intent. Historical metadata is not application authority.".into());
            }
            let mut change = None;
            let mut orphan_base = None;
            if let Some(intent) = &live {
                check(state, intent, &bound)?;
                match &intent.operation {
                    Operation::Move { .. } => {
                        if manifest.restore_of.is_some()
                            || intent.plan.as_deref()
                                != Some(hash(&manifest.reviewed_plan)?.as_str())
                        {
                            return Err("Entry publication differs from its exact engine-prepared move. Prepare a fresh review.".into());
                        }
                    }
                    Operation::Restoration {
                        recovery_id,
                        recovery_fingerprint,
                    } => {
                        let origin = manifest
                            .restore_of
                            .as_ref()
                            .ok_or("Entry restoration has no retained original")?;
                        if &origin.recovery_id != recovery_id
                            || &origin.fingerprint != recovery_fingerprint
                        {
                            return Err(
                                "Entry restoration changed its original sealed recovery record."
                                    .into(),
                            );
                        }
                        let record =
                            crate::serve::resource_journal::read_recovery(&root, recovery_id)?;
                        checked_inverse(&root, &bound, intent, &record)?;
                    }
                    Operation::InterruptedCompletion => {
                        let namespace = interrupted_namespace(state, intent, &bound)?;
                        let sources: BTreeMap<_, _> = manifest.reviewed_plan.patches.iter()
                            .map(|patch| (patch.path.clone(), patch.before.clone())).collect();
                        let expected = completion_plan(intent, &namespace, &sources)?;
                        if manifest.version != 4 || manifest.restore_of.is_some()
                            || intent.sources.as_deref() != Some(hash(&sources)?.as_str())
                            || intent.plan.as_deref() != Some(hash(&manifest.reviewed_plan)?.as_str())
                            || hash(&expected.plan)? != hash(&manifest.reviewed_plan)?
                            || expected.saved_before != manifest.before || expected.saved_after != manifest.after
                        {
                            return Err("Interrupted publication differs from its original saved and draft review.".into());
                        }
                        let old = intent.response.before_entry.path.as_ref().ok_or("Interrupted Move lost its old source")?;
                        if matches!(manifest.before[old], SavedSource::Missing)
                            && matches!(sources[old], SourceState::File { .. })
                        {
                            let SavedSource::File { text, .. } = &namespace.original.before[old] else {
                                return Err("Retained orphan lacks its original saved room baseline.".into());
                            };
                            orphan_base = Some((old.clone(), text.clone()));
                        }
                    }
                }
                let source_identity = bound
                    .payload
                    .current
                    .path
                    .as_ref()
                    .and_then(|path| {
                        manifest
                            .reviewed_plan
                            .patches
                            .iter()
                            .find(|patch| &patch.path == path)
                    })
                    .and_then(|patch| match &patch.before {
                        SourceState::File { identity, .. } => Some(identity.clone()),
                        _ => None,
                    })
                    .or_else(|| match &intent.operation {
                        Operation::Restoration { recovery_id, .. } => bound
                            .payload
                            .history
                            .iter()
                            .find(|row| &row.request_id == recovery_id)
                            .and_then(|row| row.change.source_identity.clone()),
                        _ => None,
                    });
                let (kind, source_identity) = match &intent.operation {
                    Operation::Move { .. } => (TransitionKind::Move, source_identity),
                    Operation::Restoration { .. } => (TransitionKind::Restoration, source_identity),
                    Operation::InterruptedCompletion => (
                        TransitionKind::InterruptedCompletion,
                        manifest.reviewed_plan.patches.iter().find_map(|patch| {
                            (Some(&patch.path) == intent.response.after_entry.path.as_ref())
                                .then_some(&patch.after).and_then(|after| match after {
                                    SourceState::File { identity, .. } => Some(identity.clone()),
                                    SourceState::Missing { .. } => None,
                                })
                        }),
                    ),
                };
                let candidate = EntryChange {
                    version: 1,
                    kind,
                    before: bound.payload.current.clone(),
                    after: intent.response.after_entry.clone(),
                    source_identity,
                };
                candidate.validate(&manifest.reviewed_plan)?;
                change = Some(candidate);
            } else if let Some(main) = &bound.payload.current.path {
                for patch in &manifest.reviewed_plan.patches {
                    if matches!(patch.after, SourceState::Missing { .. })
                        && leases::protects_main_namespace(&root, main, &patch.path)?
                    {
                        return Err("Moving or deleting the retained main requires an explicit reviewed workspace-entry intent. Neither provider artifacts nor missing files grant that transition.".into());
                    }
                }
            } else if bound.payload.bootstrap_default {
                if let Some(patch) = manifest
                    .reviewed_plan
                    .patches
                    .iter()
                    .find(|patch| patch.path == "project.md")
                {
                    if let SourceState::File { identity, text, .. } = &patch.after {
                        if hick_lang::parse(text)
                            .is_ok_and(|document| !document.find_tags("project").is_empty())
                        {
                            let candidate = EntryChange {
                                version: 1,
                                kind: TransitionKind::Create,
                                before: bound.payload.current.clone(),
                                after: bound.payload.current.advance(Some(patch.path.clone()))?,
                                source_identity: Some(identity.clone()),
                            };
                            candidate.validate(&manifest.reviewed_plan)?;
                            change = Some(candidate);
                        }
                    }
                }
            }
            if change.is_some() && bound.payload.history.len() >= 256 {
                return Err("The bounded main-transition journal is full; no original identity or evidence was discarded.".into());
            }
            if change.is_some() {
                let files = manifest
                    .after
                    .values()
                    .filter(|state| {
                        matches!(
                            state,
                            crate::serve::resource_inputs::SavedSource::File { .. }
                        )
                    })
                    .count();
                let retained = serde_json::to_vec(&bound.payload)
                    .map_err(|error| error.to_string())?
                    .len();
                if retained
                    .saturating_add(files.saturating_mul(20 * 1024))
                    .saturating_add(64 * 1024)
                    > 32 * 1024 * 1024
                {
                    return Err("Entry transition exceeds its bounded producer-evidence capacity. No prior identity was discarded and no source was written.".into());
                }
            }
            if change.is_some() && transactional {
                return Err("Entry transitions do not use automatic conflict rollback. Review the explicit inverse from retained source evidence.".into());
            }
            let request = manifest
                .request_id
                .clone()
                .ok_or("Resource publication lacks its retained request identity")?;
            let fingerprint = hash(manifest)?;
            let failure_original = change
                .as_ref()
                .map(|_| super::super::failures::Evidence::prepare(&root, manifest))
                .transpose()?;
            if change.is_some()
                && bound
                    .payload
                    .no_effects
                    .as_ref()
                    .is_some_and(|rows| rows.len() >= 256)
            {
                return Err(
                    "Retained no-effect evidence capacity is full; no source was written".into(),
                );
            }
            let result = Self {
                root,
                coordination,
                bound,
                intent: live,
                change,
                request,
                manifest: fingerprint,
                reviewed_fingerprint: reviewed_fingerprint.into(),
                begun: false,
                confirmed: false,
                failure_original,
                writers_started: false,
                no_effect: None,
                orphan_base,
                _ordinary: ordinary,
            };
            result.check(state, &std::collections::BTreeSet::new())?;
            Ok(result)
        }
        pub(in crate::serve) fn prepare_lineage(
            &self,
            root: &Path,
            resource_id: &str,
            manifest: &crate::serve::resource_journal::Manifest,
            before_identities: &BTreeMap<String, String>,
        ) -> Result<crate::serve::source_history::ResourceLineageCapture, String> {
            use crate::serve::source_history;
            let Some(proof) = self.intent.as_ref().and_then(|intent| intent.entry_lineage.as_deref()) else {
                return source_history::prepare_resource_lineage(root, resource_id, manifest, before_identities);
            };
            self.coordination.check_exclusive_for(root)?;
            let admission = proof.admit(&self.bound)?;
            source_history::prepare_resource_lineage_with_entry(root, resource_id, manifest, before_identities, &admission)
        }
        pub(crate) fn check(
            &self,
            state: &LocalState,
            published: &std::collections::BTreeSet<String>,
        ) -> Result<(), String> {
            self.coordination.check_exclusive_for(&self.root)?;
            self.bound.unchanged()?;
            leases::ordinary_root(&self.root)?;
            state.resource_root.check(&self.root)?;
            if let Some(intent) = &self.intent {
                if intent.created.elapsed() >= LIFETIME {
                    return Err("Original entry review expired during publication. Retained source evidence was not retried.".into());
                }
                for parent in &intent.parents {
                    if crate::serve::resource_inputs::RootIdentity::fingerprint(
                        &self.root.join(&parent.path),
                    )? != parent.identity
                    {
                        return Err("An original entry source/destination parent changed during publication.".into());
                    }
                }
                let expected = intent.admission.before();
                if &self.bound.payload.current != expected {
                    return Err(
                        "Entry generation changed during its reviewed source publication.".into(),
                    );
                }
                if !self.begun {
                    check(state, intent, &self.bound)?;
                } else if self.bound.payload.pending.as_ref().is_none_or(|pending| {
                    pending.request_id != self.request || pending.manifest_sha256 != self.manifest
                }) {
                    return Err(
                        "Entry pending record no longer belongs to this exact source publication."
                            .into(),
                    );
                }
                if let Some(path) = &expected.path {
                    if !published.contains(path) {
                        if let Some(guard) = intent.admission.original_guard() {
                            crate::serve::source_history::validate_observed_guard(
                                &self.root, &self.root.join(path), path, &guard,
                            )?;
                        } else if intent.admission.interrupted().is_none() {
                            return Err("Entry intent lost its original source guard".into());
                        }
                    }
                }
            }
            Ok(())
        }
        pub(crate) fn begin(&mut self) -> Result<(), String> {
            if self.begun {
                return Ok(());
            }
            self.coordination.check_exclusive_for(&self.root)?;
            if let Some(change) = &self.change {
                let recovery = if change.kind == TransitionKind::InterruptedCompletion {
                    let recovery = self.bound.payload.pending.as_ref().ok_or("Interrupted Move pending record disappeared")?.next_recovery(&self.root)?;
                    if recovery.reservation_floor != change.after.generation {
                        return Err("Interrupted completion reservation differs from the reviewed generation.".into());
                    }
                    Some(recovery)
                } else {
                    None
                };
                if change.kind == TransitionKind::Move {
                    self.bound.payload.ownership = Some(super::super::claims::retained_inventory(
                        &self.root, &change.before, self.bound.payload.ownership.as_ref(),
                    )?);
                }
                self.bound.payload.pending = Some(super::super::Pending {
                    request_id: self.request.clone(),
                    manifest_sha256: self.manifest.clone(),
                    change: change.clone(),
                    after_guard: None,
                    failure: self.failure_original.clone(),
                    reviewed_fingerprint: Some(self.reviewed_fingerprint.clone()),
                    recovery,
                    namespace_guards: None,
                    before_guard: self
                        .intent
                        .as_ref()
                        .and_then(|intent| intent.admission.original_guard()),
                });
                self.bound.persist()?;
                write_hint(&self.root, &change.before.identity, change.after.generation)?;
            }
            self.begun = true;
            Ok(())
        }
        pub(crate) fn before_path(&mut self, path: &str) -> Result<(), String> {
            self.begin()?;
            if self.change.is_some() {
                let pending = self
                    .bound
                    .payload
                    .pending
                    .as_mut()
                    .ok_or("Entry pending publication disappeared")?;
                pending
                    .failure
                    .as_mut()
                    .ok_or("Entry attempt has no original namespace evidence")?
                    .attempt(path)?;
                self.bound.persist()?;
            }
            // Only the live invocation can know that no filesystem writer began.
            // Restarted records never reconstruct this flag as live authority.
            self.writers_started = true;
            Ok(())
        }
        pub(crate) fn observed_path(
            &mut self,
            path: &str,
            saved: &crate::serve::resource_inputs::SavedSource,
            written: &crate::serve::resource_publish::ObservedWrite,
        ) -> Result<(), String> {
            if self.change.is_some() {
                let pending = self
                    .bound
                    .payload
                    .pending
                    .as_mut()
                    .ok_or("Entry pending publication disappeared")?;
                pending
                    .failure
                    .as_mut()
                    .ok_or("Entry producer has no original namespace evidence")?
                    .observe(&self.root, path, saved, written)?;
                self.bound.persist()?;
            }
            Ok(())
        }
        fn finish_no_effect(
            &mut self,
            manifest: &crate::serve::resource_journal::Manifest,
        ) -> Result<(), String> {
            if self.writers_started || self.change.is_none() {
                return Ok(());
            }
            if self.change.as_ref().is_some_and(|change| change.kind == TransitionKind::InterruptedCompletion) {
                self.failure_original.as_ref().ok_or("Interrupted no-effect refusal lost its namespace proofs")?
                    .no_effect(&self.root, manifest)?;
                let pending = self.bound.payload.pending.as_ref().ok_or("Interrupted no-effect refusal lost the prior pending Move")?;
                if self.begun && (pending.request_id != self.request || pending.manifest_sha256 != self.manifest) {
                    return Err("An unrelated pending entry cannot confirm interrupted no-effect.".into());
                }
                self.no_effect = Some(self.bound.payload.current.clone());
                return Ok(());
            }
            let Some(pending) = self.bound.payload.pending.as_ref() else {
                self.failure_original
                    .as_ref()
                    .ok_or("No-effect refusal lost its original namespace proofs")?
                    .no_effect(&self.root, manifest)?;
                self.no_effect = Some(self.bound.payload.current.clone());
                return Ok(());
            };
            if pending.request_id != self.request || pending.manifest_sha256 != self.manifest {
                return Err("An unrelated pending entry cannot be retired as no-effect".into());
            }
            self.failure_original
                .as_ref()
                .ok_or("No-effect cleanup lost its original source proofs")?
                .no_effect(&self.root, manifest)?;
            let before = self.bound.payload.current.clone();
            let after = before.advance(before.path.clone())?;
            write_hint(&self.root, &before.identity, after.generation)?;
            let row = super::super::failures::NoEffect {
                request_id: self.request.clone(),
                manifest_sha256: self.manifest.clone(),
                before,
                after: after.clone(),
            };
            self.bound
                .payload
                .no_effects
                .get_or_insert_with(Vec::new)
                .push(row);
            self.bound.payload.current = after.clone();
            self.bound.payload.pending = None;
            self.bound.persist()?;
            self.no_effect = Some(after);
            Ok(())
        }
        pub(crate) fn orphan_saved_base(&self, path: &str) -> Option<&str> {
            self.orphan_base.as_ref().filter(|(original, _)| original == path).map(|(_, base)| base.as_str())
        }

        pub(crate) fn is_document_source(&self, path: &str) -> bool {
            path.ends_with(".md")
                || self.bound.payload.current.path.as_deref() == Some(path)
                || self.change.as_ref().is_some_and(|change| {
                    change.before.path.as_deref() == Some(path)
                        || change.after.path.as_deref() == Some(path)
                })
        }
        pub(crate) fn document_entry(&self) -> Option<&str> {
            self.change
                .as_ref()
                .map(|change| &change.after)
                .unwrap_or(&self.bound.payload.current)
                .path
                .as_deref()
        }
        pub(crate) fn finish(
            &mut self,
            state: &LocalState,
            manifest: &crate::serve::resource_journal::Manifest,
            outcome: &crate::serve::resource_publish::Outcome,
            files: &[Value],
        ) -> Result<(), String> {
            self.coordination.check_exclusive_for(&self.root)?;
            self.bound.unchanged()?;
            if !outcome.complete {
                if outcome.applied.is_empty() {
                    self.finish_no_effect(manifest)?;
                }
                return Ok(());
            }
            if let Some(change) = &self.change {
                if !self.begun {
                    return Err("Entry transition has no durable pending publication. No new main generation was confirmed.".into());
                }
                let mut guards = BTreeMap::new();
                for file in files {
                    let path = file
                        .get("file")
                        .and_then(|file| file.get("path"))
                        .and_then(Value::as_str)
                        .ok_or("Entry receipt source path is unavailable")?;
                    let guard = file
                        .get("save_guard")
                        .and_then(Value::as_str)
                        .ok_or("Entry receipt has no observed source guard")?;
                    guards.insert(path.to_owned(), guard.to_owned());
                }
                validate_namespace_guards(&self.root, manifest, &guards)?;
                let record =
                    crate::serve::resource_journal::read_recovery(&self.root, &self.request)?;
                if record.manifest_sha256 != self.manifest || !record.all_paths_recorded_applied {
                    return Err(
                        "Entry completion is missing its exact retained source publication.".into(),
                    );
                }
                let after_guard = change
                    .after
                    .path
                    .as_ref()
                    .and_then(|path| guards.get(path))
                    .cloned();
                let guards_for_history = guards.clone();
                let recovered_from = {
                    let pending = self
                        .bound
                        .payload
                        .pending
                        .as_mut()
                        .ok_or("Entry pending publication disappeared")?;
                    pending.after_guard = after_guard.clone();
                    pending.namespace_guards = Some(guards);
                    pending.recovery.clone()
                };
                self.bound.persist()?;
                let before_guard = self
                    .intent
                    .as_ref()
                    .and_then(|intent| intent.admission.original_guard());
                self.bound
                    .payload
                    .history
                    .push(super::super::RetainedTransition {
                        request_id: self.request.clone(),
                        manifest_sha256: self.manifest.clone(),
                        change: change.clone(),
                        after_guard,
                        before_guard,
                        namespace_guards: Some(guards_for_history),
                        recovered_from,
                    });
                self.bound.payload.ownership = if change.after.path.is_some() {
                    match super::super::claims::inventory(&self.root, &change.after) {
                        Ok(inventory) => Some(inventory),
                        Err(_) => self.bound.payload.ownership.clone(),
                    }
                } else {
                    None
                };
                self.bound.payload.current = change.after.clone();
                self.bound.payload.bootstrap_default = false;
                self.bound.payload.pending = None;
                self.bound.persist()?;
                self.confirmed = true;
                // Positive registration follows the declared role, not filename extension.
                if let Some(path) = &change.after.path {
                    state.index.add(path);
                }
            }
            Ok(())
        }
        pub(crate) fn proved_no_effect(&self) -> bool {
            self.no_effect.is_some()
        }
        pub(crate) fn outcome(&self) -> Option<Value> {
            if let Some(change) = &self.change {
                if let Some(reading) = self.intent.as_ref().and_then(|intent| intent.admission.interrupted()) {
                    let previous = reading.material.reservation_floor;
                    let floor = if self.confirmed { change.after.generation } else {
                        self.bound.payload.pending.as_ref().map(|pending| pending.reservation_floor()).unwrap_or(previous)
                    };
                    let remaining = (!self.confirmed).then(|| json!({
                        "request_id": reading.material.request_id,
                        "manifest_sha256": reading.material.manifest_sha256
                    }));
                    return Some(json!({
                        "version":2, "kind":"interrupted_completion",
                        "state":if self.confirmed { "confirmed" } else if self.no_effect.is_some() { "no_effect" } else { "pending" },
                        "request_id":self.request, "before_entry":change.before,
                        "after_entry":change.after, "current_entry":self.bound.payload.current,
                        "reservation_floor_before":previous, "reservation_floor":floor,
                        "reservation_consumed":floor > previous, "remaining_interrupted":remaining
                    }));
                }
            }
            if let (Some(current), Some(change)) = (&self.no_effect, &self.change) {
                return Some(
                    json!({"version":1,"state":"no_effect","request_id":self.request,"before_entry":change.before,"after_entry":change.after,"current_entry":current,"reservation_consumed":current.generation!=change.before.generation,"notice":"No filesystem writer began. The original namespace was revalidated. Any persisted reservation consumed a generation; an unreserved refusal keeps its original generation."}),
                );
            }
            self.change.as_ref().map(|change|json!({"version":1,"state":if self.confirmed{"confirmed"}else{"pending"},"request_id":self.request,"before_entry":change.before,"after_entry":change.after,"current_entry":if self.confirmed{&change.after}else{&change.before}}))
        }
    }
    fn validate_namespace_guards(
        root: &Path,
        manifest: &crate::serve::resource_journal::Manifest,
        guards: &BTreeMap<String, String>,
    ) -> Result<(), String> {
        let files: std::collections::BTreeSet<_> = manifest
            .after
            .iter()
            .filter_map(|(path, state)| {
                matches!(
                    state,
                    crate::serve::resource_inputs::SavedSource::File { .. }
                )
                .then_some(path)
            })
            .collect();
        if files != guards.keys().collect() {
            return Err(
                "Entry completion requires the exact observed after-source namespace.".into(),
            );
        }
        for (path, expected) in &manifest.after {
            let (full, actual) = crate::serve::resource_inputs::saved_target(root, path)?;
            if &actual != expected {
                return Err(format!(
                    "Entry saved namespace changed before confirmation: {path}"
                ));
            }
            if let Some(guard) = guards.get(path) {
                crate::serve::source_history::validate_observed_guard(root, &full, path, guard)?;
            }
        }
        Ok(())
    }
    fn write_hint(root: &Path, identity: &str, generation: u32) -> Result<(), String> {
        use std::io::Write;
        let directory = root.join(".hick-cache");
        crate::serve::resource_journal::regular_directory(&directory)?;
        let path = directory.join("workspace-entry-role.json");
        let original = match std::fs::symlink_metadata(&path) {
            Ok(_) => Some(crate::serve::resource_journal::read_regular(&path, 4096)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.to_string()),
        };
        if let Some(bytes) = &original {
            let value: Value =
                serde_json::from_slice(bytes).map_err(|_| "Entry recovery hint is damaged")?;
            if value.get("identity").and_then(Value::as_str) != Some(identity)
                || value
                    .get("generation_floor")
                    .and_then(Value::as_u64)
                    .is_some_and(|before| before > generation as u64)
            {
                return Err(
                    "Entry recovery hint belongs to another identity or newer generation.".into(),
                );
            }
        }
        let bytes = serde_json::to_vec(
            &json!({"version":1,"identity":identity,"generation_floor":generation}),
        )
        .map_err(|error| error.to_string())?;
        let mut temporary =
            tempfile::NamedTempFile::new_in(&directory).map_err(|error| error.to_string())?;
        temporary
            .write_all(&bytes)
            .map_err(|error| error.to_string())?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|error| error.to_string())?;
        if original.is_some() {
            if crate::serve::resource_journal::read_regular(&path, 4096)? != original.unwrap() {
                return Err("Entry recovery hint changed before retention.".into());
            }
            temporary
                .persist(&path)
                .map_err(|error| error.to_string())?;
        } else {
            temporary
                .persist_noclobber(&path)
                .map_err(|error| error.to_string())?;
        }
        std::fs::File::open(&directory)
            .and_then(|file| file.sync_all())
            .map_err(|error| error.to_string())?;
        if crate::serve::resource_journal::read_regular(&path, 4096)? != bytes {
            return Err("Entry recovery hint changed after retention.".into());
        }
        Ok(())
    }
</ui:copy>

## Recover metadata without replaying source publication

A retained transition is read-only evidence. Restoration needs a fresh original reading and the same producer namespace; equal bytes from a replaced inode cannot recreate authority. Restart recovery may only finish a pending metadata commit whose complete record and sealed after guards still match. It never invokes the source writer. The focused models exercise material drift, expired live authority and literal URI routing; integration evidence must also drive the real source path.

<ui:copy id="workspace-entry-transition-recovery">
    fn retained_transition<'a>(
        binding: &'a Binding,
        record: &crate::serve::resource_journal::Recovery,
    ) -> Result<&'a super::super::RetainedTransition, String> {
        binding
            .history
            .iter()
            .find(|row| {
                row.request_id == record.id && row.manifest_sha256 == record.manifest_sha256
            })
            .ok_or_else(|| "This record has no sealed retained entry transition".into())
    }
    fn check_retained_namespace(
        root: &Path,
        binding: &Binding,
        record: &crate::serve::resource_journal::Recovery,
    ) -> Result<(), String> {
        let row = retained_transition(binding, record)?;
        let guards=row.namespace_guards.as_ref().ok_or("This historical entry record has no sealed producer namespace; it remains evidence only")?;
        validate_namespace_guards(root, &record.manifest, guards)
    }
    pub(crate) fn recovery_metadata(
        root: &Path,
        record: &crate::serve::resource_journal::Recovery,
    ) -> Value {
        let read = (|| -> Result<Value, String> {
            let base = hickory_workspace::data_root().map_err(|error| error.to_string())?;
            let Some(bound) = storage::Bound::optional_under(&base, root)? else {
                return Ok(Value::Null);
            };
            if bound
                .payload
                .pending
                .as_ref()
                .is_some_and(|pending| pending.request_id == record.id)
            {
                return Err("This retained main transition remains pending; inspect its source receipts without applying historical authority.".into());
            }
            if let Some(row) = bound
                .payload
                .no_effects
                .as_ref()
                .and_then(|rows| rows.iter().find(|row| row.request_id == record.id))
            {
                if row.manifest_sha256 != record.manifest_sha256 {
                    return Err("No-effect evidence does not match this retained manifest".into());
                }
                return Err("No filesystem writer began for this retained entry attempt. Its original namespace was checked and a new generation retired the refused live intent; it grants no inverse authority.".into());
            }
            if interrupted_record(&bound.payload, &record.id) {
                return Err("This record belongs to an interrupted Move and its recovery. Its saved and draft snapshots remain readable. A generic inverse cannot restore the interrupted namespace.".into());
            }
            let Some(row) = bound
                .payload
                .history
                .iter()
                .find(|row| row.request_id == record.id)
            else {
                return Ok(Value::Null);
            };
            if row.manifest_sha256 != record.manifest_sha256 {
                return Err(
                    "Retained entry transition does not match this recovery manifest.".into(),
                );
            }
            Ok(
                json!({"version":1,"state":"retained","request_id":row.request_id,"manifest_sha256":row.manifest_sha256,"kind":row.change.kind,"before_entry":row.change.before,"after_entry":row.change.after}),
            )
        })();
        match read {
            Ok(value) => value,
            Err(notice) => json!({"version":1,"state":"unavailable","notice":notice}),
        }
    }
    pub(crate) fn recover_completed_under(base: &Path, root: &Path) -> Result<bool, String> {
        let Some(snapshot) = storage::Bound::optional_under(base, root)? else {
            return Ok(false);
        };
        let Some(pending) = snapshot.payload.pending.clone() else {
            return Ok(false);
        };
        if pending.namespace_guards.is_none() {
            return Ok(false);
        }
        drop(snapshot);
        let coordination = leases::coordination_under(base, root, true)?;
        coordination.check_exclusive_for(root)?;
        let mut bound = storage::Bound::open_under(base, root, true)?;
        let Some(current) = bound.payload.pending.clone() else {
            return Ok(false);
        };
        if current.request_id != pending.request_id
            || current.manifest_sha256 != pending.manifest_sha256
        {
            return Err("Pending entry recovery changed before confirmation. No different transition was selected.".into());
        }
        let guards = current
            .namespace_guards
            .as_ref()
            .ok_or("Pending entry recovery has no sealed producer namespace")?;
        let record = crate::serve::resource_journal::read_recovery(root, &current.request_id)?;
        if !record.all_paths_recorded_applied || record.manifest_sha256 != current.manifest_sha256 {
            return Err("Entry recovery has no exact complete source publication. No source was republished.".into());
        }
        current.change.validate(&record.manifest.reviewed_plan)?;
        validate_namespace_guards(root, &record.manifest, guards)?;
        crate::serve::source_history::validate_resource_after(root, &record)?;
        write_hint(
            root,
            &current.change.before.identity,
            current.change.after.generation,
        )?;
        bound
            .payload
            .history
            .push(super::super::RetainedTransition {
                request_id: current.request_id,
                manifest_sha256: current.manifest_sha256,
                change: current.change.clone(),
                after_guard: current.after_guard,
                before_guard: current.before_guard,
                namespace_guards: Some(guards.clone()),
                recovered_from: current.recovery.clone(),
            });
        bound.payload.ownership = if current.change.after.path.is_some() {
            match super::super::claims::inventory(root, &current.change.after) {
                Ok(inventory) => Some(inventory),
                Err(_) => bound.payload.ownership.clone(),
            }
        } else {
            None
        };
        bound.payload.current = current.change.after;
        bound.payload.bootstrap_default = false;
        bound.payload.pending = None;
        bound.persist()?;
        coordination.check_exclusive_for(root)?;
        Ok(true)
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        fn no_effect_fixture() -> (
            tempfile::TempDir,
            std::path::PathBuf,
            crate::serve::resource_journal::Manifest,
            Publication,
        ) {
            use crate::serve::resource_changes::ResourcePatch;
            use crate::serve::resource_inputs::SavedSource;
            let temporary = tempfile::tempdir().unwrap();
            let root = temporary.path().join("workspace");
            let base = temporary.path().join("private");
            std::fs::create_dir(&root).unwrap();
            let source = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project/></h:doc>";
            std::fs::write(root.join("project.md"), source).unwrap();
            let coordination = leases::coordination_under(&base, &root, true).unwrap();
            let bound = storage::Bound::resolve_coordinated(&base, &root, &coordination).unwrap();
            let before = bound.payload.current.clone();
            let change = EntryChange {
                version: 1,
                kind: TransitionKind::Move,
                before: before.clone(),
                after: before.advance(Some("next.spec".into())).unwrap(),
                source_identity: Some("original".into()),
            };
            let original = SourceState::File {
                identity: "original".into(),
                text: source.into(),
                writable: true,
                removable: true,
                version: None,
            };
            let missing = SourceState::Missing { creatable: true };
            let saved = crate::serve::resource_inputs::saved_target(&root, "project.md")
                .unwrap()
                .1;
            let manifest = crate::serve::resource_journal::Manifest {
                version: 2,
                workspace: bound.payload.physical_root.clone(),
                context: "fixture".into(),
                title: "Move".into(),
                request_id: Some("7".repeat(32)),
                restore_of: None,
                before: BTreeMap::from([
                    ("project.md".into(), saved.clone()),
                    ("next.spec".into(), SavedSource::Missing),
                ]),
                after: BTreeMap::from([
                    ("project.md".into(), SavedSource::Missing),
                    ("next.spec".into(), saved),
                ]),
                reviewed_plan: ResourcePlan {
                    version: 1,
                    context: "fixture".into(),
                    title: "Move".into(),
                    annotations: vec![],
                    patches: vec![
                        ResourcePatch {
                            path: "project.md".into(),
                            before: original.clone(),
                            after: missing.clone(),
                        },
                        ResourcePatch {
                            path: "next.spec".into(),
                            before: missing,
                            after: original,
                        },
                    ],
                },
            };
            let evidence =
                super::super::super::failures::Evidence::prepare(&root, &manifest).unwrap();
            let publication = Publication {
                root,
                coordination,
                bound,
                intent: None,
                change: Some(change),
                request: "7".repeat(32),
                manifest: hash(&manifest).unwrap(),
                reviewed_fingerprint: "f".repeat(64),
                begun: false,
                confirmed: false,
                failure_original: Some(evidence),
                writers_started: false,
                no_effect: None,
                orphan_base: None,
                _ordinary: crate::engine::private_policy::ordinary(),
            };
            (temporary, base, manifest, publication)
        }
        #[test]
        fn unreserved_refusal_reports_exact_no_effect_without_advancing_role() {
            let (_temporary, _base, manifest, mut publication) = no_effect_fixture();
            let original = publication.bound.payload.current.clone();
            publication.finish_no_effect(&manifest).unwrap();
            let outcome = publication.outcome().unwrap();
            assert_eq!(outcome["state"], "no_effect");
            assert_eq!(outcome["reservation_consumed"], false);
            assert_eq!(publication.bound.payload.current, original);
            assert!(publication.bound.payload.pending.is_none());
            assert!(publication.bound.payload.no_effects.is_none());
        }
        #[test]
        fn persisted_zero_writer_reservation_consumes_generation_and_survives_restart() {
            let (_temporary, base, manifest, mut publication) = no_effect_fixture();
            let root = publication.root.clone();
            let original = publication.bound.payload.current.clone();
            publication.begin().unwrap();
            assert!(publication.bound.payload.pending.is_some());
            publication.finish_no_effect(&manifest).unwrap();
            assert_eq!(publication.outcome().unwrap()["reservation_consumed"], true);
            drop(publication);
            let reopened = storage::Bound::open_under(&base, &root, false).unwrap();
            assert_eq!(
                reopened.payload.current,
                original.advance(original.path.clone()).unwrap()
            );
            assert!(reopened.payload.pending.is_none());
            assert_eq!(reopened.payload.no_effects.as_ref().unwrap().len(), 1);
        }
        #[test]
        fn started_writer_or_foreign_pending_never_clears_as_no_effect() {
            let (_temporary, _base, manifest, mut publication) = no_effect_fixture();
            publication.before_path("next.spec").unwrap();
            publication.finish_no_effect(&manifest).unwrap();
            assert!(!publication.proved_no_effect());
            assert!(publication.bound.payload.pending.is_some());
            publication.writers_started = false;
            publication.request = "8".repeat(32);
            assert!(publication.finish_no_effect(&manifest).is_err());
            assert!(publication.bound.payload.pending.is_some());
        }
        #[test]
        fn same_byte_source_replacement_keeps_zero_writer_reservation_pending() {
            let (_temporary, _base, manifest, mut publication) = no_effect_fixture();
            publication.begin().unwrap();
            let path = publication.root.join("project.md");
            let source = std::fs::read(&path).unwrap();
            std::fs::remove_file(&path).unwrap();
            std::fs::write(path, source).unwrap();
            assert!(publication.finish_no_effect(&manifest).is_err());
            assert!(publication.bound.payload.pending.is_some());
            assert!(!publication.proved_no_effect());
        }

        fn capture() -> Capture {
            serde_json::from_value(json!({"context":"original","edit":{},"drafts":{"main.spec":{"text":"draft\r\n","base_sha256":"a".repeat(64),"version":1,"writable":true}},"project_entry":null,"ownership_sources":{},"unaddressed_sources":{},"saved_guards":{}})).unwrap()
        }
        #[test]
        fn retained_material_may_expand_but_never_rebind_original_text_version_or_writability() {
            let initial = capture();
            let mut original = Originals::default();
            original.extend(&initial).unwrap();
            for kind in ["text", "version", "writable", "removed"] {
                let mut changed = initial.clone();
                match kind {
                    "text" => changed.drafts.get_mut("main.spec").unwrap().text.push('x'),
                    "version" => changed.drafts.get_mut("main.spec").unwrap().version = Some(2),
                    "writable" => changed.drafts.get_mut("main.spec").unwrap().writable = false,
                    _ => {
                        changed.drafts.clear();
                    }
                }
                assert!(original.clone().extend(&changed).is_err(), "{kind}")
            }
            let mut expanded = initial.clone();
            expanded
                .ownership_sources
                .insert("part.md".into(), "context".into());
            original.extend(&expanded).unwrap();
            expanded.ownership_sources.remove("part.md");
            assert!(original.extend(&expanded).is_err());
        }
        #[test]
        fn signed_intent_context_is_recognizable_but_never_rehydrates_live_authority() {
            let key = [7u8; 32];
            let value = token(&key).unwrap();
            assert_eq!(value.len(), 64);
            assert!(reserved_context(&key, &value).unwrap());
            assert!(!reserved_context(&[8u8; 32], &value).unwrap());
            assert!(obtain(&value).is_err());
            assert!(!reserved_context(&key, &"a".repeat(64)).unwrap());
        }
        #[test]
        fn literal_reserved_source_characters_keep_exact_resource_addresses() {
            let old = uri("literal%#main.spec").unwrap();
            assert!(old.contains("%25%23"));
            let edit = json!({"documentChanges":[{"kind":"rename","oldUri":old,"newUri":uri("next/main.spec").unwrap()}]});
            assert_eq!(
                crate::serve::resource_changes::targets(&edit).unwrap(),
                vec!["literal%#main.spec", "next/main.spec"]
            );
        }
    }
</ui:copy>

## Keep unsupported providers truthful

Unix engines expose the checked service through the existing HTTP adapter. Engines without the durable role provider refuse entry operations while preserving their ordinary source paths. No host-specific renderer grants an entry transition.

<ui:copy id="workspace-entry-transition-http">
}
pub(crate) async fn intent(
    State(state): State<LocalState>,
    Json(request): Json<IntentRequest>,
) -> ApiResult<Json<IntentResponse>> {
    #[cfg(unix)]
    {
        state
            .resource_root
            .ready()
            .await
            .map_err(ApiError::conflict)?;
        tokio::task::spawn_blocking(move || unix::intent(state, request))
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?
            .map(Json)
            .map_err(ApiError::conflict)
    }
    #[cfg(not(unix))]
    {
        let _ = (state, request);
        Err(ApiError::conflict(
            "This engine cannot hold durable workspace-entry transitions. Source remains readable.",
        ))
    }
}
pub(crate) async fn interrupted_intent(
    State(state): State<LocalState>,
    Json(request): Json<super::recovery::IntentRequest>,
) -> ApiResult<Json<IntentResponse>> {
    #[cfg(unix)]
    {
        state.resource_root.ready().await.map_err(ApiError::conflict)?;
        tokio::task::spawn_blocking(move || unix::interrupted_intent(state, request))
            .await
            .map_err(|error| ApiError::internal(error.to_string()))?
            .map(Json)
            .map_err(ApiError::conflict)
    }
    #[cfg(not(unix))]
    {
        let _ = (state, request);
        Err(ApiError::unavailable("Interrupted entry completion is unavailable on this engine platform."))
    }
}
pub(crate) async fn inputs(
    State(state): State<LocalState>,
    Json(request): Json<InputRequest>,
) -> ApiResult<Json<Inputs>> {
    #[cfg(unix)]
    {
        unix::inputs(state, request)
            .await
            .map(Json)
            .map_err(ApiError::conflict)
    }
    #[cfg(not(unix))]
    {
        let _ = (state, request);
        Err(ApiError::conflict(
            "This engine cannot hold durable workspace-entry transitions. Source remains readable.",
        ))
    }
}

#[cfg(unix)]
pub(crate) use unix::{
    AcquisitionScope, Publication, capture_scope, plan_interrupted,
    recover_completed_under, recovery_metadata, saved_move_projection,
    validate_restoration_origin,
};
#[cfg(not(unix))]
pub(crate) struct AcquisitionScope;
#[cfg(not(unix))]
impl AcquisitionScope {
    pub(crate) fn canonical_entry(&self) -> &str { unreachable!("no interrupted entry scope on this platform") }
    pub(crate) fn check(&self, _root: &Path, _request: &Capture) -> Result<(), String> { Err("Interrupted acquisition is unavailable on this engine platform.".into()) }
    pub(crate) fn compiler_request(&self, request: &Capture) -> Capture { request.clone() }
    pub(crate) fn retained_generated_targets(&self, _paths: &[String]) -> Result<BTreeSet<String>, String> { Ok(BTreeSet::new()) }
    pub(crate) fn orphan_draft(&self, _path: &str, _request: &Capture, _saved: &SavedSource) -> Result<Option<Draft>, String> { Ok(None) }
    pub(crate) fn permits_retained_source(&self, _path: &str, _source: &SourceState, _text: &str) -> bool { false }
    pub(crate) fn validate_orphan_sources(&self, _sources: &BTreeMap<String, SourceState>) -> Result<(), String> { Ok(()) }
}
#[cfg(not(unix))]
pub(crate) fn capture_scope(_state: &LocalState, _request: &Capture) -> Result<Option<AcquisitionScope>, String> { Ok(None) }
#[cfg(not(unix))]
pub(crate) fn plan_interrupted(_state: &LocalState, _request: &crate::serve::resource_changes::Request) -> Result<Option<ResourcePlan>, String> { Ok(None) }
#[cfg(not(unix))]
pub(crate) struct Publication;
#[cfg(not(unix))]
impl Publication {
    pub(crate) fn admit(
        _state: &LocalState,
        _manifest: &crate::serve::resource_journal::Manifest,
        _transactional: bool,
        _reviewed_fingerprint: &str,
    ) -> Result<Self, String> {
        Ok(Self)
    }
    pub(in crate::serve) fn prepare_lineage(
        &self,
        root: &Path,
        resource_id: &str,
        manifest: &crate::serve::resource_journal::Manifest,
        before_identities: &BTreeMap<String, String>,
    ) -> Result<crate::serve::source_history::ResourceLineageCapture, String> {
        crate::serve::source_history::prepare_resource_lineage(root, resource_id, manifest, before_identities)
    }
    pub(crate) fn check(
        &self,
        _state: &LocalState,
        _published: &std::collections::BTreeSet<String>,
    ) -> Result<(), String> {
        Ok(())
    }
    pub(crate) fn begin(&mut self) -> Result<(), String> {
        Ok(())
    }
    pub(crate) fn before_path(&mut self, _path: &str) -> Result<(), String> {
        Ok(())
    }
    pub(crate) fn observed_path(
        &mut self,
        _path: &str,
        _saved: &crate::serve::resource_inputs::SavedSource,
        _written: &crate::serve::resource_publish::ObservedWrite,
    ) -> Result<(), String> {
        Ok(())
    }
    pub(crate) fn orphan_saved_base(&self, _path: &str) -> Option<&str> { None }
    pub(crate) fn is_document_source(&self, path: &str) -> bool {
        path.ends_with(".md")
    }
    pub(crate) fn document_entry(&self) -> Option<&str> {
        None
    }
    pub(crate) fn finish(
        &mut self,
        _state: &LocalState,
        _manifest: &crate::serve::resource_journal::Manifest,
        _outcome: &crate::serve::resource_publish::Outcome,
        _files: &[Value],
    ) -> Result<(), String> {
        Ok(())
    }
    pub(crate) fn proved_no_effect(&self) -> bool {
        false
    }
    pub(crate) fn outcome(&self) -> Option<Value> {
        None
    }
}
#[cfg(not(unix))]
pub(crate) fn validate_restoration_origin(
    state: &LocalState,
    _capture: &Capture,
    record: &crate::serve::resource_journal::Recovery,
) -> Result<(), String> {
    crate::serve::source_history::validate_resource_after(state.index.root(), record)
}
#[cfg(not(unix))]
pub(crate) fn recovery_metadata(
    _root: &Path,
    _record: &crate::serve::resource_journal::Recovery,
) -> Value {
    Value::Null
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/workspace_entry_transitions.rs"><ui:paste select="#workspace-entry-transition-wire"/><ui:paste select="#workspace-entry-transition-intent"/><ui:paste select="#workspace-entry-transition-publisher"/><ui:paste select="#workspace-entry-transition-recovery"/><ui:paste select="#workspace-entry-transition-http"/></ui:file>

</ui:doc>
