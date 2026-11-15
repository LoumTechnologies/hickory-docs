//! Ordered, reversible source resource proposals. This module performs no filesystem writes.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceState {
    File {
        identity: String,
        text: String,
        writable: bool,
        removable: bool,
        version: Option<i32>,
    },
    Missing {
        creatable: bool,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcePatch {
    pub path: String,
    pub before: SourceState,
    pub after: SourceState,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcePlan {
    pub version: u32,
    pub context: String,
    pub title: String,
    pub patches: Vec<ResourcePatch>,
    pub annotations: Vec<Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub context: String,
    pub title: String,
    pub edit: Value,
    pub sources: BTreeMap<String, SourceState>,
}
fn path(uri: &str) -> Result<String, String> {
    let raw = uri
        .strip_prefix("hick:///")
        .ok_or("Resource proposals require workspace source URIs.")?;
    if raw.contains(['?', '#']) {
        return Err("Resource source URIs cannot contain query or fragment metadata.".into());
    }
    let mut bytes = Vec::with_capacity(raw.len());
    let mut input = raw.as_bytes().iter().copied();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let a = input.next().and_then(|byte| (byte as char).to_digit(16));
            let b = input.next().and_then(|byte| (byte as char).to_digit(16));
            let (Some(a), Some(b)) = (a, b) else {
                return Err("Malformed resource URI escape.".into());
            };
            bytes.push((a * 16 + b) as u8);
        } else {
            bytes.push(byte);
        }
    }
    let path = String::from_utf8(bytes).map_err(|_| "Resource URI is not UTF-8.")?;
    validate_path(&path)?;
    Ok(path)
}
pub(super) fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.contains([':', '\\'])
        || path.chars().any(char::is_control)
        || path.split('/').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || [".git", ".hick-cache", ".hick-journal", "sessions"]
                    .iter()
                    .any(|reserved| part.eq_ignore_ascii_case(reserved))
        })
    {
        return Err(
            "Use a source path inside the workspace, outside protected history/session metadata, without dot segments."
                .into(),
        );
    }
    Ok(())
}
fn known<'a>(
    states: &'a BTreeMap<String, SourceState>,
    path: &str,
) -> Result<&'a SourceState, String> {
    states.get(path).ok_or_else(|| {
        format!("Acquire source or confirmed absence before preparing this proposal: {path}")
    })
}
fn can_remove(state: &SourceState) -> bool {
    matches!(
        state,
        SourceState::File {
            writable: true,
            removable: true,
            ..
        }
    )
}
fn source_uri(item: &Value, key: &str) -> Result<String, String> {
    path(
        item.get(key)
            .and_then(Value::as_str)
            .ok_or("Missing resource source URI.")?,
    )
}
fn boolean(value: &Value, key: &str) -> Result<bool, String> {
    match value.get(key) {
        None => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        _ => Err(format!("Malformed resource option: {key}")),
    }
}
fn annotation(
    item: &Value,
    definitions: &BTreeMap<String, Value>,
    used: &mut BTreeSet<String>,
) -> Result<(), String> {
    if let Some(id) = item.get("annotationId") {
        let id = id.as_str().ok_or("Malformed proposal annotation ID.")?;
        if !definitions.contains_key(id) {
            return Err(format!("Missing proposal annotation: {id}"));
        }
        used.insert(id.into());
    }
    Ok(())
}
fn edit_text(
    current: &SourceState,
    item: &Value,
    definitions: &BTreeMap<String, Value>,
    used: &mut BTreeSet<String>,
    count: &mut usize,
) -> Result<SourceState, String> {
    let SourceState::File {
        identity,
        text,
        writable: true,
        removable,
        version,
    } = current
    else {
        return Err("Text edits require an acquired, writable source file.".into());
    };
    if let Some(expected) = item
        .pointer("/textDocument/version")
        .filter(|value| !value.is_null())
    {
        let expected = expected
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .ok_or("Malformed provider document version.")?;
        if *version != Some(expected) {
            return Err("Provider source version changed. Synchronize and prepare the whole proposal again.".into());
        }
    }
    let edits = item
        .get("edits")
        .and_then(Value::as_array)
        .ok_or("Missing source edit list.")?;
    *count = count
        .checked_add(edits.len())
        .ok_or("Too many source edits.")?;
    if *count > 10000 {
        return Err("Proposal exceeds 10,000 source replacements. Narrow the change.".into());
    }
    let mut requested = BTreeSet::new();
    let position = |position: &Value| -> Result<(u64, u64), String> {
        Ok((
            position
                .get("line")
                .and_then(Value::as_u64)
                .ok_or("Invalid source line.")?,
            position
                .get("character")
                .and_then(Value::as_u64)
                .ok_or("Invalid UTF-16 source column.")?,
        ))
    };
    let mut replacements = Vec::new();
    for edit in edits {
        annotation(edit, definitions, used)?;
        let from = position(
            edit.pointer("/range/start")
                .ok_or("Missing source range start.")?,
        )?;
        let to = position(
            edit.pointer("/range/end")
                .ok_or("Missing source range end.")?,
        )?;
        let replacement = edit
            .get("newText")
            .and_then(Value::as_str)
            .ok_or("Missing replacement text.")?;
        if from > to {
            return Err("A source replacement runs backwards.".into());
        }
        requested.insert(from);
        requested.insert(to);
        replacements.push((from, to, replacement));
    }
    let mut positions = BTreeMap::new();
    let (mut line, mut column) = (0u64, 0u64);
    for (byte, ch) in text.char_indices() {
        if requested.contains(&(line, column)) {
            positions.insert((line, column), byte);
        }
        if ch == '\n' {
            line += 1;
            column = 0
        } else {
            column += ch.len_utf16() as u64
        }
    }
    if requested.contains(&(line, column)) {
        positions.insert((line, column), text.len());
    }
    if requested.len() != positions.len() {
        return Err(
            "A source range does not fit scalar UTF-16 boundaries. Prepare the proposal again."
                .into(),
        );
    }
    let mut edits: Vec<_> = replacements
        .into_iter()
        .map(|(from, to, replacement)| (positions[&from], positions[&to], replacement))
        .collect();
    edits.sort_by_key(|edit| (edit.0, edit.1));
    for pair in edits.windows(2) {
        if pair[0].1 > pair[1].0 {
            return Err("Overlapping source replacements. No changes were prepared.".into());
        }
    }
    let added: usize = edits
        .iter()
        .map(|edit| edit.2.len())
        .try_fold(0usize, |sum, len| sum.checked_add(len))
        .ok_or("Replacement text is too large.")?;
    if text.len().saturating_add(added) > 16 * 1024 * 1024 {
        return Err(
            "Source draft exceeds the interactive preparation limit. Narrow the change.".into(),
        );
    }
    let mut after = text.clone();
    for (from, to, replacement) in edits.into_iter().rev() {
        after.replace_range(from..to, replacement);
    }
    if after.len() > 8 * 1024 * 1024 {
        return Err("Source draft exceeds 8 MiB. Narrow the change.".into());
    }
    Ok(SourceState::File {
        identity: identity.clone(),
        text: after,
        writable: true,
        removable: *removable,
        version: *version,
    })
}
fn options<'a>(item: &'a Value, allowed: &[&str]) -> Result<&'a Value, String> {
    match item.get("options") {
        None => Ok(&Value::Null),
        Some(value)
            if value
                .as_object()
                .is_some_and(|map| map.keys().all(|key| allowed.contains(&key.as_str()))) =>
        {
            Ok(value)
        }
        _ => Err("Unsupported or malformed resource options. Review the whole proposal.".into()),
    }
}
fn definitions(edit: &Value) -> Result<BTreeMap<String, Value>, String> {
    let Some(raw) = edit.get("changeAnnotations") else {
        return Ok(BTreeMap::new());
    };
    let raw = raw.as_object().ok_or("Malformed proposal annotations.")?;
    if raw.len() > 10000 {
        return Err("Too many proposal annotations.".into());
    }
    raw.iter()
        .map(|(id, value)| {
            let label = value
                .get("label")
                .and_then(Value::as_str)
                .ok_or("A proposal annotation needs its label.")?;
            let needs = boolean(value, "needsConfirmation")?;
            let description = match value.get("description") {
                None => None,
                Some(Value::String(value)) => Some(value.as_str()),
                _ => return Err("Malformed proposal annotation description.".into()),
            };
            if id.len() > 4096
                || label.len() > 4096
                || description.is_some_and(|value| value.len() > 16384)
            {
                return Err("A proposal annotation is too large.".into());
            }
            Ok((
                id.clone(),
                json!({"id":id,"label":label,"description":description,"needs_confirmation":needs}),
            ))
        })
        .collect()
}
fn validate_states(states: &BTreeMap<String, SourceState>) -> Result<(), String> {
    if states.len() > 10000 {
        return Err("Too many source targets. Narrow the proposal.".into());
    }
    let (mut size, mut identities) = (0usize, BTreeSet::new());
    for (path, state) in states {
        validate_path(path)?;
        if path.len() > 4096 {
            return Err("A source path is too long.".into());
        }
        if let SourceState::File { identity, text, .. } = state {
            if identity.is_empty() || identity.len() > 8192 || !identities.insert(identity) {
                return Err(
                    "Ambiguous source identities. Acquire the affected files again.".into(),
                );
            }
            if text.len() > 8 * 1024 * 1024 {
                return Err(format!("Source {path} exceeds 8 MiB. Narrow the change."));
            }
            size = size
                .checked_add(text.len())
                .ok_or("Source set is too large.")?;
        }
    }
    if size > 64 * 1024 * 1024 {
        return Err("Source set exceeds 64 MiB. Narrow the change.".into());
    }
    Ok(())
}
pub fn prepare(request: Request) -> Result<ResourcePlan, String> {
    if request.context.is_empty() || request.context.len() > 4096 {
        return Err(
            "Capture the current workspace/provider context before preparing this proposal.".into(),
        );
    }
    if request.title.len() > 4096 {
        return Err("Proposal title is too long.".into());
    }
    validate_states(&request.sources)?;
    let definitions = definitions(&request.edit)?;
    let mut used = BTreeSet::new();
    let edit = request
        .edit
        .as_object()
        .ok_or("Malformed workspace edit.")?;
    if edit.contains_key("changes") && edit.contains_key("documentChanges") {
        return Err(
            "Provider returned two workspace edit representations. Prepare one complete proposal."
                .into(),
        );
    }
    let operations = if let Some(changes) = edit.get("changes") {
        let changes = changes.as_object().ok_or("Malformed source changes.")?;
        let mut seen = BTreeSet::new();
        let mut operations = Vec::new();
        for (uri, edits) in changes {
            let path = path(uri)?;
            if !seen.insert(path) {
                return Err(
                    "Provider returned aliased source URIs. Prepare one unambiguous proposal."
                        .into(),
                );
            }
            operations.push(json!({"textDocument":{"uri":uri,"version":null},"edits":edits}));
        }
        operations
    } else if let Some(changes) = edit.get("documentChanges") {
        changes
            .as_array()
            .ok_or("Malformed ordered source operations.")?
            .clone()
    } else {
        return Err("The provider returned no workspace edits.".into());
    };
    if operations.len() > 10000 {
        return Err("Proposal exceeds 10,000 operations. Narrow the change.".into());
    }
    let mut states = request.sources.clone();
    let (mut replacements, mut work) = (0usize, 0usize);
    for (index, item) in operations.into_iter().enumerate() {
        if let Some(kind) = item.get("kind") {
            annotation(&item, &definitions, &mut used)?;
            match kind.as_str().ok_or("Malformed resource operation kind.")? {
                "create" => {
                    let destination = source_uri(&item, "uri")?;
                    let opts = options(&item, &["overwrite", "ignoreIfExists"])?;
                    let overwrite = boolean(opts, "overwrite")?;
                    let ignore = boolean(opts, "ignoreIfExists")?;
                    let previous = known(&states, &destination)?;
                    match previous {
                        SourceState::Missing { creatable: true } => {}
                        SourceState::Missing { creatable: false } => {
                            return Err(format!("Destination cannot be created: {destination}"));
                        }
                        SourceState::File { .. } if !overwrite && ignore => continue,
                        SourceState::File { .. } if !overwrite => {
                            return Err(format!("Destination already exists: {destination}"));
                        }
                        SourceState::File { .. } if !can_remove(previous) => {
                            return Err(format!(
                                "Destination is read-only or cannot be replaced: {destination}"
                            ));
                        }
                        SourceState::File { .. } => {}
                    }
                    states.insert(
                        destination.clone(),
                        SourceState::File {
                            identity: format!("created:{index}:{destination}"),
                            text: String::new(),
                            writable: true,
                            removable: true,
                            version: None,
                        },
                    );
                }
                "rename" => {
                    let source = source_uri(&item, "oldUri")?;
                    let destination = source_uri(&item, "newUri")?;
                    let opts = options(&item, &["overwrite", "ignoreIfExists"])?;
                    let overwrite = boolean(opts, "overwrite")?;
                    let ignore = boolean(opts, "ignoreIfExists")?;
                    let previous = known(&states, &source)?.clone();
                    if !can_remove(&previous) {
                        return Err(format!("Source cannot be moved: {source}"));
                    }
                    if source == destination {
                        continue;
                    }
                    match known(&states, &destination)? {
                        SourceState::Missing { creatable: true } => {}
                        SourceState::Missing { creatable: false } => {
                            return Err(format!("Destination cannot be created: {destination}"));
                        }
                        SourceState::File { .. } if !overwrite && ignore => continue,
                        SourceState::File { .. } if !overwrite => {
                            return Err(format!("Destination already exists: {destination}"));
                        }
                        target if !can_remove(target) => {
                            return Err(format!(
                                "Destination is read-only or cannot be replaced: {destination}"
                            ));
                        }
                        _ => {}
                    }
                    states.insert(source, SourceState::Missing { creatable: true });
                    states.insert(destination, previous);
                }
                "delete" => {
                    let source = source_uri(&item, "uri")?;
                    let opts = options(&item, &["recursive", "ignoreIfNotExists"])?;
                    boolean(opts, "recursive")?;
                    let ignore = boolean(opts, "ignoreIfNotExists")?;
                    match known(&states, &source)? {
                        SourceState::Missing { .. } if ignore => continue,
                        state if can_remove(state) => {}
                        _ => {
                            return Err(format!(
                                "Source is absent, read-only or cannot be deleted: {source}"
                            ));
                        }
                    }
                    states.insert(source, SourceState::Missing { creatable: true });
                }
                _ => return Err("Unsupported resource operation. No changes were prepared.".into()),
            }
        } else {
            let source = path(
                item.pointer("/textDocument/uri")
                    .and_then(Value::as_str)
                    .ok_or("Missing text document source identity.")?,
            )?;
            let previous = known(&states, &source)?;
            if let SourceState::File { text, .. } = previous {
                work = work.saturating_add(text.len());
                if work > 128 * 1024 * 1024 {
                    return Err("Repeated source transformations exceed the interactive preparation limit. Narrow the change.".into());
                }
            }
            let after = edit_text(previous, &item, &definitions, &mut used, &mut replacements)?;
            states.insert(source, after);
        }
    }
    validate_states(&states)?;
    let patches = request
        .sources
        .into_iter()
        .filter_map(|(path, before)| {
            let after = states
                .remove(&path)
                .expect("all operations require captured paths");
            (before != after).then_some(ResourcePatch {
                path,
                before,
                after,
            })
        })
        .collect();
    Ok(ResourcePlan {
        version: 1,
        context: request.context,
        title: request.title,
        patches,
        annotations: used
            .into_iter()
            .map(|id| definitions[&id].clone())
            .collect(),
    })
}
impl ResourcePlan {
    pub fn inverse(&self) -> Self {
        Self {
            version: self.version,
            context: self.context.clone(),
            title: format!("Undo {}", self.title),
            patches: self
                .patches
                .iter()
                .map(|patch| ResourcePatch {
                    path: patch.path.clone(),
                    before: patch.after.clone(),
                    after: patch.before.clone(),
                })
                .collect(),
            annotations: self.annotations.clone(),
        }
    }
    /// Validate every snapshot before returning a new draft map. Never mutates the caller.
    pub fn apply(
        &self,
        context: &str,
        current: &BTreeMap<String, SourceState>,
    ) -> Result<BTreeMap<String, SourceState>, String> {
        if self.context != context {
            return Err(
                "Workspace or provider context changed. Prepare the whole proposal again.".into(),
            );
        }
        validate_states(current)?;
        if self.version != 1 {
            return Err("Unsupported resource plan version.".into());
        }
        let mut seen = BTreeSet::new();
        for patch in &self.patches {
            validate_path(&patch.path)?;
            if !seen.insert(&patch.path)
                || !current
                    .get(&patch.path)
                    .is_some_and(|state| same_draft(state, &patch.before))
                || !allowed_transition(&patch.before, &patch.after)
            {
                return Err(format!(
                    "Source or destination changed: {}. Prepare the whole proposal again.",
                    patch.path
                ));
            }
        }
        let mut after = current.clone();
        for patch in &self.patches {
            after.insert(patch.path.clone(), patch.after.clone());
        }
        validate_states(&after)?;
        Ok(after)
    }
}
pub async fn inspect(
    axum::Json(request): axum::Json<Request>,
) -> super::api::ApiResult<axum::Json<Value>> {
    tokio::task::spawn_blocking(move || prepare(request))
        .await
        .map_err(|error| {
            super::api::ApiError::internal(format!(
                "Resource proposal preparation failed: {error}. Prepare it again."
            ))
        })?
        .map(|plan| axum::Json(json!({"plan":plan,"applied":false,"saved":false,"snapshot_authority":"provided_drafts_only"})))
        .map_err(super::api::ApiError::unprocessable)
}

