//! Read-only test facts over exact supplied source and compiler-owned projections.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{Json, extract::State};
use hick_plugins::testing::candidates::{self, Confidence};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Component, Path, PathBuf},
};
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub path: String,
    pub source: String,
    /// An explicit construction entry; absent means only this document's local scope.
    pub project_entry: Option<String>,
    #[serde(default)]
    pub sources: BTreeMap<String, String>,
    /// Register the complete compiler output tree in memory only when explicitly requested.
    #[serde(default)]
    pub capture: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct SourcePoint {
    pub path: String,
    pub line: u32,
    pub character: u32,
    pub hash: String,
}
#[derive(Debug, Serialize)]
pub struct Candidate {
    pub id: String,
    pub language: String,
    pub run_scope: hick_plugins::testing::Scope,
    pub name: String,
    pub name_filter: Option<String>,
    pub confidence: Confidence,
    pub source: Option<SourcePoint>,
    pub target: SourcePoint,
}
#[derive(Serialize)]
pub struct Scan {
    pub version: u32,
    pub workspace: String,
    pub workspace_id: String,
    pub input_path: String,
    pub input_hash: String,
    pub scope: String,
    pub project_entry: Option<String>,
    pub fingerprint: String,
    pub source_hashes: BTreeMap<String, String>,
    pub candidates: Vec<Candidate>,
    pub syntax_complete: bool,
    pub notices: Vec<String>,
    pub unsupported_languages: Vec<String>,
    pub unclassified_outputs: Vec<String>,
    pub scanned_files: usize,
    pub unused_sources: Vec<String>,
    pub project_context: Option<Value>,
    #[serde(skip)]
    pub(crate) tree: Option<hick_lang::ProjectTree>,
}
pub(crate) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn relative(base: &Path, path: &str) -> Result<String, String> {
    if path.is_empty()
        || path.contains('\\')
        || path.contains(':')
        || path.chars().any(char::is_control)
        || Path::new(path).is_absolute()
    {
        return Err("Use a relative path inside the workspace.".into());
    }
    let mut parts = vec![];
    for part in base.join(path).components() {
        match part {
            Component::Normal(value) => parts.push(value.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                if parts.pop().is_none() {
                    return Err(
                        "Source path escapes the workspace. Choose a workspace source.".into(),
                    );
                }
            }
            _ => return Err("Source path must be inside the workspace.".into()),
        }
    }
    let result = parts.join("/");
    hick_lang::validate_project_path(&result)?;
    Ok(result)
}
fn saved_source(root: &Path, path: &str) -> Result<String, String> {
    let joined = root.join(path);
    if !std::fs::symlink_metadata(&joined)
        .map_err(|e| {
            format!("Cannot read source {path}: {e}. Save it or supply its current draft.")
        })?
        .is_file()
    {
        return Err(format!(
            "Source {path} must be a regular file. Choose or supply a source page."
        ));
    }
    let actual = joined
        .canonicalize()
        .map_err(|e| format!("Cannot resolve source {path}: {e}. Reload the source."))?;
    if !actual.starts_with(root) {
        return Err(format!(
            "Source {path} escapes the workspace. Choose a workspace source."
        ));
    }
    let mut value = String::new();
    std::fs::File::open(actual)
        .map_err(|e| e.to_string())?
        .take(8 * 1024 * 1024 + 1)
        .read_to_string(&mut value)
        .map_err(|e| format!("Cannot read UTF-8 source {path}: {e}. Check its encoding."))?;
    if value.len() > 8 * 1024 * 1024 {
        return Err(format!(
            "Source {path} exceeds 8 MiB. Narrow the entry or use runner discovery."
        ));
    }
    Ok(value)
}
fn point(path: &str, line: u32, character: u32, hash: &str) -> SourcePoint {
    SourcePoint {
        path: path.into(),
        line,
        character,
        hash: hash.into(),
    }
}
fn candidate(
    language: &str,
    item: candidates::TestCandidate,
    source: Option<SourcePoint>,
    target: SourcePoint,
) -> Result<Candidate, String> {
    let run_scope = hick_plugins::testing::scope(language)?;
    let id = digest(
        &serde_json::to_vec(&json!([
            language,
            run_scope,
            source,
            target,
            item.name,
            item.name_filter
        ]))
        .unwrap(),
    );
    Ok(Candidate {
        id,
        run_scope,
        language: language.into(),
        name: item.name,
        name_filter: item.name_filter,
        confidence: item.confidence,
        source,
        target,
    })
}
fn base_scan(request: &Request, path: &str, root: &Path) -> Scan {
    let hash = digest(request.source.as_bytes());
    Scan {
        version: 1,
        workspace: root.to_string_lossy().into_owned(),
        workspace_id: digest(root.as_os_str().as_encoded_bytes()),
        input_path: path.into(),
        input_hash: hash.clone(),
        scope: "plain".into(),
        project_entry: None,
        fingerprint: String::new(),
        source_hashes: BTreeMap::from([(path.into(), hash)]),
        candidates: vec![],
        syntax_complete: true,
        notices: vec![],
        unsupported_languages: vec![],
        unclassified_outputs: vec![],
        scanned_files: 0,
        unused_sources: vec![],
        project_context: None,
        tree: None,
    }
}
fn seal(mut scan: Scan) -> Scan {
    scan.candidates.sort_by(|a, b| {
        a.target
            .path
            .cmp(&b.target.path)
            .then(a.target.line.cmp(&b.target.line))
            .then(a.target.character.cmp(&b.target.character))
            .then(a.id.cmp(&b.id))
    });
    scan.candidates.dedup_by(|a, b| a.id == b.id);
    scan.notices.sort();
    scan.notices.dedup();
    scan.unsupported_languages.sort();
    scan.unsupported_languages.dedup();
    scan.fingerprint=digest(&serde_json::to_vec(&json!({"version":scan.version,"workspace":scan.workspace_id,"scope":scan.scope,"entry":scan.project_entry,"sources":scan.source_hashes,"targets":scan.candidates.iter().map(|c|(&c.id,&c.target.path,&c.target.hash,&c.run_scope)).collect::<Vec<_>>()})).unwrap());
    scan
}
fn project_state(
    root: &Path,
    request: &Request,
    path: &str,
    entry: &str,
) -> Result<(hick_lsp::document::HickDocumentState, PathBuf, Vec<String>), String> {
    let entry = relative(Path::new(""), entry)?;
    let base = Path::new(&entry).parent().unwrap_or(Path::new(""));
    let name = Path::new(&entry)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Choose a UTF-8 construction entry.")?;
    let mut supplied = BTreeMap::new();
    for (path, value) in &request.sources {
        let normalized = relative(Path::new(""), path)?;
        if value.len() > 8 * 1024 * 1024 {
            return Err(format!(
                "Draft {path} exceeds 8 MiB. Narrow the selected source scope."
            ));
        }
        supplied.insert(normalized, value.clone());
    }
    supplied.insert(path.into(), request.source.clone());
    let observed = std::cell::RefCell::new((BTreeSet::new(), 0usize));
    let sources = hick_lang::load_project_sources(name, &|logical| {
        let key = relative(base, logical)?;
        let value = if let Some(value) = supplied.get(&key) {
            value.clone()
        } else {
            saved_source(root, &key)?
        };
        let mut observed = observed.borrow_mut();
        if observed.0.insert(key) {
            observed.1 += value.len();
            if observed.0.len() > 10000 || observed.1 > 128 * 1024 * 1024 {
                return Err("Construction source set is too large. Narrow the entry or use runner discovery.".into());
            }
        }
        Ok(value)
    })?;
    let observed = observed.into_inner().0;
    if !observed.contains(path) {
        return Err(format!(
            "{path} is not in the selected construction entry. Choose a contributing source page or another entry."
        ));
    }
    let unused = supplied
        .keys()
        .filter(|key| !observed.contains(*key))
        .cloned()
        .collect();
    let state=hick_lsp::document::HickDocumentState::from_project(name,&sources).map_err(|e|format!("Construction analysis failed: {e}. Correct the selected source or choose another entry."))?;
    Ok((state, base.to_path_buf(), unused))
}
/// Collect source facts only. No test planner, runner or materializer is called.
pub(crate) fn scan(root: &Path, request: &Request) -> Result<Scan, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("Workspace unavailable: {e}. Reconnect and retry."))?;
    let path = relative(Path::new(""), &request.path)?;
    if request.source.len() > 8 * 1024 * 1024
        || request.sources.len() > 4096
        || request.sources.values().map(String::len).sum::<usize>() > 32 * 1024 * 1024
    {
        return Err(
            "Source request is too large. Narrow the supplied source set or use runner discovery."
                .into(),
        );
    }
    let language=hick_plugins::routing::language_id(&path).ok_or_else(||format!("No source language for {path}. Save with a supported extension or use project runner discovery."))?;
    let mut result = base_scan(request, &path, &root);
    if language != "markdown" && request.project_entry.is_none() {
        let found = candidates::detect(language, &request.source)?;
        result.syntax_complete = found.syntax_complete;
        result.notices = found.notices;
        result.scanned_files = 1;
        for item in found.candidates {
            let at = point(&path, item.line, item.character, &result.input_hash);
            result
                .candidates
                .push(candidate(language, item, Some(at.clone()), at)?);
        }
        return Ok(seal(result));
    }
    let parsed = hick_lang::parse(&request.source).map_err(|e| {
        format!("Source construction cannot be read: {e}. Correct it or use runner discovery.")
    })?;
    let explicit = request
        .project_entry
        .clone()
        .or_else(|| (!parsed.find_tags("project").is_empty()).then(|| path.clone()));
    let (state, base, unused) = if let Some(entry) = &explicit {
        let (state, base, unused) = project_state(&root, request, &path, entry)?;
        result.scope = "project".into();
        result.project_entry = Some(relative(Path::new(""), entry)?);
        (state, base, unused)
    } else {
        result.scope = "document".into();
        result.notices.push("Only this document's local constructions are analyzed. Select a project entry to resolve linked source pages.".into());
        (hick_lsp::document::HickDocumentState::from_source(&request.source).map_err(|e|format!("Local construction analysis failed: {e}. Select a project entry or correct the source."))?,Path::new(&path).parent().unwrap_or(Path::new("")).to_path_buf(),request.sources.keys().cloned().collect())
    };
    result.unused_sources = unused;
    if state.selected_project.is_some() {
        result.source_hashes.clear();
        for (logical, source) in &state.project_sources {
            result
                .source_hashes
                .insert(relative(&base, logical)?, digest(source.as_bytes()));
        }
    }
    let mut unsupported = BTreeSet::new();
    for file in &state.virtual_files {
        let Some(language) = file.language_id else {
            result.unclassified_outputs.push(file.path.clone());
            continue;
        };
        if !matches!(
            hick_plugins::registry().interface(language, hick_plugins::InterfaceKind::TestRunner),
            Some(hick_plugins::Interface::TestRunner {
                detect: Some(_),
                ..
            })
        ) {
            unsupported.insert(language.to_string());
            continue;
        }
        let project = state.selected_project.is_some();
        let content = if project {
            file.written_content()
        } else {
            file.content()
        };
        let map = state
            .project_maps
            .get(&file.path)
            .cloned()
            .unwrap_or_else(|| hick_lsp::position_map::PositionMap::build(&file.segments));
        let target_path = relative(&base, &file.path)?;
        let target_hash = digest(file.written_content().as_bytes());
        let found = match candidates::detect(language, &content) {
            Ok(value) => value,
            Err(error) => {
                result.syntax_complete = false;
                result.notices.push(format!("{}: {error}", file.path));
                continue;
            }
        };
        result.scanned_files += 1;
        result.syntax_complete &= found.syntax_complete;
        result.notices.extend(
            found
                .notices
                .into_iter()
                .map(|notice| format!("{}: {notice}", file.path)),
        );
        for item in found.candidates {
            let source = map.source_document(item.line - 1, item.character).and_then(
                |(owner, line, character)| {
                    let owner = if let Some(owner) = owner {
                        relative(&base, &owner).ok()?
                    } else {
                        path.clone()
                    };
                    let hash = result.source_hashes.get(&owner)?;
                    Some(point(&owner, line + 1, character, hash))
                },
            );
            let saved_line = item.line
                - if !project && content.starts_with('\n') {
                    1
                } else {
                    0
                };
            if saved_line == 0 {
                result.notices.push(format!(
                    "{} has a candidate in synthesized text; no runtime line was invented.",
                    file.path
                ));
                continue;
            }
            let character = item.character;
            result.candidates.push(candidate(
                language,
                item,
                source,
                point(&target_path, saved_line, character, &target_hash),
            )?);
        }
    }
    result.unsupported_languages = unsupported.into_iter().collect();
    result.tree = state.selected_project.clone();
    if result.candidates.len() > 10000 {
        return Err("More than 10,000 source test candidates. Narrow the selected entry or use runner discovery.".into());
    }
    Ok(seal(result))
}
/// Supplied source fingerprints are checked separately from execution's target fingerprint.
pub(crate) fn check_saved_sources(
    root: &Path,
    expected: &BTreeMap<String, String>,
) -> ApiResult<BTreeMap<String, String>> {
    let root = root.canonicalize().map_err(|e| {
        ApiError::unprocessable(format!("Workspace unavailable: {e}. Reconnect and retry."))
    })?;
    let mut checked = BTreeMap::new();
    for (path, hash) in expected {
        let path = relative(Path::new(""), path).map_err(ApiError::bad_request)?;
        if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(ApiError::bad_request(format!(
                "Source {path} needs a SHA-256 fingerprint. Refresh its test candidates."
            )));
        }
        let source = saved_source(&root, &path).map_err(ApiError::unprocessable)?;
        let actual = digest(source.as_bytes());
        if !actual.eq_ignore_ascii_case(hash) {
            return Err(ApiError::conflict(format!(
                "Source {path} changed or has unsaved edits. Save/reload and refresh the test candidates before running."
            )));
        }
        checked.insert(path, actual);
    }
    Ok(checked)
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRequest {
    pub version: u32,
    pub input: RunInput,
    pub analysis: Request,
    pub fingerprint: String,
    pub candidate_id: String,
    pub selection: Selection,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RunInput {
    Saved {},
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Selection {
    ContainingScope {},
    NameFilter {},
}
/// Recompute the source model before selecting a saved target. Older engines
/// have no such route, so an unsupported admission protocol cannot fall back.
pub async fn run(
    State(state): State<LocalState>,
    Json(request): Json<RunRequest>,
) -> ApiResult<Json<Value>> {
    if request.version != 1 {
        return Err(ApiError::conflict(
            "Unsupported source-test protocol. Update the app and engine together.",
        ));
    }
    match request.input {
        RunInput::Saved {} => {}
    }
    let root = state.index.root().to_path_buf();
    let scan = tokio::task::spawn_blocking(move || scan(&root, &request.analysis))
        .await
        .map_err(|e| {
            ApiError::internal(format!(
                "Source test analysis failed: {e}. Refresh and retry."
            ))
        })?
        .map_err(ApiError::unprocessable)?;
    if !scan.fingerprint.eq_ignore_ascii_case(&request.fingerprint) {
        return Err(ApiError::conflict(
            "The analyzed source set changed. Refresh test candidates before running.",
        ));
    }
    let candidate = scan
        .candidates
        .iter()
        .find(|item| item.id == request.candidate_id)
        .ok_or_else(|| {
            ApiError::conflict(
                "This source candidate changed. Refresh candidates or use runner discovery.",
            )
        })?;
    let name=match request.selection {Selection::ContainingScope {}=>String::new(),Selection::NameFilter {}=>candidate.name_filter.clone().ok_or_else(||ApiError::unprocessable("This candidate needs runner discovery. Discover concrete cases or explicitly run the containing scope."))?};
    let body = super::test_run::RunBody {
        path: candidate.target.path.clone(),
        name,
        language: candidate.language.clone(),
        expected_source_hash: Some(candidate.target.hash.clone()),
        expected_sources: scan.source_hashes.clone(),
    };
    let mut result = super::test_run::run(State(state), Json(body)).await?.0;
    result["version"] = json!(1);
    result["analysis_fingerprint"] = json!(scan.fingerprint);
    result["candidate_id"] = json!(candidate.id);
    result["candidate_name"] = json!(candidate.name);
    result["selection"] = json!(request.selection);
    Ok(Json(result))
}
pub async fn inspect(
    State(state): State<LocalState>,
    Json(request): Json<Request>,
) -> ApiResult<Json<Scan>> {
    let root = state.index.root().to_path_buf();
    let capture = request.capture;
    let mut scan = tokio::task::spawn_blocking(move || scan(&root, &request))
        .await
        .map_err(|e| {
            ApiError::internal(format!(
                "Source test analysis failed: {e}. Retry or narrow the source."
            ))
        })?
        .map_err(ApiError::unprocessable)?;
    if capture {
        let tree=scan.tree.take().filter(|tree|!tree.entries().is_empty()).ok_or_else(||ApiError::unprocessable("Choose a construction entry with executable outputs before capturing a test project."))?;
        let id = format!("{:016x}", super::rand_id());
        let uri = state
            .lsp
            .projects()
            .register(&id, tree)
            .map_err(ApiError::unprocessable)?;
        scan.project_context =
            Some(json!({"id":id,"uri":uri,"immutable":true,"materialized":false}));
    } else {
        scan.tree = None;
    }
    Ok(Json(scan))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_test_scan_is_read_only_and_binds_supplied_text() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let source = format!(
            "from pathlib import Path\nPath({:?}).write_text('executed')\ndef test_present():\n    pass\n",
            root.join("executed").to_string_lossy()
        );
        let request = Request {
            path: "tests/test_present.py".into(),
            source: source.clone(),
            project_entry: None,
            sources: BTreeMap::new(),
            capture: false,
        };
        let result = scan(&root, &request).unwrap();
        assert_eq!(result.scope, "plain");
        assert_eq!(result.input_hash, digest(source.as_bytes()));
        assert_eq!(result.candidates.len(), 1);
        let item = &result.candidates[0];
        assert_eq!(item.name, "test_present");
        assert_eq!(item.source.as_ref().unwrap().path, request.path);
        assert_eq!(item.target.hash, result.input_hash);
        assert!(!root.join("executed").exists());
        assert!(!root.join("tests").exists());
        assert!(result.tree.is_none());
    }
    #[test]
    fn source_test_local_document_keeps_authored_and_runtime_lines_distinct() {
        let temp = tempfile::tempdir().unwrap();
        let source = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n<h:file path=\"test_case.py\">\ndef test_local():\n    pass\n</h:file>\n</h:doc>\n";
        let request = Request {
            path: "note.md".into(),
            source: source.into(),
            project_entry: None,
            sources: BTreeMap::new(),
            capture: false,
        };
        let result = scan(temp.path(), &request).unwrap();
        assert_eq!(result.scope, "document");
        let item = &result.candidates[0];
        assert_eq!(item.source.as_ref().unwrap().path, "note.md");
        assert_eq!(item.source.as_ref().unwrap().line, 3);
        assert_eq!(item.target.line, 1);
        assert_eq!(item.target.character, 4);
        assert_eq!(item.target.hash, digest(b"def test_local():\n    pass\n"));
        assert!(!temp.path().join("test_case.py").exists());
    }
    #[test]
    fn source_test_project_retains_foreign_contributors_and_guards_their_saved_bytes() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let entry = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"cases.md\"/></h:doc>";
        let saved = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n<h:copy id=\"cases\">\ndef test_saved():\n    pass\n</h:copy>\n<h:file path=\"tests/test_case.py\"><h:paste select=\"#cases\"/></h:file>\n</h:doc>";
        let draft = saved.replace("test_saved", "test_draft");
        std::fs::write(root.join("entry.md"), entry).unwrap();
        std::fs::write(root.join("cases.md"), saved).unwrap();
        let request = Request {
            path: "cases.md".into(),
            source: draft.clone(),
            project_entry: Some("entry.md".into()),
            sources: BTreeMap::from([("unused.md".into(), "unused".into())]),
            capture: false,
        };
        let result = scan(root, &request).unwrap();
        assert_eq!(result.scope, "project");
        assert_eq!(result.unused_sources, ["unused.md"]);
        let item = &result.candidates[0];
        assert_eq!(item.name, "test_draft");
        assert_eq!(item.source.as_ref().unwrap().path, "cases.md");
        assert_eq!(item.source.as_ref().unwrap().hash, digest(draft.as_bytes()));
        assert_eq!(item.target.path, "tests/test_case.py");
        assert!(result.tree.is_some());
        assert!(!root.join("tests/test_case.py").exists());
        let failure = check_saved_sources(root, &result.source_hashes).unwrap_err();
        assert!(failure.message().contains("cases.md"));
        std::fs::write(root.join("cases.md"), draft).unwrap();
        assert_eq!(
            check_saved_sources(root, &result.source_hashes).unwrap(),
            result.source_hashes
        );
    }
    #[test]
    fn source_test_admission_rejects_unknown_execution_scope_and_paths() {
        let valid = json!({"version":1,"input":{"kind":"saved"},"analysis":{"path":"test.py","source":"def test_a(): pass"},"fingerprint":"x","candidate_id":"x","selection":{"kind":"name_filter"}});
        assert!(serde_json::from_value::<RunRequest>(valid.clone()).is_ok());
        let mut nested = valid;
        nested["input"]["project_uri"] = json!("unexpected");
        assert!(serde_json::from_value::<RunRequest>(nested).is_err());
        let body = json!({"version":1,"input":{"kind":"captured"},"analysis":{"path":"test.py","source":"def test_a(): pass"},"fingerprint":"x","candidate_id":"x","selection":{"kind":"name_filter"}});
        assert!(serde_json::from_value::<RunRequest>(body).is_err());
        let body = json!({"version":1,"input":{"kind":"saved"},"analysis":{"path":"test.py","source":"def test_a(): pass"},"fingerprint":"x","candidate_id":"x","selection":{"kind":"containing_scope"},"project_uri":"unexpected"});
        assert!(serde_json::from_value::<RunRequest>(body).is_err());
        let temp = tempfile::tempdir().unwrap();
        let request = Request {
            path: "../escape.py".into(),
            source: "def test_a(): pass".into(),
            project_entry: None,
            sources: BTreeMap::new(),
            capture: false,
        };
        assert!(scan(temp.path(), &request).is_err());
        assert!(
            check_saved_sources(
                temp.path(),
                &BTreeMap::from([("../escape.md".into(), "0".repeat(64))])
            )
            .is_err()
        );
    }
}