fn same_draft(left: &SourceState, right: &SourceState) -> bool {
    match (left, right) {
        (
            SourceState::File {
                identity: a,
                text: b,
                writable: c,
                removable: d,
                ..
            },
            SourceState::File {
                identity: e,
                text: f,
                writable: g,
                removable: h,
                ..
            },
        ) => a == e && b == f && c == g && d == h,
        _ => left == right,
    }
}

fn allowed_transition(before: &SourceState, after: &SourceState) -> bool {
    match (before, after) {
        (SourceState::Missing { creatable: a }, SourceState::Missing { creatable: b }) => a == b,
        (
            SourceState::Missing { creatable: true },
            SourceState::File {
                writable: true,
                removable: true,
                ..
            },
        ) => true,
        (SourceState::File { .. }, SourceState::Missing { creatable: true }) => can_remove(before),
        (
            SourceState::File {
                identity: a,
                text: b,
                writable: c,
                removable: d,
                ..
            },
            SourceState::File {
                identity: e,
                text: f,
                writable: g,
                removable: h,
                ..
            },
        ) => {
            if a == e {
                c == g && d == h && (b == f || *c)
            } else {
                can_remove(before) && *g && *h
            }
        }
        _ => false,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn file(id: &str, text: &str) -> SourceState {
        SourceState::File {
            identity: id.into(),
            text: text.into(),
            writable: true,
            removable: true,
            version: Some(7),
        }
    }
    fn missing() -> SourceState {
        SourceState::Missing { creatable: true }
    }
    fn request(edit: Value, sources: BTreeMap<String, SourceState>) -> Request {
        Request {
            context: "fixture".into(),
            title: "Fixture proposal".into(),
            edit,
            sources,
        }
    }
    fn text(uri: &str, version: Value, from: u64, to: u64, new: &str) -> Value {
        json!({"textDocument":{"uri":uri,"version":version},"edits":[{"range":{"start":{"line":0,"character":from},"end":{"line":0,"character":to}},"newText":new}]})
    }
    #[test]
    fn ordered_resources_text_and_annotations_form_one_reversible_draft_plan() {
        let sources = BTreeMap::from([
            ("a.py".into(), file("a", "😀n=1\n")),
            ("b.py".into(), missing()),
            ("new.py".into(), missing()),
            ("old.py".into(), file("old", "retained for undo")),
        ]);
        let edit = json!({"changeAnnotations":{"move":{"label":"Move source","needsConfirmation":true,"description":"Retain the original document identity"}},"documentChanges":[{"kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py","annotationId":"move"},text("hick:///b.py",json!(7),2,3,"m"),{"kind":"create","uri":"hick:///new.py"},text("hick:///new.py",Value::Null,0,0,"new"),{"kind":"delete","uri":"hick:///old.py"}]});
        let plan = prepare(request(edit, sources.clone())).unwrap();
        assert_eq!(plan.patches.len(), 4);
        assert_eq!(plan.annotations[0]["needs_confirmation"], true);
        assert!(plan.apply("different-workspace", &sources).is_err());
        let mut after = plan.apply("fixture", &sources).unwrap();
        assert!(
            matches!(&after["b.py"],SourceState::File{identity,text,..} if identity=="a"&&text=="😀m=1\n")
        );
        assert_eq!(after["a.py"], missing());
        assert_eq!(after["old.py"], missing());
        if let SourceState::File { version, .. } = after.get_mut("b.py").unwrap() {
            *version = Some(8);
        }
        assert_eq!(plan.inverse().apply("fixture", &after).unwrap(), sources);
        let mut stale = sources.clone();
        if let SourceState::File { text, .. } = stale.get_mut("old.py").unwrap() {
            *text = "external draft".into();
        }
        let before = stale.clone();
        assert!(plan.apply("fixture", &stale).is_err());
        assert_eq!(stale, before);
    }
    #[test]
    fn later_invalid_operations_refuse_without_mutating_any_input() {
        let sources = BTreeMap::from([("a.py".into(), file("a", "x")), ("b.py".into(), missing())]);
        let original = sources.clone();
        let edit = json!({"documentChanges":[{"kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py"},text("hick:///b.py",json!(99),0,1,"wrong version")]});
        assert!(
            prepare(request(edit, sources.clone()))
                .unwrap_err()
                .contains("version")
        );
        assert_eq!(sources, original);
        let unknown = json!({"documentChanges":[{"kind":"create","uri":"hick:///uncaptured.py"}]});
        assert!(
            prepare(request(unknown, sources.clone()))
                .unwrap_err()
                .contains("confirmed absence")
        );
        let mut readonly = sources.clone();
        if let SourceState::File { writable, .. } = readonly.get_mut("a.py").unwrap() {
            *writable = false;
        }
        assert!(
            prepare(request(
                json!({"documentChanges":[{"kind":"delete","uri":"hick:///a.py"}]}),
                readonly
            ))
            .is_err()
        );
    }
    #[test]
    fn overwrite_and_ignore_options_have_explicit_whole_plan_semantics() {
        let sources = BTreeMap::from([
            ("a.py".into(), file("a", "A")),
            ("b.py".into(), file("b", "B")),
        ]);
        let rename = |options| json!({"documentChanges":[{"kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py","options":options}]});
        assert!(prepare(request(rename(json!({})), sources.clone())).is_err());
        assert!(
            prepare(request(
                rename(json!({"ignoreIfExists":true})),
                sources.clone()
            ))
            .unwrap()
            .patches
            .is_empty()
        );
        let plan = prepare(request(
            rename(json!({"overwrite":true,"ignoreIfExists":true})),
            sources.clone(),
        ))
        .unwrap();
        let after = plan.apply("fixture", &sources).unwrap();
        assert!(matches!(&after["b.py"],SourceState::File{text,..} if text=="A"));
        assert_eq!(plan.inverse().apply("fixture", &after).unwrap(), sources);
        assert!(prepare(request(rename(json!({"overwrite":"yes"})), sources.clone())).is_err());
        assert!(prepare(request(json!({"documentChanges":[{"kind":"delete","uri":"hick:///gone.py","options":{"ignoreIfNotExists":true}}]}),BTreeMap::from([("gone.py".into(),missing())]))).unwrap().patches.is_empty());
    }
    #[test]
    fn scalar_ranges_and_ordered_same_point_inserts_preserve_provider_order() {
        let sources = BTreeMap::from([("a.py".into(), file("a", "😀x"))]);
        assert!(
            prepare(request(
                json!({"documentChanges":[text("hick:///a.py",json!(7),1,2,"split")]}),
                sources
            ))
            .is_err()
        );
        let mut edit = text("hick:///a.py", json!(7), 0, 0, "a");
        let mut second = edit["edits"][0].clone();
        second["newText"] = json!("b");
        let mut replacement = second.clone();
        replacement["range"]["end"]["character"] = json!(1);
        replacement["newText"] = json!("R");
        edit["edits"]
            .as_array_mut()
            .unwrap()
            .extend([second, replacement]);
        let sources = BTreeMap::from([("a.py".into(), file("a", "x"))]);
        let plan = prepare(request(json!({"documentChanges":[edit]}), sources.clone())).unwrap();
        assert!(
            matches!(&plan.apply("fixture", &sources).unwrap()["a.py"],SourceState::File{text,..} if text=="abR")
        );
        let edit = json!({"documentChanges":[{"textDocument":{"uri":"hick:///a.py","version":7},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"newText":"A"},{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"newText":"B"}]}]});
        assert!(prepare(request(edit, sources)).is_err());
    }
    #[test]
    fn literal_uri_identity_and_confirmed_source_identity_do_not_alias() {
        assert_eq!(path("hick:///caf%C3%A9%2520.py").unwrap(), "café%20.py");
        for uri in [
            "hick:///../x.py",
            "hick:///%2e%2e/x.py",
            "hick:///.GIT/config",
            "hick:///a%5cb.py",
            "hick:///a%00b.py",
            "hick:///a%GG.py",
            "hick:///a.py?revision=old",
            "file:///a.py",
        ] {
            assert!(path(uri).is_err(), "{uri}");
        }
        let sources = BTreeMap::from([
            ("a.py".into(), file("same", "A")),
            ("b.py".into(), file("same", "B")),
        ]);
        assert!(
            prepare(request(json!({"changes":{}}), sources))
                .unwrap_err()
                .contains("identities")
        );
        let sources = BTreeMap::from([("a.py".into(), file("a", "A"))]);
        let edit = json!({"changes":{"hick:///a.py":[],"hick:///%61.py":[]}});
        assert!(
            prepare(request(edit, sources))
                .unwrap_err()
                .contains("aliased")
        );
    }
    #[test]
    fn missing_annotations_and_expanding_text_budgets_refuse_the_whole_plan() {
        let sources = BTreeMap::from([("a.py".into(), file("a", "x"))]);
        let edit = json!({"documentChanges":[{"kind":"delete","uri":"hick:///a.py","annotationId":"missing"}]});
        assert!(
            prepare(request(edit, sources))
                .unwrap_err()
                .contains("annotation")
        );
        let sources = BTreeMap::from([("a.py".into(), file("a", &"x".repeat(8 * 1024 * 1024)))]);
        let edit = json!({"documentChanges":[text("hick:///a.py",json!(7),0,0,"more")]});
        assert!(
            prepare(request(edit, sources))
                .unwrap_err()
                .contains("8 MiB")
        );
    }
    #[test]
    fn applying_a_deserialized_plan_cannot_promote_capabilities() {
        let mut readonly = file("a", "x");
        if let SourceState::File { writable, .. } = &mut readonly {
            *writable = false;
        }
        let states = BTreeMap::from([("a.py".into(), readonly.clone())]);
        let mut promoted = readonly.clone();
        if let SourceState::File { writable, .. } = &mut promoted {
            *writable = true;
        }
        let plan = ResourcePlan {
            version: 1,
            context: "fixture".into(),
            title: "untrusted".into(),
            patches: vec![ResourcePatch {
                path: "a.py".into(),
                before: readonly.clone(),
                after: promoted,
            }],
            annotations: vec![],
        };
        assert!(plan.apply("fixture", &states).is_err());
        let mut deletion = plan.clone();
        deletion.patches[0].after = missing();
        assert!(deletion.apply("fixture", &states).is_err());
        let states = BTreeMap::from([("new.py".into(), SourceState::Missing { creatable: false })]);
        let plan = ResourcePlan {
            version: 1,
            context: "fixture".into(),
            title: "untrusted".into(),
            patches: vec![ResourcePatch {
                path: "new.py".into(),
                before: SourceState::Missing { creatable: false },
                after: file("new", "created"),
            }],
            annotations: vec![],
        };
        assert!(plan.apply("fixture", &states).is_err());
    }
}
/// Discover the full literal source/destination read set before acquiring any bytes.
pub fn targets(edit: &Value) -> Result<Vec<String>, String> {
    let object = edit.as_object().ok_or("Malformed workspace edit.")?;
    if object.contains_key("changes") && object.contains_key("documentChanges") {
        return Err("Provider returned two workspace edit representations.".into());
    }
    let mut targets = BTreeSet::new();
    if let Some(changes) = object.get("changes") {
        for (uri, edits) in changes.as_object().ok_or("Malformed source changes.")? {
            if !edits.is_array() {
                return Err("Malformed source edit list.".into());
            }
            let source = path(uri)?;
            if !targets.insert(source) {
                return Err("Provider returned aliased source URIs.".into());
            }
        }
    } else if let Some(changes) = object.get("documentChanges") {
        let changes = changes
            .as_array()
            .ok_or("Malformed ordered source operations.")?;
        if changes.len() > 10000 {
            return Err("Proposal exceeds 10,000 operations.".into());
        }
        for item in changes {
            match item.get("kind") {
                Some(Value::String(kind)) if kind == "create" || kind == "delete" => {
                    targets.insert(source_uri(item, "uri")?);
                }
                Some(Value::String(kind)) if kind == "rename" => {
                    targets.insert(source_uri(item, "oldUri")?);
                    targets.insert(source_uri(item, "newUri")?);
                }
                None => {
                    if !item.get("edits").is_some_and(Value::is_array) {
                        return Err("Missing source edit list.".into());
                    }
                    targets.insert(path(
                        item.pointer("/textDocument/uri")
                            .and_then(Value::as_str)
                            .ok_or("Missing text document source identity.")?,
                    )?);
                }
                _ => return Err("Unsupported resource operation. No sources were acquired.".into()),
            }
            if targets.len() > 10000 {
                return Err("Proposal exceeds 10,000 source targets.".into());
            }
        }
    } else {
        return Err("The provider returned no workspace edits.".into());
    }
    if targets.len() > 10000 || targets.iter().map(String::len).sum::<usize>() > 4 * 1024 * 1024 {
        return Err("Resource target set is too large. Narrow the change.".into());
    }
    Ok(targets.into_iter().collect())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetRequest {
    pub edit: Value,
}
pub async fn inspect_targets(
    axum::Json(request): axum::Json<TargetRequest>,
) -> super::api::ApiResult<axum::Json<Value>> {
    tokio::task::spawn_blocking(move || targets(&request.edit))
        .await
        .map_err(|error| {
            super::api::ApiError::internal(format!(
                "Resource target discovery failed: {error}. Prepare it again."
            ))
        })?
        .map(|paths| axum::Json(json!({"version":1,"paths":paths,"acquired":false,"saved":false})))
        .map_err(super::api::ApiError::unprocessable)
}
#[cfg(test)]
#[test]
fn resource_target_discovery_captures_temporary_destinations_and_rejects_ambiguous_addresses() {
    let edit = json!({"documentChanges":[{"kind":"create","uri":"hick:///new.py"},{"kind":"rename","oldUri":"hick:///a%2520.py","newUri":"hick:///new.py"},{"kind":"delete","uri":"hick:///gone.py"},{"textDocument":{"uri":"hick:///new.py","version":null},"edits":[]}]});
    assert_eq!(
        targets(&edit).unwrap(),
        vec!["a%20.py", "gone.py", "new.py"]
    );
    assert!(targets(&json!({"changes":{"hick:///a.py":[],"hick:///%61.py":[]}})).is_err());
    assert!(
        targets(&json!({"documentChanges":[{"kind":"delete","uri":"file:///outside.py"}]}))
            .is_err()
    );
    assert!(
        targets(&json!({"documentChanges":[{"kind":"unknown","uri":"hick:///a.py"}]})).is_err()
    );
}
