<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Follow a change through declared behavior and observed construction

For maintainers deciding what a source change affects and which verification can safely run. Compiler origins establish construction, metadata establishes declared relationships, and retained receipts establish only their reported observations. A familiar filename never establishes test relevance.

## Keep graph claims typed

A source can have a declared contract and still need explanation. Historical contracts remain visible, and a declared check without an admitted provider remains unsupported. Impact is a bounded inference through explicit edges, never inferred coverage.

<ui:copy id="behavior-readings-model">
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
pub(super) fn id(kind: &str, path: &str) -> String {
    format!("{kind}:{}", hash(path.as_bytes()))
}
pub(super) fn synthetic_key(path:&str,fragment:&str)->String {serde_json::to_string(&(path,fragment)).expect("string tuples serialize")}
#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: String,
    pub path: String,
    pub source_path: Option<String>,
    pub fragment_id: Option<String>,
    pub text_origin:Option<String>,
    pub reading_notice:Option<String>,
    pub label: String,
    pub status: String,
    pub hash: Option<String>,
    pub preview: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub relationship: String,
    pub basis: String,
    pub reason: String,
    pub authority: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Check {
    pub id: String,
    pub label: String,
    pub path: String,
    pub available: bool,
    pub reason: String,
    pub candidate_id: Option<String>,
    pub analysis_fingerprint: Option<String>,
}
#[derive(Default, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: BTreeMap<String, Node>,
    pub edges: Vec<Edge>,
    pub checks: Vec<Check>,
    pub notices: Vec<String>,
    pub truncated: bool,
    pub metadata_hashes: BTreeMap<String, String>,
}
impl Graph {
    pub(super) fn node(&mut self, kind: &str, path: &str, label: &str, status: &str) -> String {
        let key = id(kind, path);
        if self.nodes.contains_key(&key) {
            return key;
        }
        if self.nodes.len() >= 2048 {
            self.truncated = true;
            return key;
        }
        self.nodes.insert(
            key.clone(),
            Node {
                id: key.clone(),
                kind: kind.into(),
                path: path.into(),
                source_path: Some(path.into()),
                fragment_id:None,
                text_origin:None,reading_notice:None,
                label: label.chars().take(512).collect(),
                status: status.into(),
                hash: None,
                preview: None,
            },
        );
        key
    }
    pub(super) fn edge(
        &mut self,
        from: &str,
        to: &str,
        relation: &str,
        basis: &str,
        reason: &str,
        authority: &str,
    ) {
        if self.edges.len() >= 8192 {
            self.truncated = true;
            return;
        }
        if !self.nodes.contains_key(from) || !self.nodes.contains_key(to) {
            return;
        }
        let next = Edge {
            from: from.into(),
            to: to.into(),
            relationship: relation.into(),
            basis: basis.into(),
            reason: reason.into(),
            authority: authority.into(),
        };
        if !self
            .edges
            .iter()
            .any(|e| e.from == next.from && e.to == next.to && e.relationship == next.relationship)
        {
            self.edges.push(next)
        }
    }
</ui:copy>

## Attach only explicitly declared relationships

Contract references and paired test modules are declared navigation edges. Their names never establish semantic coverage. Unknown contract dispositions stay unclassified, while historical and needs-explanation statuses remain visible.

<ui:copy id="behavior-readings-metadata">
    pub(super) fn metadata(&mut self, map: &Value, narrative: &Value, paths: &BTreeSet<String>) {
        if let Some(contracts) = map["contracts"].as_object() {
            for (key, c) in contracts {
                let related = c["references"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .chain(c["owned"].as_array().into_iter().flatten())
                    .any(|p| p.as_str().is_some_and(|p| paths.contains(p)));
                if !related {
                    continue;
                }
                let page = c["page"].as_str().unwrap_or("");
                let status = match c["disposition"].as_str() {
                    Some("historical") => "historical",
                    Some("retained-requirement" | "reconciled-requirement" | "rewritten") => {
                        "canonical-declared"
                    }
                    _ => "declared-status-unknown",
                };
                let contract = self.node("contract", &synthetic_key(page,key), c["title"].as_str().unwrap_or(key), status);
                if let Some(node)=self.nodes.get_mut(&contract){node.path=page.into();node.source_path=Some(page.into());}
                for p in c["references"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .chain(c["owned"].as_array().into_iter().flatten())
                    .filter_map(Value::as_str)
                {
                    if paths.contains(p) {
                        let target = self.node("source", p, p, "captured");
                        self.edge(&contract,&target,"declares-related-source","declared","Contract metadata references this source; this is not semantic coverage.","hickory-dogfood/behavior-map.json");
                    }
                }
                for test in c["tests"].as_array().into_iter().flatten() {
                    if self.checks.len() >= 1024 {
                        self.truncated = true;
                        break;
                    }
                    if let Some(file) = test["file"].as_str() {
                        let key = serde_json::to_string(&(file,&test["runner"],&test["name"])).expect("JSON metadata serializes");
                        let check = id("check", &key);
                        let node = check.clone();
                        if self.nodes.len() < 2048 {
                            self.nodes.insert(
                                node.clone(),
                                Node {
                                    id: node.clone(),
                                    kind: "check".into(),
                                    path: file.into(),
                                    source_path: Some(file.into()),
                                    fragment_id:None,
                text_origin:None,reading_notice:None,
                                    label: test["name"].as_str().unwrap_or(file).into(),
                                    status: "declared".into(),
                                    hash: None,
                                    preview: None,
                                },
                            );
                        } else {
                            self.truncated = true;
                        }
                        self.edge(&contract,&node,"declares-check","declared","This exact runner/file/name was declared by contract metadata; execution support is checked separately.","hickory-dogfood/behavior-map.json");
                        if !self.checks.iter().any(|c| c.id == check) {
                            self.checks.push(Check{id:check,label:test["name"].as_str().unwrap_or(file).into(),path:file.into(),available:false,reason:format!("Declared runner {} has no exact admitted source candidate yet.",test["runner"].as_str().unwrap_or("unknown")),candidate_id:None,analysis_fingerprint:None});
                        }
                    }
                }
            }
        }
        if let Some(pairs) = narrative["paired_test_modules"].as_array() {
            for pair in pairs {
                let Some(implementation) = pair["implementation"].as_str() else {
                    continue;
                };
                if !paths.contains(implementation) {
                    continue;
                }
                if let Some(test) = pair["tests"].as_str() {
                    let a = self.node("source", implementation, implementation, "captured");
                    let b = self.node("test-module", test, test, "declared");
                    self.edge(&a,&b,"paired-test-module","declared","Narrative metadata explicitly pairs these modules; this does not establish execution or assertion quality.","hickory-dogfood/narrative-map.json");
                }
            }
        }
        if let Some(fragments) = narrative["fragments"].as_array() {
            for fragment in fragments {
                let Some(output) = fragment["output"].as_str() else {
                    continue;
                };
                if !paths.contains(output) {
                    continue;
                }
                let page = fragment["page"].as_str().unwrap_or("");
                let symbol = fragment["symbol"].as_str().unwrap_or("");
                let fragment_id=fragment["id"].as_str().unwrap_or(symbol);
                let key = synthetic_key(page,fragment_id);
                let basis = fragment["basis"].as_str().unwrap_or("needs-explanation");
                let status = if basis == "needs-explanation" {
                    "needs-explanation"
                } else {
                    "explanation-declared"
                };
                let a = self.node("explanation", &key, symbol, status);
                if let Some(node) = self.nodes.get_mut(&a) {
                    node.path=format!("{page}#{fragment_id}");node.fragment_id=Some(fragment_id.into());
                    node.source_path = Some(page.into());
                }
                let b = self.node("source", output, output, "captured");
                self.edge(
                    &a,
                    &b,
                    "explains",
                    "declared",
                    basis,
                    "hickory-dogfood/narrative-map.json",
                );
            }
        }
    }
</ui:copy>

## Explain bounded transitive impact

Impact walks only the displayed edges, in either direction, and labels the resulting relationship as inferred. A depth or graph limit keeps omitted relationships unknown rather than implying an empty impact set.

<ui:copy id="behavior-readings-impact">
    pub(super) fn impact(&self, focus: &str) -> Value {
        let mut reached: BTreeSet<String> = self
            .nodes
            .values()
            .filter(|n| n.path == focus)
            .map(|n| n.id.clone())
            .collect();
        let mut links = Vec::new();
        let mut complete = false;
        for _ in 0..8 {
            let before = reached.len();
            for edge in &self.edges {
                if reached.contains(&edge.from) || reached.contains(&edge.to) {
                    reached.insert(edge.from.clone());
                    reached.insert(edge.to.clone());
                    if !links.iter().any(|e: &&Edge| {
                        e.from == edge.from
                            && e.to == edge.to
                            && e.relationship == edge.relationship
                    }) {
                        links.push(edge)
                    }
                }
            }
            if reached.len() == before {
                complete = true;
                break;
            }
        }
        json!({"basis":"inferred","reason":"Related through the displayed compiler or declared-metadata edges; not a claim of executed reachability.","nodes":reached,"via":links,"depth_limit":8,"truncated":!complete})
    }
}
</ui:copy>

## Connect recorded input and measured reachability without conflating them

A record storing an input hash does not prove that input executed. Only an actual retained collector report supplies line reachability, and manual/imported reports retain their declared authority. Guarded verification receipts link to their exact selected check; process exit remains separate from a collected test verdict. These observations are attached after computing source admission identity so an independent receipt refresh cannot revoke unrelated source reads.

<ui:copy id="behavior-readings-observation-links">
impl Graph {
    /// Add observation links after computing the input/admission fingerprint.
    /// An independently arriving receipt must not change source authority.
    pub(super) fn observations(&mut self,evidence:&Value,receipts:&Value,fingerprint:&str,budget:&mut usize){
        for record in evidence["records"].as_array().into_iter().flatten(){
            let Some(record_id)=record["id"].as_str() else {continue};
            let related=record["matching_inputs"].as_array().into_iter().flatten().filter_map(Value::as_str).any(|path|self.nodes.contains_key(&id("source",path)));
            if !related {continue}
            let key=self.node("evidence",&format!("coverage:{record_id}"),record["collector"].as_str().unwrap_or("Retained coverage record"),record["freshness"].as_str().unwrap_or("unknown"));
            self.observation_preview(&key,record,budget);
            for path in record["matching_inputs"].as_array().into_iter().flatten().filter_map(Value::as_str){let source=id("source",path);self.edge(&source,&key,"recorded-input",if record["origin"]=="current-workspace"{"observed"}else{"declared"},"The retained record names this input. Workspace origin and byte freshness remain separate; this does not prove current-source execution or frozen inputs.","coverage record");}
            for reach in record["reachability"].as_array().into_iter().flatten(){let Some(path)=reach["path"].as_str()else{continue};let source=id("source",path);let automatic=record["evidence_kind"]=="automated-collector"&&record["origin"]=="current-workspace";self.edge(&source,&key,"reported-line-reachability",if automatic{"observed"}else{"declared"},if automatic{"The collector reports line reachability for this file. Attribution and source freshness remain separate; no guarantee or assertion-quality relationship is inferred."}else{"This manual, imported or origin-unconfirmed report declares line reachability. It is not current-workspace automated verification or a collected test verdict."},"retained collector report");}
        }
        for receipt in receipts["records"].as_array().into_iter().flatten(){let (Some(request),Some(check))=(receipt["request_id"].as_str(),receipt["check_id"].as_str())else{continue};if !self.nodes.contains_key(check){continue}let owned=receipt["origin"]=="current-workspace";let current=owned&&receipt["reading_fingerprint"]==fingerprint;let key=self.node("verification",&format!("request:{request}"),receipt["label"].as_str().unwrap_or(request),if current{"current-reading-receipt"}else if owned{"historical-reading-receipt"}else{"historical-origin-unknown"});self.observation_preview(&key,receipt,budget);self.edge(check,&key,"verification-request",if owned{"observed"}else{"declared"},if owned{"The owned retained request explicitly selected this check. A process outcome is not a collected test-case verdict."}else{"This retained receipt declares a historical check association, but its workspace origin and live process are unconfirmed. This is not a collected test-case verdict."},"guarded verification receipt");}
    }
    fn observation_preview(&mut self,key:&str,value:&Value,budget:&mut usize){if let Some(node)=self.nodes.get_mut(key){node.source_path=None;node.text_origin=Some("retained-summary".into());let raw=value.to_string();node.hash=Some(hash(raw.as_bytes()));let mut preview=String::new();for character in raw.chars().take(2000){if preview.len()+character.len_utf8()>*budget{break}preview.push(character)}*budget=budget.saturating_sub(preview.len());node.preview=Some(preview);}}
}
</ui:copy>

## Verify that unrelated names never become edges

The focused policy tests reject filename-only relationships and preserve explanation gaps and historical contracts without promoting them to verified behavior.

<ui:copy id="behavior-readings-model-tests">
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_declared_relationships_create_checks() {
        let mut graph = Graph::default();
        graph.metadata(&json!({"contracts":{"c":{"page":"c.md","references":["a.py"],"tests":[{"runner":"pytest","file":"test_a.py","name":"test_a"}]}}}),&json!({}),&BTreeSet::from(["a.py".into(),"test_unrelated.py".into()]));
        assert_eq!(graph.checks.len(), 1);
        assert!(!graph.checks[0].available);
        assert!(!graph.nodes.values().any(|n| n.path == "test_unrelated.py"));
        assert!(graph.edges.iter().all(|e| e.basis == "declared"));
    }
    #[test]
    fn literal_hash_names_do_not_alias_synthetic_fragment_or_shared_contract_identities(){let mut graph=Graph::default();graph.metadata(&json!({"contracts":{"one":{"page":"same.md","references":["a.py"]},"two":{"page":"same.md","references":["a.py"]}}}),&json!({"fragments":[{"page":"a#b","id":"c","output":"a.py"},{"page":"a","id":"b#c","output":"a.py"}]}),&BTreeSet::from(["a.py".into()]));assert_eq!(graph.nodes.values().filter(|node|node.kind=="explanation").count(),2);assert_eq!(graph.nodes.values().filter(|node|node.kind=="contract").count(),2);assert_ne!(id("fragment",&synthetic_key("a#b","c")),id("fragment",&synthetic_key("a","b#c")));}

    #[test]
    fn observations_keep_manual_reachability_and_process_outcomes_distinct(){let mut graph=Graph::default();let source=graph.node("source","a#literal.py","a#literal.py","captured");assert_eq!(graph.nodes[&source].source_path.as_deref(),Some("a#literal.py"));let check=graph.node("check","selected","selected","declared");let evidence=json!({"records":[{"id":"manual","collector":"import","freshness":"origin-unknown","evidence_kind":"manual-or-imported","matching_inputs":["a#literal.py"],"reachability":[{"path":"a#literal.py","hit_lines":1}]}]});let receipts=json!({"records":[{"request_id":"owned","origin":"current-workspace","check_id":check,"reading_fingerprint":"old","label":"selected","status":"process-finished","outcome":"process-only"}]});let mut budget=1024;graph.observations(&evidence,&receipts,"new",&mut budget);assert!(graph.edges.iter().any(|edge|edge.relationship=="reported-line-reachability"&&edge.basis=="declared"));assert!(graph.nodes.values().any(|node|node.status=="historical-reading-receipt"));assert!(graph.edges.iter().filter(|edge|edge.relationship=="verification-request").all(|edge|edge.reason.contains("not a collected test-case verdict")));}

    #[test]
    fn collector_mode_without_confirmed_origin_does_not_become_current_execution(){for origin in ["unknown","current-workspace"]{let mut graph=Graph::default();graph.node("source","a.py","a.py","captured");let evidence=json!({"records":[{"id":"report","origin":origin,"collector":"pytest-cov","freshness":"origin-unknown","evidence_kind":"automated-collector","matching_inputs":["a.py"],"reachability":[{"path":"a.py","hit_lines":1}]}]});let mut budget=4096;graph.observations(&evidence,&json!({"records":[]}),"reading",&mut budget);assert!(graph.edges.iter().filter(|edge|edge.relationship=="reported-line-reachability").all(|edge|edge.basis==if origin=="current-workspace"{"observed"}else{"declared"}));}}

    #[test]
    fn gaps_and_history_do_not_become_verified() {
        let mut graph = Graph::default();
        graph.metadata(&json!({"contracts":{"c":{"page":"old.md","disposition":"historical","references":["a.py"]}}}),&json!({"fragments":[{"id":"a","page":"a.md","output":"a.py","basis":"needs-explanation"}]}),&BTreeSet::from(["a.py".into()]));
        assert!(graph.nodes.values().any(|n| n.status == "historical"));
        assert!(
            graph
                .nodes
                .values()
                .any(|n| n.status == "needs-explanation")
        );
    }
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/behavior_readings/model.rs"><ui:paste select="#behavior-readings-model"/><ui:paste select="#behavior-readings-metadata"/><ui:paste select="#behavior-readings-impact"/><ui:paste select="#behavior-readings-observation-links"/><ui:paste select="#behavior-readings-model-tests"/></ui:file>

## Recompute source and metadata identity before execution

The service captures construction before reading optional metadata. Missing metadata stays unknown. Related declarations are eligible only when an explicit contract or module pairing connects them; saved Run remains the authority for provider discovery and exact source admission. Reading previews are bounded and never write constructed outputs.

<ui:copy id="behavior-readings-service">
//! Connected readings preserve the authority of each relationship and observation.
mod model;
mod store;
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    source_tests,
};
use axum::{
    Json,
    extract::{Query, State},
};
use model::{Graph, hash, id};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path},
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingRequest {
    pub version: u32,
    pub entry: String,
    #[serde(default)]
    pub sources: BTreeMap<String, String>,
    #[serde(default)]
    pub missing_sources: BTreeSet<String>,
    #[serde(default)]
    pub unaddressed_sources:BTreeMap<String,String>,
    #[serde(default)]
    pub saved_guards:BTreeMap<String,String>,
    pub focus: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyRequest {
    pub version: u32,
    pub engine_workspace_id: String,
    pub workspace_id: String,
    pub reading_fingerprint: String,
    pub check_id: String,
    pub request_id: String,
    pub context: ReadingRequest,
    pub profile: Option<crate::debug_sessions::ProfileSelection>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationQuery {
    pub id: String,
}
fn root_lease(state:&LocalState)->ApiResult<String>{state.resource_root.check(state.index.root()).map(str::to_owned).map_err(ApiError::conflict)}
fn root_current(state:&LocalState,expected:&str)->ApiResult<()>{if root_lease(state)?!=expected{return Err(ApiError::conflict("Workspace engine or physical root changed. No verification was started."));}Ok(())}
fn intent_fingerprint(request: &VerifyRequest) -> Result<String, serde_json::Error> {
    serde_json::to_vec(request).map(|bytes| hash(&bytes))
}
fn safe_read(root: &Path, path: &str, limit: u64) -> Result<Option<Vec<u8>>, String> {
    if path.is_empty()
        || Path::new(path).is_absolute()
        || !Path::new(path)
            .components()
            .all(|p| matches!(p, Component::Normal(_)))
    {
        return Err("Reading path must be a confined workspace-relative source.".into());
    }
    let mut next = root.to_path_buf();
    for component in Path::new(path).components() {
        next.push(component);
        match std::fs::symlink_metadata(&next) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err(format!(
                        "Reading source {path} is a symlink; origin is unavailable."
                    ));
                }
                if meta.is_file() && meta.len() > limit {
                    return Err(format!(
                        "Reading source {path} exceeds the bounded preview limit."
                    ));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.to_string()),
        }
    }
    std::fs::read(next).map(Some).map_err(|e| e.to_string())
}
fn metadata(root: &Path, path: &str, graph: &mut Graph) -> Value {
    match safe_read(root, path, 24 * 1024 * 1024) {
        Ok(Some(bytes)) => {
            graph.metadata_hashes.insert(path.into(), hash(&bytes));
            match serde_json::from_slice::<Value>(&bytes) {
                Ok(value) if value["format"] == 1 => value,
                _ => {
                    graph.notices.push(format!(
                        "{path}: unsupported or malformed metadata; relationships remain unknown."
                    ));
                    Value::Null
                }
            }
        }
        Ok(None) => {
            graph.metadata_hashes.insert(path.into(), "missing".into());
            graph.notices.push(format!(
                "{path}: missing; undeclared relationships are unknown."
            ));
            Value::Null
        }
        Err(error) => {
            graph
                .metadata_hashes
                .insert(path.into(), "unavailable".into());
            graph.notices.push(error);
            Value::Null
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileRequest {
    pub engine_workspace_id: String,
    pub workspace_id: String,
    pub reading_fingerprint: String,
    pub context: ReadingRequest,
}
</ui:copy>

## Choose an exact discovered profile explicitly

A declared check may lack a source-candidate provider. The user can associate it with a real engine-discovered profile. The service derives a captured output observation context, encodes its source once, and delegates selection and process creation to the existing exact-profile authority. It accepts no argv or client-selected program.

<ui:copy id="behavior-readings-profiles">
fn profile_query(
    request: &ReadingRequest,
    built: &Built,
) -> ApiResult<super::launch_profiles::Query> {
    let viewed = built.profile_program.as_deref().ok_or_else(|| {
        ApiError::unprocessable(
            "No captured regular output is available as a launch-profile observation context.",
        )
    })?;
    Ok(super::launch_profiles::Query {
        uri: crate::launch_profiles::inventory::workspace_source_uri(&request.entry)
            .map_err(ApiError::bad_request)?,
        project: "new".into(),
        program: Some(viewed.into()),
        expected_source: None,
        expected_document: request.sources.get(&request.entry).cloned(),
    })
}
// Attaching observes an existing process; it cannot execute a relevant check.
// Keep that provider distinction visible without offering a misleading Run choice.
fn verification_profiles(mut value:Value)->Result<Value,String>{let profiles=value["profiles"].as_array_mut().ok_or("Launch-profile provider returned no profile list.")?;let mut unavailable=Vec::new();profiles.retain(|profile|{if profile.get("attach").is_some_and(|attach|!attach.is_null()){unavailable.push(format!("{}: attach profile unavailable for relevant verification Run. Use Debug to inspect its existing target; no check process is launched.",profile["name"].as_str().unwrap_or("Unnamed profile")));false}else{true}});let notices=value.as_object_mut().ok_or("Launch-profile provider returned no object.")?.entry("notices").or_insert_with(||json!([])).as_array_mut().ok_or("Launch-profile notices are unavailable.")?;notices.extend(unavailable.into_iter().map(Value::String));Ok(value)}
pub async fn profiles(
    State(state): State<LocalState>,
    Json(request): Json<ProfileRequest>,
) -> ApiResult<Json<Value>> {
    let engine=root_lease(&state)?;root_current(&state,&request.engine_workspace_id)?;
    let root = state.index.root().to_path_buf();
    let context = request.context.clone();
    let build_root = root.clone();
    let built = tokio::task::spawn_blocking(move || build(&build_root, &context))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map_err(ApiError::unprocessable)?;
    root_current(&state,&engine)?;
    if built.value["verification_available"]!=true{return Err(ApiError::conflict("Physical source or construction authority is unavailable. Captured readings remain readable; verification was not started."));}
    if built.value["workspace_id"] != request.workspace_id
        || built.value["reading_fingerprint"] != request.reading_fingerprint
    {
        return Err(ApiError::conflict(
            "Reading changed; refresh before choosing a profile.",
        ));
    }
    let scan=built.scan.as_ref().ok_or_else(||ApiError::conflict("Construction input authority is unavailable. Repair or refresh it before verification."))?;
    source_tests::check_saved_sources(&root, &scan.source_hashes)?;
    root_current(&state,&engine)?;
    let query=profile_query(&request.context,&built)?;let provider=state.clone();let runtime=tokio::runtime::Handle::current();
    let value=tokio::task::spawn_blocking(move||runtime.block_on(super::launch_profiles::list(State(provider),Json(query)))).await.map_err(|error|ApiError::internal(error.to_string()))??;root_current(&state,&engine)?;Ok(Json(verification_profiles(value.0).map_err(ApiError::unprocessable)?))
}
</ui:copy>

## Capture one text snapshot per source and coordinate domain

Every node for a source shares its captured bytes and hash. Compiler-owned test
modules and outputs use the typed projected tree; authored pages reuse the
compiler capture or supplied editor snapshot. Metadata-only pages are read once.
A named explanation preview starts near its observed parser span, with missing
or ambiguous declarations left explicit. Full readings remain separately paged.
The total preview budget applies across all these nodes without retaining a
second unbounded source cache.

<ui:copy id="behavior-readings-snapshots">
fn preview_text(source:&str,start:usize,budget:&mut usize)->String {let mut text=String::new();for character in source.chars().skip(start).take(2000){if text.len()+character.len_utf8()>*budget{break}text.push(character)}*budget=budget.saturating_sub(text.len());text}
fn attach_source_previews(root:&Path,request:&ReadingRequest,scan:Option<&source_tests::Scan>,graph:&mut Graph)->usize {
 use std::borrow::Cow;
 let mut budget=1024*1024usize;let mut groups:BTreeMap<String,Vec<String>>=BTreeMap::new();for node in graph.nodes.values(){if let Some(path)=&node.source_path{groups.entry(path.clone()).or_default().push(node.id.clone());}}
 let base=Path::new(&request.entry).parent().unwrap_or(Path::new(""));let outputs:BTreeMap<String,&std::sync::Arc<hick_lang::ProjectEntry>>=scan.and_then(|scan|scan.tree.as_ref()).into_iter().flat_map(|tree|tree.entries()).filter_map(|(path,entry)|source_tests::relative(base,path).ok().map(|path|(path,entry))).collect();
 for (path,nodes) in groups {
  let projected=outputs.get(&path).filter(|entry|matches!(entry.kind,hick_lang::EntryKind::File));let projected_hash=projected.map(|entry|hash(&entry.bytes));
  let provided=request.sources.get(&path);let authored=provided.or_else(||scan.and_then(|scan|scan.source_texts.get(&path)));
  let raw:Result<Option<Cow<'_,[u8]>>,String>=if source_tests::relative(Path::new(""),&path).ok().as_deref()!=Some(path.as_str()){Err("Declared source spelling is foreign or ambiguous; its reading is unresolved.".into())}else if let Some(source)=authored{Ok(Some(Cow::Borrowed(source.as_bytes())))}else{safe_read(root,&path,4*1024*1024).map(|bytes|bytes.map(Cow::Owned))};
  let failure=raw.as_ref().err().cloned();let raw=raw.ok().flatten();let raw_hash=raw.as_ref().map(|bytes|hash(bytes));let raw_text=raw.as_ref().and_then(|bytes|std::str::from_utf8(bytes).ok());
  let needs_spans=nodes.iter().any(|key|graph.nodes.get(key).is_some_and(|node|node.fragment_id.is_some()));let mut starts:BTreeMap<String,Vec<usize>>=BTreeMap::new();if needs_spans{if let Some(source)=raw_text{if let Ok(document)=hick_lang::parse(source){for tag in document.all_tags(){if let (Some(name),Some(span))=(tag.get_attribute("id"),tag.source_span){if span.file_id.is_none(){starts.entry(name.into()).or_default().push(span.start);}}}}}}
  for key in nodes {let node=graph.nodes.get_mut(&key).expect("captured node");if node.status=="saved-baseline-absent-from-draft"{node.text_origin=Some("saved-baseline".into());if let Some(text)=node.preview.take(){node.preview=Some(preview_text(&text,0,&mut budget));}continue}
   let projected_node=projected.is_some()&&(node.status=="constructed"||node.kind=="check"||node.kind=="test-module");let text=if projected_node{node.hash=projected_hash.clone();node.text_origin=Some("constructed".into());projected.and_then(|entry|std::str::from_utf8(&entry.bytes).ok())}else{node.hash=raw_hash.clone();node.text_origin=raw.as_ref().map(|_|if provided.is_some(){"captured-draft"}else{"saved-source"}.into());raw_text};
   let Some(text)=text else{node.preview=None;node.reading_notice=Some(failure.clone().unwrap_or_else(||if node.hash.is_some(){"Binary source has no UTF-8 text reading.".into()}else{"Source is missing; no replacement text was acquired.".into()}));continue};
   if node.kind=="fragment"{if let Some(body)=node.preview.take(){node.preview=Some(preview_text(&body,0,&mut budget));continue}}
   let mut start=0;if node.kind=="explanation"{if let Some(fragment)=&node.fragment_id{match starts.get(fragment).map(Vec::as_slice){Some([byte])=>{if let Some(prefix)=text.get(..*byte){start=prefix.chars().count().saturating_sub(800);}},_=>{node.preview=None;node.reading_notice=Some("Declared fragment is missing or ambiguous in this captured source. Read the whole source; no precise target was invented.".into());continue}}}}
   node.preview=Some(preview_text(text,start,&mut budget));if budget==0&&!text.is_empty(){node.reading_notice=Some("Preview budget exhausted; the bound full-source reader remains available.".into());}else if text.is_empty(){node.reading_notice=Some("Captured empty text.".into());}
  }
 }
 budget
}
</ui:copy>

## Capture compiler and metadata facts independently

Compiler origins establish construction. Metadata establishes ownership and declared relationships; collector records establish only their retained observations. Compiler output previews use compiler bytes, never a same-spelled disk file. Synthetic fragment labels carry a separate raw source path so literal # and % filenames remain exact.

<ui:copy id="behavior-readings-acquisition">
struct Built {
    value: Value,
    analysis: source_tests::Request,
    scan: Option<source_tests::Scan>,
    profile_program:Option<String>,
}
// Retained text is captured independently of compiler participation. A filename
// suffix cannot prove that an authored construction belongs to the selected entry.
fn retained_source_authority(root:&Path,request:&ReadingRequest,selected:&BTreeMap<String,String>)->Result<(),String>{
 if request.sources.len()+request.unaddressed_sources.len()>4096{return Err("Retained source count exceeds the guarded reading budget.".into())}
 let mut bytes=0usize;for text in request.sources.values().chain(request.unaddressed_sources.values()){if text.len()>8*1024*1024{return Err("A retained source exceeds the guarded reading budget.".into())}bytes=bytes.checked_add(text.len()).ok_or("Retained source capacity overflow.")?;if bytes>32*1024*1024{return Err("Retained sources exceed the guarded reading budget.".into())}}
 for text in request.unaddressed_sources.values(){if super::resource_inputs::contains_construction(text)?{return Err("An unaddressed retained draft contains construction declarations. Assign/select its real context or reconcile its deleted identity before verification; captured readings remain readable.".into())}}
 if request.saved_guards.len()>4096||request.saved_guards.values().any(|guard|guard.is_empty()||guard.len()>16*1024)||request.saved_guards.values().map(String::len).sum::<usize>()>4*1024*1024{return Err("Loaded physical source guards exceed their bounded reading capacity.".into())}
 for(path,text)in &request.sources{super::resource_changes::validate_path(path)?;if !request.saved_guards.contains_key(path){return Err(format!("Originally loaded physical source authority is missing for {path}. Reload/reconcile that retained editor before verification."))}if super::resource_inputs::contains_construction(text)?&&selected.get(path)!=Some(&hash(text.as_bytes())){return Err(format!("Retained construction {path} is outside the selected captured context. Select/reconcile it before verification."))}}
 super::resource_inputs::validate_guard_map(root,&request.saved_guards,&BTreeSet::new())?;
 for(path,text)in &request.sources{let(_,saved)=super::resource_inputs::saved_target(root,path)?;if !matches!(saved,super::resource_inputs::SavedSource::File{text:saved,..} if saved==*text){return Err(format!("Retained source {path} has unsaved or missing text. Captured draft readings remain readable; saved verification requires that source to be saved/reconciled first."))}}
 super::resource_inputs::validate_guard_map(root,&request.saved_guards,&BTreeSet::new())
}
fn build(root: &Path, request: &ReadingRequest) -> Result<Built, String> {
    if request.version != 1 {
        return Err("Unsupported behavior reading protocol.".into());
    }
    let construction = match source_tests::construction_reading(
        root,
        source_tests::ConstructionRequest {
            entry: request.entry.clone(),
            sources: request.sources.clone(),
            missing_sources: request.missing_sources.clone(),
            focus: request.focus.clone(),
        },
    ) {
        Ok(value) => value,
        Err(error) => return unavailable(root, request, error),
    };
    let source = request
        .sources
        .get(&request.entry)
        .cloned()
        .or_else(|| {
            safe_read(root, &request.entry, 8 * 1024 * 1024)
                .ok()
                .flatten()
                .and_then(|b| String::from_utf8(b).ok())
        })
        .ok_or("Construction entry is unreadable.")?;
    let analysis = source_tests::Request {
        path: request.entry.clone(),
        source,
        project_entry: Some(request.entry.clone()),
        sources: request.sources.clone(),
        capture: false,
    };
    let scan =
        match source_tests::scan_with_namespace(root, &analysis, None, &request.missing_sources) {
            Ok(scan) => scan,
            Err(error) => return unavailable(root, request, error),
        };
    if json!(scan.source_hashes) != construction["after_sources"] {
        return unavailable(
            root,
            request,
            "Construction inputs changed while relationships were read. Refresh the reading."
                .into(),
        );
    }
    let mut graph = Graph::default();
    graph.truncated=construction["outputs_truncated"]==true||construction["tests_truncated"]==true;
    let map = metadata(root, "hickory-dogfood/behavior-map.json", &mut graph);
    let narrative = metadata(root, "hickory-dogfood/narrative-map.json", &mut graph);
    let mut paths: BTreeSet<String> = if let Some(focus) = &request.focus {
        BTreeSet::from([focus.clone()])
    } else {
        construction["after_sources"]
            .as_object()
            .into_iter()
            .flat_map(|m| m.keys().cloned())
            .collect()
    };
    for output in construction["outputs"].as_array().into_iter().flatten() {
        if let Some(path) = output["path"].as_str() {
            paths.insert(path.into());
            for contributor in output["contributors"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                paths.insert(contributor.into());
            }
            let target = graph.node("source", path, path, "constructed");
            for contributor in output["contributors"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                let source = graph.node("source", contributor, contributor, "captured");
                graph.edge(&source,&target,"constructs","observed","Compiler origins report this contribution; byte ranges remain in the construction reading.","compiler");
            }
        }
    }
    for output in construction["outputs"].as_array().into_iter().flatten() {
        let Some(output_path) = output["path"].as_str() else {
            continue;
        };
        for origin in output["origins"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|origin| origin["phase"] == "after")
        {
            let (Some(path), Some(name), Some(body)) = (
                origin["path"].as_str(),
                origin["fragment"]["name"].as_str(),
                origin["fragment"]["body"].as_array(),
            ) else {
                continue;
            };
            let Some(source) = scan.source_texts.get(path) else {
                continue;
            };
            let range = body
                .first()
                .and_then(Value::as_u64)
                .zip(body.get(1).and_then(Value::as_u64));
            let Some((start, end)) = range else { continue };
            let Some(text) = source.get(start as usize..end as usize) else {
                continue;
            };
            let fragment = graph.node(
                "fragment",
                &model::synthetic_key(path,name),
                name,
                "compiler-declared",
            );
            if let Some(node) = graph.nodes.get_mut(&fragment) {
                node.path=format!("{path}#{name}");node.fragment_id=Some(name.into());
                node.source_path = Some(path.into());
                node.hash = Some(hash(source.as_bytes()));
                node.preview = Some(text.chars().take(2000).collect());
            }
            let owner = graph.node("source", path, path, "captured");
            let target = graph.node("source", output_path, output_path, "constructed");
            graph.edge(
                &owner,
                &fragment,
                "declares-fragment",
                "observed",
                "The compiler reports this named source fragment and its exact body range.",
                "compiler",
            );
            graph.edge(&fragment,&target,"contributes-fragment","observed","The compiler reports this fragment contributing to the selected output; this is not execution coverage.","compiler");
        }
    }
    graph.notices.extend(scan.notices.iter().filter(|notice|notice.contains("Physical workspace identity")).cloned());
    graph.metadata(&map, &narrative, &paths);
    if let Some(constructors) = map["constructors"].as_object() {
        for path in &paths {
            if let Some(owner) = constructors.get(path).and_then(|v| v["source"].as_str()) {
                let a = graph.node("source", owner, owner, "declared-owner");
                let b = graph.node("source", path, path, "captured");
                graph.edge(&a,&b,"owns-construction","declared","Construction metadata names the owning literate source; ownership is not semantic verification.","hickory-dogfood/behavior-map.json");
            }
        }
    }
    let paired: BTreeSet<String> = graph
        .nodes
        .values()
        .filter(|n| n.kind == "test-module")
        .map(|n| n.path.clone())
        .collect();
    for candidate in &scan.candidates {
        let path = &candidate.target.path;
        if !paired.contains(path)
            && !graph
                .checks
                .iter()
                .any(|c| c.path == *path && c.label == candidate.name)
        {
            continue;
        }
        let existing = graph
            .checks
            .iter()
            .position(|c| c.path == *path && c.label == candidate.name);
        let check_id = existing
            .map(|i| graph.checks[i].id.clone())
            .unwrap_or_else(|| id("candidate-check", &candidate.id));
        let check=model::Check{id:check_id.clone(),label:candidate.name.clone(),path:path.clone(),available:candidate.name_filter.as_ref().is_some_and(|v|!v.is_empty()),reason:"Observed declaration in an explicitly related test module; filtered saved Run revalidates all construction inputs. Process exit is not a collected test verdict.".into(),candidate_id:Some(candidate.id.clone()),analysis_fingerprint:Some(scan.fingerprint.clone())};
        if let Some(index) = existing {
            graph.checks[index] = check
        } else if graph.checks.len() < 1024 {
            graph.checks.push(check)
        } else {
            graph.truncated = true;
            continue;
        }
        let a = graph.node("test-module", path, path, "declared");
        let b = check_id.clone();
        if graph.nodes.len() < 2048 {
            graph.nodes.insert(
                b.clone(),
                model::Node {
                    id: b.clone(),
                    kind: "check".into(),
                    path: path.clone(),
                    source_path: Some(path.clone()),
                    fragment_id:None,
                text_origin:None,reading_notice:None,
                    label: candidate.name.clone(),
                    status: "detected-declaration".into(),
                    hash: None,
                    preview: None,
                },
            );
        } else {
            graph.truncated = true;
        }
        graph.edge(
            &a,
            &b,
            "contains-declaration",
            "observed",
            "The test-source provider detected this exact declaration, not a collected test case.",
            "source-test analysis",
        );
    }
    for output in construction["outputs"].as_array().into_iter().flatten() {
        let Some(path) = output["path"].as_str() else {
            continue;
        };
        let node_id = id("source", path);
        if let Some(node) = graph.nodes.get_mut(&node_id) {
            let snapshot = if output["after"].is_object() {
                &output["after"]
            } else {
                node.status = "saved-baseline-absent-from-draft".into();
                &output["before"]
            };
            node.hash = snapshot["hash"].as_str().map(str::to_string);
            node.preview = snapshot["preview"].as_str().map(str::to_string);
        }
    }
    let authority=retained_source_authority(root,request,&scan.source_hashes);if let Err(reason)=&authority{graph.notices.push(format!("Relevant verification unavailable: {reason}"));}
    let mut budget=attach_source_previews(root,request,Some(&scan),&mut graph);
    let fingerprint=hash(&serde_json::to_vec(&json!({"workspace":scan.workspace_id,"context":request,"construction":construction,"metadata":graph.metadata_hashes,"graph":graph})).map_err(|e|e.to_string())?);
    let impact = request.focus.as_deref().map(|focus| graph.impact(focus));
    let mut evidence_sources:BTreeMap<String,String>=paths.iter().filter_map(|path|scan.source_hashes.get(path).map(|hash|(path.clone(),hash.clone()))).collect();for output in construction["outputs"].as_array().into_iter().flatten(){if let (Some(path),Some(hash))=(output["path"].as_str(),output["after"]["hash"].as_str()){evidence_sources.insert(path.into(),hash.into());}}
    let evidence = crate::coverage::reading_evidence(root, &scan.workspace_id, &evidence_sources)
        .unwrap_or_else(
            |error| json!({"status":"unknown","records":[],"notice":error.to_string()}),
        );
    let base=Path::new(&request.entry).parent().unwrap_or(Path::new(""));
    let profile_candidates:Vec<(String,String)>=scan.tree.as_ref().into_iter().flat_map(|tree|tree.entries()).filter(|(_,entry)|matches!(entry.kind,hick_lang::EntryKind::File)).filter_map(|(path,_)|source_tests::relative(base,path).ok().filter(|root_path|construction["outputs"].as_array().into_iter().flatten().any(|output|output["path"]==*root_path)).map(|root_path|(root_path,path.clone()))).collect();
    let selected_profile_source=profile_candidates.iter().find(|(root_path,_)|request.focus.as_deref()==Some(root_path)).or_else(||profile_candidates.first());
    let profile_source=selected_profile_source.map(|(root_path,_)|root_path.as_str());
    let profile_program=selected_profile_source.map(|(_,project_path)|project_path.clone());
    let receipts = store::list(root)
        .unwrap_or_else(|error| json!({"records":[],"status":"unknown","notice":error}));
    graph.observations(&evidence,&receipts,&fingerprint,&mut budget);
    let value = json!({"profile_source":profile_source,"receipts":receipts,"version":1,"behavior_protocol":1,"namespace":root.canonicalize().map_err(|e|e.to_string())?.to_string_lossy(),"workspace_id":scan.workspace_id,"reading_fingerprint":fingerprint,"context":request,"construction":construction,"graph":graph,"impact":impact,"evidence":evidence,"verification_available":!scan.workspace_id.is_empty()&&authority.is_ok(),"evidence_limit":"Source hashes bind captured bytes; they do not prove execution inputs were frozen. Manual and legacy-origin records do not become automated evidence."});
    Ok(Built {
        value,
        analysis,
        scan: Some(scan),
        profile_program,
    })
}
</ui:copy>

## Keep declared readings usable after compilation fails

Failed compilation must not hide an independently readable contract or explanation. Keep those previews and declared checks, mark construction impact unknown, and withhold execution authority until saved construction inputs can be established.

<ui:copy id="behavior-readings-unavailable">
fn unavailable(root: &Path, request: &ReadingRequest, error: String) -> Result<Built, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let workspace = super::resource_inputs::RootIdentity::fingerprint(&root).unwrap_or_default();
    let mut graph = Graph::default();
    let map = metadata(&root, "hickory-dogfood/behavior-map.json", &mut graph);
    let narrative = metadata(&root, "hickory-dogfood/narrative-map.json", &mut graph);
    let mut paths: BTreeSet<String> = request.sources.keys().cloned().collect();
    paths.insert(request.entry.clone());
    if let Some(focus) = &request.focus {
        paths.insert(focus.clone());
    }
    graph.metadata(&map, &narrative, &paths);
    graph.notices.push(format!("Construction unavailable: {error}. Declared relationships remain readable; construction impact and execution authority are unknown."));
    let mut hashes = BTreeMap::new();
    let mut budget = 1024 * 1024usize;
    for path in &paths {
        let bytes = request
            .sources
            .get(path)
            .map(|source| source.as_bytes().to_vec())
            .or_else(|| safe_read(&root, path, 4 * 1024 * 1024).ok().flatten());
        if let Some(bytes) = bytes {
            hashes.insert(path.clone(), hash(&bytes));
        }
    }
    budget=attach_source_previews(&root,request,None,&mut graph);
    let fingerprint=hash(&serde_json::to_vec(&json!({"workspace":workspace,"context":request,"graph":graph,"construction_error":error})).map_err(|e|e.to_string())?);
    let evidence = crate::coverage::reading_evidence(&root, &workspace, &hashes).unwrap_or_else(
        |error| json!({"status":"unknown","records":[],"notice":error.to_string()}),
    );
    let impact = request.focus.as_deref().map(|focus| {
        let mut impact = graph.impact(focus);
        impact["construction_status"] = json!("unknown");
        impact["construction_error"] = json!(error);
        impact
    });
    let source = request
        .sources
        .get(&request.entry)
        .cloned()
        .unwrap_or_default();
    let analysis = source_tests::Request {
        path: request.entry.clone(),
        source,
        project_entry: Some(request.entry.clone()),
        sources: request.sources.clone(),
        capture: false,
    };
    let receipts = store::list(&root)
        .unwrap_or_else(|error| json!({"records":[],"status":"unknown","notice":error}));
    graph.observations(&evidence,&receipts,&fingerprint,&mut budget);
    let value = json!({"version":1,"behavior_protocol":1,"namespace":root.to_string_lossy(),"workspace_id":workspace,"reading_fingerprint":fingerprint,"context":request,"graph":graph,"impact":impact,"evidence":evidence,"receipts":receipts,"verification_available":false,"construction":{"status":"unavailable","error":error},"evidence_limit":"Construction failed; declared relationships and captured snippets remain readable. No zero-impact or execution-authority claim is made."});
    Ok(Built {
        value,
        analysis,
        scan: None,
        profile_program:None,
    })
}
</ui:copy>

## Bind physical verification to known constructed inputs

Saved source-candidate Run executes the physical workspace. Checking only the
authored pages and selected test file would permit a stale same-named generated
dependency to run. Revalidate every known regular compiler output byte-for-byte
before this provider starts, streaming binary assets without text coercion.
Captured-profile Run instead retains its existing immutable-tree preparation.
These checks are admission observations, not proof of frozen execution inputs.

<ui:copy id="behavior-readings-execution-inputs">
fn checked_execution_parts(root:&Path,context:&ReadingRequest,sources:&BTreeMap<String,String>,tree:Option<&hick_lang::ProjectTree>,physical_outputs:bool)->Result<BTreeMap<String,String>,String>{
 retained_source_authority(root,context,sources)?;
 let mut checked=source_tests::check_saved_sources(root,sources).map_err(|error|error.detail().to_owned())?;
 if physical_outputs {use sha2::{Digest,Sha256};use std::io::Read;let base=Path::new(&context.entry).parent().unwrap_or(Path::new(""));let tree=tree.ok_or("Construction input authority is unavailable.")?;for (path,entry) in tree.entries(){if !matches!(entry.kind,hick_lang::EntryKind::File){continue}let path=source_tests::relative(base,path)?;let saved=crate::coverage::safe_path(root,&path).map_err(|error|format!("Known constructed output {path} is missing or foreign: {error}. Materialize the reviewed saved construction or explicitly choose a captured profile."))?;let metadata=std::fs::metadata(&saved).map_err(|error|error.to_string())?;if !metadata.is_file()||metadata.len()!=entry.bytes.len() as u64{return Err(format!("Known constructed output {path} differs from the reviewed construction. No verification was started."))}let mut file=std::fs::File::open(saved).map_err(|error|error.to_string())?;let mut digest=Sha256::new();let mut buffer=[0u8;65536];loop{let count=file.read(&mut buffer).map_err(|error|error.to_string())?;if count==0{break}digest.update(&buffer[..count]);}let actual=hex::encode(digest.finalize());if actual!=hash(&entry.bytes){return Err(format!("Known constructed output {path} changed. Refresh/materialize the reviewed construction before verification."))}checked.insert(path,actual);}}
 Ok(checked)
}
fn terminal_receipt(value:&Value)->Value {let fields=["id","title","cwd","state","since_ms","monitor","cwd_is_live","exit_code"];Value::Object(fields.into_iter().filter_map(|key|value.get(key).map(|value|(key.to_string(),if let Some(text)=value.as_str(){json!(text.chars().take(if key=="cwd"{4096}else{512}).collect::<String>())}else{value.clone()}))).collect())}
</ui:copy>

## Revalidate and consume an exact execution intent

Every request binds workspace, reading fingerprint, selected check, exact profile and captured context. Persist the full typed intent fingerprint before delegating to saved Run or exact-profile Run. A reused ID can inspect only that same intent. Process completion never fabricates a collected test verdict.

<ui:copy id="behavior-readings-execution">
pub async fn reading(State(state):State<LocalState>,Json(request):Json<ReadingRequest>)->ApiResult<Json<Value>>{
 let engine=root_lease(&state)?;let root=state.index.root().to_path_buf();
 let mut value=tokio::task::spawn_blocking(move||build(&root,&request).map(|b|b.value)).await.map_err(|e|ApiError::internal(e.to_string()))?.map_err(ApiError::unprocessable)?;
 root_current(&state,&engine)?;value["engine_workspace_id"]=json!(engine);Ok(Json(value))
}

pub async fn verify(
    State(state): State<LocalState>,
    Json(request): Json<VerifyRequest>,
) -> ApiResult<Json<Value>> {
    if request.version != 1 {
        return Err(ApiError::conflict(
            "Unsupported behavior verification protocol.",
        ));
    }
    let request_fingerprint =
        intent_fingerprint(&request).map_err(|e| ApiError::bad_request(e.to_string()))?;
    if let Some(owned)=store::owned(&state.resource_root,&state.terminals,&request.request_id).map_err(ApiError::conflict)? {
        if owned.value["request_fingerprint"]!=request_fingerprint{return Err(ApiError::conflict("This request ID belongs to another complete verification intention."));}
        return observe_owned(&state,&request.request_id,owned).map(Json);
    }
    let engine=root_lease(&state)?;root_current(&state,&request.engine_workspace_id)?;
    let root = state.index.root().to_path_buf();
    let context = request.context.clone();
    let build_root = root.clone();
    let built = tokio::task::spawn_blocking(move || build(&build_root, &context))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map_err(ApiError::unprocessable)?;
    root_current(&state,&engine)?;
    if built.value["verification_available"]!=true{return Err(ApiError::conflict("Physical source or construction authority is unavailable. Captured readings remain readable; verification was not started."));}
    if built.value["workspace_id"] != request.workspace_id
        || built.value["reading_fingerprint"] != request.reading_fingerprint
    {
        return Err(ApiError::conflict(
            "Reading inputs or metadata changed. Refresh before verification; no check was started.",
        ));
    }
    if let Some(receipt) =
        store::existing(&root, &request.request_id).map_err(ApiError::unprocessable)?
    {
        if receipt["request_fingerprint"] != request_fingerprint {
            return Err(ApiError::conflict(
                "This request ID belongs to a different verification. Inspect its retained receipt.",
            ));
        }
        root_current(&state,&engine)?;
        let mut historical=receipt;if historical["terminal"].is_object(){historical["terminal_snapshot"]=historical["terminal"].clone();historical["terminal"]=Value::Null;}historical["notice"]=json!("Historical consumed request. This engine has no positively owned process for it; no replacement was started.");return Ok(Json(historical));
    }
    let check = built.value["graph"]["checks"]
        .as_array()
        .and_then(|checks| checks.iter().find(|c| c["id"] == request.check_id))
        .ok_or_else(|| ApiError::conflict("Check is not in the captured reading."))?;
    if request.profile.is_none() && check["available"] != true {
        return Err(ApiError::unprocessable(check["reason"].as_str().unwrap_or(
            "This declared check has no admitted execution provider.",
        )));
    }
    let candidate = check["candidate_id"].as_str().unwrap_or("").to_string();
    let scan=built.scan.as_ref().ok_or_else(||ApiError::conflict("Construction input authority is unavailable. Repair or refresh it before verification."))?;
    let guard_root=root.clone();let guard_context=request.context.clone();let guard_sources=scan.source_hashes.clone();let guard_tree=scan.tree.clone();let physical_outputs=request.profile.is_none();
    let checked_sources=tokio::task::spawn_blocking(move||checked_execution_parts(&guard_root,&guard_context,&guard_sources,guard_tree.as_ref(),physical_outputs)).await.map_err(|error|ApiError::internal(error.to_string()))?.map_err(ApiError::conflict)?;
    root_current(&state,&engine)?;
    let mut receipt = json!({"version":1,"engine_workspace_id":engine,"workspace_identity_kind":"physical-root-v1","request_id":request.request_id,"request_fingerprint":request_fingerprint,"workspace_id":request.workspace_id,"reading_fingerprint":request.reading_fingerprint,"check_id":request.check_id,"label":check["label"],"profile":request.profile,"relationship":if request.profile.is_some(){"user-declared profile for this check"}else{"detected declaration in explicitly related module"},"started_ms":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis(),"status":"admitting","outcome":"unknown","source_hashes":checked_sources,"execution_inputs_frozen":false,"notice":"Process-only verification; no collected test verdict is inferred."});
    if let Some(owned)=store::claim_owned(&state.resource_root,&state.terminals,&engine,&request.request_id,&receipt).map_err(ApiError::conflict)? {return observe_owned(&state,&request.request_id,owned).map(Json);}
    if let Err(error)=store::reserve(&root,&request.request_id,&receipt){receipt["status"]=json!("request-refused");receipt["error"]=json!(error);store::update_owned(&state.resource_root,&state.terminals,&request.request_id,None,receipt.clone(),None,false).map_err(ApiError::internal)?;return Ok(Json(receipt));}
    store::disk_reserved(&state.resource_root,&state.terminals,&request.request_id).map_err(ApiError::internal)?;
    if let Err(error)=root_current(&state,&engine){receipt["status"]=json!("request-refused");receipt["error"]=json!(error.detail());let owned=store::update_owned(&state.resource_root,&state.terminals,&request.request_id,None,receipt,None,false).map_err(ApiError::internal)?;return observe_owned(&state,&request.request_id,owned).map(Json);}

    let profile=request.profile.clone();let query=if profile.is_some(){Some(profile_query(&request.context,&built)?)}else{None};let provider=state.clone();let provider_engine=engine.clone();let runtime=tokio::runtime::Handle::current();let fingerprint=scan.fingerprint.clone();let analysis=built.analysis;
    let result=tokio::task::spawn_blocking(move||runtime.block_on(async move {
    root_current(&provider,&provider_engine)?;
    if let Some(profile) = profile {
        super::launch_profiles::run(
            State(provider),
            Json(super::launch_profiles::Run {
                context: query.ok_or_else(||ApiError::conflict("Captured profile context is unavailable"))?,
                profile,
            }),
        )
        .await
    } else {
        source_tests::run(
            State(provider),
            Json(source_tests::RunRequest {
                version: 1,
                input: source_tests::RunInput::Saved {},
                analysis,
                fingerprint,
                candidate_id: candidate,
                selection: source_tests::Selection::NameFilter {},
            }),
        )
        .await
    }
    })).await.unwrap_or_else(|error|Err(ApiError::internal(error.to_string())));
    match result {
        Ok(Json(value)) => {
            receipt["status"] = json!("process-running");
            receipt["terminal"] = terminal_receipt(&value);
        }
        Err(error) => {
            receipt["status"] = json!("admission-unknown");
            receipt["error"] = json!(error.detail().chars().take(4096).collect::<String>());
        }
    }
    let known=receipt["terminal"]["id"].as_str().filter(|id|state.terminals.get(id).is_some()).map(str::to_owned);
    let owned=store::update_owned(&state.resource_root,&state.terminals,&request.request_id,None,receipt,known,false).map_err(ApiError::internal)?;
    observe_owned(&state,&request.request_id,owned).map(Json)
}
</ui:copy>

## Refresh one retained process without starting work

A request reserves bounded engine-owned memory before dispatch. Refresh uses only a terminal returned by that admitted provider and retained under the exact root handle and terminal hub. Disk records supply historical observations, never live process ownership. Losing the source directory retires source authority while this independently owned process remains readable. Refresh stores an abbreviated observation only while the original root is current. It never asks the background check monitor for this checkout's regression status or retries an uncertain process.

<ui:copy id="behavior-readings-observation">
pub async fn verification(
    State(state): State<LocalState>,
    Query(query): Query<VerificationQuery>,
) -> ApiResult<Json<Value>> {
    if let Some(owned)=store::owned(&state.resource_root,&state.terminals,&query.id).map_err(ApiError::unprocessable)? {return observe_owned(&state,&query.id,owned).map(Json);}
    let engine=root_lease(&state)?;
    let mut receipt=store::existing(state.index.root(),&query.id).map_err(ApiError::unprocessable)?.ok_or_else(||ApiError::not_found("Unknown verification request. No check was retried."))?;
    root_current(&state,&engine)?;
    if receipt["terminal"].is_object(){receipt["terminal_snapshot"]=receipt["terminal"].clone();receipt["terminal"]=Value::Null;}
    receipt["notice"]=json!("Historical receipt only. This engine has no positively owned process for this request; no replacement was adopted or restarted.");
    Ok(Json(receipt))
}

fn observe_owned(state:&LocalState,id:&str,snapshot:store::OwnedSnapshot)->ApiResult<Value>{
    let mut receipt=snapshot.value;
    if let Some(terminal)=snapshot.known_terminal.as_deref().and_then(|id|state.terminals.get(id)){
        receipt["terminal"]=terminal_receipt(&json!(terminal.summary()));
        let output=terminal.screen_text();receipt["output"]=json!(output.chars().take(65536).collect::<String>());receipt["output_truncated"]=json!(output.chars().count()>65536);
        if let Some(exit)=terminal.exit_code(){receipt["status"]=json!("process-finished");receipt["exit_code"]=json!(exit);receipt["outcome"]=json!("process-only");}
    }else if snapshot.known_terminal.is_some(){receipt["terminal_snapshot"]=receipt["terminal"].clone();receipt["terminal"]=Value::Null;if receipt["status"]!="process-finished"{receipt["status"]=json!("process-unavailable");receipt["outcome"]=json!("unknown");}}
    let current=state.resource_root.check(state.index.root()).is_ok();receipt["workspace_current"]=json!(current);
    if !current{receipt["source_origin"]=json!("historical-unverified");receipt["notice"]=json!("Workspace physical identity changed. Only this engine's positively owned process remains observable; source relationships are historical and no new execution is authorized.");}
    let observed=store::update_owned(&state.resource_root,&state.terminals,id,Some(snapshot.revision),receipt,None,false).map_err(ApiError::internal)?;
    // Memory is authoritative for owned processes. Persist only under the still-held original root.
    if current&&snapshot.disk_reserved&&state.resource_root.check(state.index.root()).is_ok()&&store::save(state.index.root(),id,&observed.value).is_ok(){store::acknowledge_persisted(&state.resource_root,&state.terminals,id,observed.revision).map_err(ApiError::internal)?;}
    Ok(observed.value)
}
</ui:copy>

## Consume execution requests before crossing the process boundary

Persist an admitting receipt before Run. A crash or uncertain response cannot authorize a second process under the same request, and an unrelated workspace cannot adopt a copied receipt. Explicit refresh reads only that receipt's known terminal; this service never asks the background check system for checkout regression state. Terminal completion remains a process-only observation.

<ui:copy id="behavior-readings-store">
use super::*;
use std::io::Write;
fn path(root: &Path, id: &str, create: bool) -> Result<std::path::PathBuf, String> {
    if id.len() != 32
        || !id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("Verification ID must be 32 lowercase hexadecimal characters.".into());
    }
    let mut dir = root.canonicalize().map_err(|e| e.to_string())?;
    for part in [".hick-cache", "behavior-verification"] {
        dir.push(part);
        if create && !dir.exists() {
            std::fs::create_dir(&dir).map_err(|e| e.to_string())?
        }
        match std::fs::symlink_metadata(&dir) {
            Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => {}
            Ok(_) => {
                return Err("Verification storage cannot be a symlink or non-directory.".into());
            }
            Err(e) if !create && e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(dir.join(format!("{id}.json")))
}
fn read_retained(root: &Path, id: &str) -> Result<Option<Value>, String> {
    let path = path(root, id, false)?;
    let meta = match std::fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    if meta.file_type().is_symlink() || !meta.is_file() || meta.len() > 2 * 1024 * 1024 {
        return Err("Verification receipt is unsafe or too large.".into());
    }
    let value: Value = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if value["request_id"]!=id{return Err("Verification receipt request identity changed.".into());}
    Ok(Some(value))
}
pub(super) fn existing(root:&Path,id:&str)->Result<Option<Value>,String>{let value=read_retained(root,id)?;if let Some(value)=&value{if value["workspace_identity_kind"]!="physical-root-v1"||value["workspace_id"]!=super::super::resource_inputs::RootIdentity::fingerprint(root)?{return Err("Historical or foreign verification origin is unconfirmed; no request was replayed.".into());}}Ok(value)}
pub(super) fn reserve(root: &Path, id: &str, value: &Value) -> Result<(), String> {
    let path = path(root, id, true)?;
    let count = std::fs::read_dir(path.parent().unwrap())
        .map_err(|e| e.to_string())?
        .take(4097)
        .count();
    if count >= 4096 {
        return Err("Verification receipt store is full. Retained receipts were not evicted; no check was started.".into());
    }
    let bytes=serde_json::to_vec(value).map_err(|error|error.to_string())?;
    if bytes.len()>1024*1024{return Err("Verification input receipt exceeds the retained-capacity budget. No check was started.".into())}
    let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e|format!("Verification request is already consumed or storage is unavailable: {e}. Inspect its receipt; do not retry with a new ID."))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}
pub(super) fn save(root: &Path, id: &str, value: &Value) -> Result<(), String> {
    let path = path(root, id, true)?;
    let bytes=serde_json::to_vec(value).map_err(|error|error.to_string())?;
    if bytes.len()>2*1024*1024{return Err("Verification observation exceeds its retained budget. The previous consumed receipt remains authoritative; no process was retried.".into())}
    let mut file =
        tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}
pub(super) fn list(root: &Path) -> Result<Value, String> {
    let directory = path(root, "00000000000000000000000000000000", false)?
        .parent()
        .unwrap()
        .to_path_buf();
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(json!({"status":"missing","records":[],"truncated":false}));
        }
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    let mut budget = 16 * 1024 * 1024u64;
    let mut truncated = false;
    let mut unknown = 0;
    for entry in entries.take(4097) {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(id) = name.strip_suffix(".json") else {
            continue;
        };
        let meta = std::fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
        if meta.len() > budget || records.len() >= 100 {
            truncated = true;
            break;
        }
        budget -= meta.len();
        match read_retained(root, id) {
            Ok(Some(mut record)) => {if record["terminal"].is_object(){record["terminal_snapshot"]=record["terminal"].clone();record["terminal"]=Value::Null;}let current=record["workspace_identity_kind"]=="physical-root-v1"&&super::super::resource_inputs::RootIdentity::fingerprint(root).ok().is_some_and(|identity|record["workspace_id"]==identity);if current{record["origin"]=json!("current-workspace");}else{record["origin"]=json!("unknown");record["terminal"]=Value::Null;record["freshness"]=json!("origin-unknown");record["notice"]=json!("Retained historical origin is unconfirmed; no live process or replay authority is attached.");}records.push(record)},
            Ok(None) => {}
            Err(_) => unknown += 1,
        }
    }
    records.sort_by_key(|record| std::cmp::Reverse(record["started_ms"].as_u64().unwrap_or(0)));
    Ok(
        json!({"status":"retained","records":records,"truncated":truncated,"unknown_records":unknown}),
    )
}
use std::sync::{Arc,Mutex,OnceLock,Weak};
type RootScope=super::super::resource_inputs::RootIdentity;
struct OwnedProcess {scope:Arc<RootScope>,hub:Weak<hick_term::Terminals>,runtime:String,id:String,intent:String,value:Value,known_terminal:Option<String>,revision:u64,quota:usize,disk_reserved:bool,persisted:bool}
#[derive(Default)]struct OwnedProcesses {rows:std::collections::VecDeque<OwnedProcess>,bytes:usize}
static OWNED:OnceLock<Mutex<OwnedProcesses>>=OnceLock::new();
#[derive(Clone)]pub(super) struct OwnedSnapshot {pub value:Value,pub known_terminal:Option<String>,pub revision:u64,pub runtime:String,pub disk_reserved:bool}
impl OwnedProcess {fn snapshot(&self)->OwnedSnapshot{OwnedSnapshot{value:self.value.clone(),known_terminal:self.known_terminal.clone(),revision:self.revision,runtime:self.runtime.clone(),disk_reserved:self.disk_reserved}}fn belongs(&self,scope:&Arc<RootScope>,hub:&Arc<hick_term::Terminals>)->bool{Arc::ptr_eq(&self.scope,scope)&&self.hub.upgrade().is_some_and(|owned|Arc::ptr_eq(&owned,hub))}}
fn owned_cache()->Result<std::sync::MutexGuard<'static,OwnedProcesses>,String>{OWNED.get_or_init(||Mutex::new(OwnedProcesses::default())).lock().map_err(|_|"Owned verification retention is unavailable; no untracked process is authorized.".into())}
pub(super) fn owned(scope:&Arc<RootScope>,hub:&Arc<hick_term::Terminals>,id:&str)->Result<Option<OwnedSnapshot>,String>{Ok(owned_cache()?.rows.iter().find(|row|row.id==id&&row.belongs(scope,hub)).map(OwnedProcess::snapshot))}
/// None means this exact engine-owned intention was reserved; Some is an existing replay.
pub(super) fn claim_owned(scope:&Arc<RootScope>,hub:&Arc<hick_term::Terminals>,runtime:&str,id:&str,value:&Value)->Result<Option<OwnedSnapshot>,String>{
 if id.len()!=32||!id.bytes().all(|byte|byte.is_ascii_digit()||(b'a'..=b'f').contains(&byte)){return Err("Verification request identity is malformed.".into())}
 let intent=value["request_fingerprint"].as_str().filter(|value|value.len()==64).ok_or("Retained verification has no full intention fingerprint.")?;
 let mut cache=owned_cache()?;if let Some(row)=cache.rows.iter().find(|row|row.id==id&&row.belongs(scope,hub)){if row.intent!=intent{return Err("This request ID belongs to a different verification intention.".into())}return Ok(Some(row.snapshot()))}
 let encoded=serde_json::to_vec(value).map_err(|error|error.to_string())?.len();if encoded>1024*1024{return Err("Verification intention exceeds the bounded retained input budget; no process was started.".into())}let quota=encoded+512*1024;
 while cache.rows.len()>=256||cache.bytes.saturating_add(quota)>32*1024*1024{let index=cache.rows.iter().position(|row|row.hub.upgrade().is_none()||row.persisted&&matches!(row.value["status"].as_str(),Some("process-finished"|"admission-refused"|"request-refused"))).ok_or("Owned verification retention is full. Refresh or stop the existing process requests before starting another; uncertain requests were not evicted.")?;let row=cache.rows.remove(index).unwrap();cache.bytes-=row.quota;}
 let mut value=value.clone();value["observation_revision"]=json!(0);value["observation_engine_workspace_id"]=json!(runtime);
 cache.rows.push_back(OwnedProcess{scope:scope.clone(),hub:Arc::downgrade(hub),runtime:runtime.into(),id:id.into(),intent:intent.into(),value,known_terminal:None,revision:0,quota,disk_reserved:false,persisted:false});cache.bytes+=quota;Ok(None)
}
pub(super) fn disk_reserved(scope:&Arc<RootScope>,hub:&Arc<hick_term::Terminals>,id:&str)->Result<(),String>{let mut cache=owned_cache()?;let row=cache.rows.iter_mut().find(|row|row.id==id&&row.belongs(scope,hub)).ok_or("Owned request disappeared before persistence.")?;row.disk_reserved=true;row.persisted=true;Ok(())}
pub(super) fn acknowledge_persisted(scope:&Arc<RootScope>,hub:&Arc<hick_term::Terminals>,id:&str,revision:u64)->Result<(),String>{let mut cache=owned_cache()?;let row=cache.rows.iter_mut().find(|row|row.id==id&&row.belongs(scope,hub)).ok_or("Owned persistence acknowledgement lost its request.")?;if row.revision==revision{row.persisted=true;}Ok(())}
/// Updates are CAS observations; a held older read cannot replace a newer final outcome.
pub(super) fn update_owned(scope:&Arc<RootScope>,hub:&Arc<hick_term::Terminals>,id:&str,expected:Option<u64>,mut value:Value,known_terminal:Option<String>,persisted:bool)->Result<OwnedSnapshot,String>{
 let mut cache=owned_cache()?;let row=cache.rows.iter_mut().find(|row|row.id==id&&row.belongs(scope,hub)).ok_or("The original engine no longer owns this verification request.")?;
 if expected.is_some_and(|expected|expected!=row.revision){return Ok(row.snapshot())}
 if value["request_fingerprint"]!=row.intent{return Err("Observed process no longer matches its complete captured intention.".into())}
 if row.value["status"]=="process-finished"&&value["status"]!="process-finished"{return Ok(row.snapshot())}
 if let Some(id)=known_terminal{if row.known_terminal.as_ref().is_some_and(|known|known!=&id){return Err("An owned request cannot adopt another process identity.".into())}row.known_terminal=Some(id)}
 row.revision=row.revision.saturating_add(1);value["observation_revision"]=json!(row.revision);value["observation_engine_workspace_id"]=json!(row.runtime);
 if serde_json::to_vec(&value).map_err(|error|error.to_string())?.len()>row.quota{value["output"]=Value::Null;value["output_truncated"]=json!(true);if let Some(id)=&row.known_terminal{value["terminal"]=json!({"id":id});}}
 if serde_json::to_vec(&value).map_err(|error|error.to_string())?.len()>row.quota{return Err("Process metadata exceeds its pre-reserved retention budget. The original owned request remains retained.".into())}
 row.value=value;row.persisted=persisted;Ok(row.snapshot())
}
</ui:copy>

## Reject duplicate and foreign request records

The owned temporary store demonstrates that a consumed request cannot reserve another process and a copied foreign-workspace receipt cannot supply identity authority.

<ui:copy id="behavior-readings-store-tests">
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_intentions_survive_root_replacement_without_borrowing_another_engine_or_process(){
        let temp=tempfile::tempdir().unwrap();let path=temp.path().join("workspace");std::fs::create_dir(&path).unwrap();
        let scope=Arc::new(RootScope::bind(&path));let hub=Arc::new(hick_term::Terminals::new(Default::default()));let id="abcdefabcdefabcdefabcdefabcdefab";let value=json!({"request_fingerprint":"a".repeat(64),"status":"admitting"});
        assert!(claim_owned(&scope,&hub,"11111111111111111111111111111111",id,&value).unwrap().is_none());
        let initial=owned(&scope,&hub,id).unwrap().unwrap();
        let mut running=value.clone();running["status"]=json!("process-running");let running=update_owned(&scope,&hub,id,Some(initial.revision),running,Some("positively-returned-process".into()),false).unwrap();
        std::fs::rename(&path,temp.path().join("retired")).unwrap();std::fs::create_dir(&path).unwrap();assert!(scope.check(&path).is_err());
        assert_eq!(owned(&scope,&hub,id).unwrap().unwrap().known_terminal.as_deref(),Some("positively-returned-process"));
        let replacement=Arc::new(RootScope::bind(&path));assert!(owned(&replacement,&hub,id).unwrap().is_none());let foreign=Arc::new(hick_term::Terminals::new(Default::default()));assert!(owned(&scope,&foreign,id).unwrap().is_none());
        let mut changed=value.clone();changed["request_fingerprint"]=json!("b".repeat(64));assert!(claim_owned(&scope,&hub,"11111111111111111111111111111111",id,&changed).is_err());
        assert!(update_owned(&scope,&hub,id,None,running.value.clone(),Some("foreign-process".into()),false).is_err());
        let mut final_value=running.value.clone();final_value["status"]=json!("process-finished");let finished=update_owned(&scope,&hub,id,Some(running.revision),final_value,None,false).unwrap();
        assert_eq!(update_owned(&scope,&hub,id,Some(running.revision),running.value,None,false).unwrap().revision,finished.revision);
    }

    #[test]
    fn unretainable_inputs_refuse_before_consuming_the_request(){let root=tempfile::tempdir().unwrap();let id="11111111111111111111111111111111";assert!(reserve(root.path(),id,&json!({"checked_sources":"x".repeat(1024*1024+1)})).is_err());assert!(!path(root.path(),id,false).unwrap().exists());let value=json!({"workspace_identity_kind":"physical-root-v1","workspace_id":super::super::super::resource_inputs::RootIdentity::fingerprint(root.path()).unwrap(),"request_id":id,"status":"admitting"});reserve(root.path(),id,&value).unwrap();assert!(existing(root.path(),id).unwrap().is_some());}

    #[test]
    fn consumed_requests_do_not_reexecute_and_foreign_receipts_refuse() {
        let root = tempfile::tempdir().unwrap();
        let id = "0123456789abcdef0123456789abcdef";
        let value = json!({"request_id":id,"workspace_identity_kind":"physical-root-v1","workspace_id":super::super::super::resource_inputs::RootIdentity::fingerprint(root.path()).unwrap(),"status":"admitting"});
        reserve(root.path(), id, &value).unwrap();
        assert!(reserve(root.path(), id, &value).is_err());
        assert_eq!(
            existing(root.path(), id).unwrap().unwrap()["status"],
            "admitting"
        );
        let other = tempfile::tempdir().unwrap();
        let mut foreign = value;
        foreign["workspace_id"] = json!(hash(other.path().as_os_str().as_encoded_bytes()));
        save(root.path(), id, &foreign).unwrap();
        assert!(existing(root.path(), id).is_err());
        assert!(existing(root.path(), "../escape").is_err());
    }
    #[test]
    fn disk_receipts_never_grant_live_terminal_authority_even_with_current_origin(){let root=tempfile::tempdir().unwrap();let id="33333333333333333333333333333333";let value=json!({"workspace_identity_kind":"physical-root-v1","workspace_id":RootScope::fingerprint(root.path()).unwrap(),"engine_workspace_id":"e".repeat(32),"request_id":id,"terminal":{"id":"forged-owned-id"},"status":"process-running"});reserve(root.path(),id,&value).unwrap();let value=list(root.path()).unwrap();assert_eq!(value["records"][0]["origin"],"current-workspace");assert!(value["records"][0]["terminal"].is_null());assert_eq!(value["records"][0]["terminal_snapshot"]["id"],"forged-owned-id");}

    #[test]
    fn legacy_path_only_origins_remain_readable_without_replay_or_terminal_authority(){let root=tempfile::tempdir().unwrap();let id="22222222222222222222222222222222";let old=json!({"version":1,"request_id":id,"workspace_id":hash(root.path().canonicalize().unwrap().as_os_str().as_encoded_bytes()),"label":"Legacy result","status":"process-finished","terminal":{"id":"old-process"}});reserve(root.path(),id,&old).unwrap();assert!(existing(root.path(),id).is_err());let value=list(root.path()).unwrap();let row=&value["records"][0];assert_eq!(row["label"],"Legacy result");assert_eq!(row["origin"],"unknown");assert!(row["terminal"].is_null());}

}
</ui:copy>
<ui:file path="crates/hickory-cli/src/serve/behavior_readings.rs"><ui:paste select="#behavior-readings-service"/><ui:paste select="#behavior-readings-profiles"/><ui:paste select="#behavior-readings-snapshots"/><ui:paste select="#behavior-readings-acquisition"/><ui:paste select="#behavior-readings-unavailable"/><ui:paste select="#behavior-readings-execution-inputs"/><ui:paste select="#behavior-readings-execution"/><ui:paste select="#behavior-readings-observation"/><ui:paste select="#behavior-readings-source"/><ui:paste select="#behavior-readings-failure-tests"/><ui:paste select="#behavior-readings-source-tests"/></ui:file>
<ui:file path="crates/hickory-cli/src/serve/behavior_readings/store.rs"><ui:paste select="#behavior-readings-store"/><ui:paste select="#behavior-readings-store-tests"/></ui:file>

## Failed compilation is unknown impact, not an empty relationship graph

Metadata and captured previews remain independently readable when a draft is invalid or a contributor disappears. Without an established saved construction authority, execution is disabled. The retained declared check remains unsupported rather than being silently dropped.

<ui:copy id="behavior-readings-failure-tests">
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn valid_original_guard_does_not_authorize_unsaved_unselected_plain_text(){let temp=tempfile::tempdir().unwrap();let root=temp.path();std::fs::write(root.join("notes.txt"),"saved").unwrap();let guard=super::super::source_history::read_observed(root,&root.join("notes.txt"),"notes.txt").unwrap().save_guard.unwrap();let mut context=ReadingRequest{version:1,entry:"project.md".into(),sources:BTreeMap::from([("notes.txt".into(),"saved".into())]),missing_sources:BTreeSet::new(),unaddressed_sources:BTreeMap::new(),saved_guards:BTreeMap::from([("notes.txt".into(),guard)]),focus:None};assert!(retained_source_authority(root,&context,&BTreeMap::new()).is_ok());context.sources.insert("notes.txt".into(),"unsaved".into());assert!(retained_source_authority(root,&context,&BTreeMap::new()).unwrap_err().contains("unsaved"));assert_eq!(std::fs::read_to_string(root.join("notes.txt")).unwrap(),"saved");}
 #[test] fn attach_profiles_remain_explicitly_unavailable_for_relevant_run(){let value=verification_profiles(json!({"fingerprint":"inventory","profiles":[{"id":"external","name":"Inspect existing process","attach":{"host":"127.0.0.1","port":5678}},{"id":"manual","name":"Run declared profile"}],"notices":["Provider notice"]})).unwrap();assert_eq!(value["profiles"].as_array().unwrap().len(),1);assert_eq!(value["profiles"][0]["id"],"manual");assert!(value["notices"].as_array().unwrap().iter().any(|notice|notice.as_str().is_some_and(|notice|notice.contains("attach profile unavailable"))));}
 #[test] fn retained_constructions_require_selected_named_context_and_original_guards(){let root=tempfile::tempdir().unwrap();let mut context=ReadingRequest{version:1,entry:"project.md".into(),sources:BTreeMap::new(),missing_sources:BTreeSet::new(),unaddressed_sources:BTreeMap::new(),saved_guards:BTreeMap::new(),focus:None};context.unaddressed_sources.insert("opaque-owner".into(),"ordinary unsaved notes".into());assert!(retained_source_authority(root.path(),&context,&BTreeMap::new()).is_ok());let declaration="<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:include path=\"elsewhere.md\"/></h:doc>";context.unaddressed_sources.insert("opaque-owner".into(),declaration.into());assert!(retained_source_authority(root.path(),&context,&BTreeMap::new()).unwrap_err().contains("unaddressed"));context.unaddressed_sources.clear();context.sources.insert("construction.unusual".into(),declaration.into());assert!(retained_source_authority(root.path(),&context,&BTreeMap::new()).unwrap_err().contains("Originally loaded"));context.saved_guards.insert("construction.unusual".into(),"original-token".into());assert!(retained_source_authority(root.path(),&context,&BTreeMap::new()).unwrap_err().contains("outside the selected"));}
 #[test] fn exact_profile_observation_anchor_uses_a_typed_regular_compiler_output(){let temporary=tempfile::tempdir().unwrap();let root=temporary.path();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\"/></h:doc>").unwrap();std::fs::write(root.join("part.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"literal%2F #.py\">value=41</h:file></h:doc>").unwrap();let request=ReadingRequest{version:1,unaddressed_sources:BTreeMap::new(),saved_guards:BTreeMap::new(),entry:"project.md".into(),sources:BTreeMap::new(),missing_sources:BTreeSet::new(),focus:Some("literal%2F #.py".into())};let built=build(root,&request).unwrap();assert_eq!(built.value["profile_source"],"literal%2F #.py","{}",built.value);let query=profile_query(&request,&built).unwrap();assert_eq!(query.program.as_deref(),Some("literal%2F #.py"));assert_eq!(query.uri,"hick:///project.md");}
 #[test] fn saved_verification_rejects_stale_known_outputs_and_hashes_binary_assets_without_text_coercion(){let temporary=tempfile::tempdir().unwrap();let root=temporary.path();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\"/></h:doc>").unwrap();std::fs::write(root.join("part.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"a.py\">value=41</h:file></h:doc>").unwrap();std::fs::write(root.join("a.py"),"value=41").unwrap();let context=ReadingRequest{version:1,unaddressed_sources:BTreeMap::new(),saved_guards:BTreeMap::new(),entry:"project.md".into(),sources:BTreeMap::new(),missing_sources:BTreeSet::new(),focus:Some("a.py".into())};let mut built=build(root,&context).unwrap();let scan=built.scan.as_mut().unwrap();let binary=vec![0,255,128,1];scan.tree.as_mut().unwrap().insert("asset.bin",hick_lang::ProjectEntry::file(binary.clone())).unwrap();std::fs::write(root.join("asset.bin"),binary).unwrap();assert!(checked_execution_parts(root,&context,&scan.source_hashes,scan.tree.as_ref(),true).unwrap().contains_key("asset.bin"));std::fs::write(root.join("a.py"),"value=99").unwrap();assert!(checked_execution_parts(root,&context,&scan.source_hashes,scan.tree.as_ref(),true).is_err());assert!(checked_execution_parts(root,&context,&scan.source_hashes,scan.tree.as_ref(),false).is_ok());}
 #[test] fn request_identity_binds_the_complete_typed_context_and_profile(){let context=json!({"version":1,"entry":"project.md","sources":{"a.md":"one"},"missing_sources":[],"focus":"a.py"});let base=json!({"version":1,"engine_workspace_id":"engine","workspace_id":"workspace","reading_fingerprint":"reading","check_id":"check","request_id":"0123456789abcdef0123456789abcdef","context":context,"profile":{"id":"one","fingerprint":"inventory"}});let fingerprint=intent_fingerprint(&serde_json::from_value(base.clone()).unwrap()).unwrap();for mutation in ["profile","context","check","workspace"]{let mut changed=base.clone();match mutation{"profile"=>changed["profile"]["id"]=json!("two"),"context"=>changed["context"]["sources"]["a.md"]=json!("two"),"check"=>changed["check_id"]=json!("different"),_=>changed["workspace_id"]=json!("different")};assert_ne!(intent_fingerprint(&serde_json::from_value(changed).unwrap()).unwrap(),fingerprint);}}
 #[test] fn invalid_drafts_and_deleted_contributors_keep_declared_readings_but_retire_execution(){
  let temporary=tempfile::tempdir().unwrap();let root=temporary.path();std::fs::create_dir(root.join("hickory-dogfood")).unwrap();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"source.md\"/></h:doc>").unwrap();std::fs::write(root.join("source.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"a.py\">value=41</h:file></h:doc>").unwrap();std::fs::write(root.join("behavior#%2F.md"),"Readable behavior contract").unwrap();std::fs::write(root.join("hickory-dogfood/behavior-map.json"),json!({"format":1,"contracts":{"c":{"page":"behavior#%2F.md","title":"Readable behavior","references":["source.md"],"tests":[{"runner":"external","file":"unavailable","name":"declared check"}]}}}).to_string()).unwrap();
  let mut request=ReadingRequest{version:1,unaddressed_sources:BTreeMap::new(),saved_guards:BTreeMap::new(),entry:"project.md".into(),sources:BTreeMap::from([("source.md".into(),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"../outside.py\">value=41</h:file></h:doc>".into())]),missing_sources:BTreeSet::new(),focus:Some("source.md".into())};
  let invalid=build(root,&request).unwrap();assert_eq!(invalid.value["verification_available"],false);assert_eq!(invalid.value["construction"]["status"],"unavailable");assert!(invalid.value["graph"]["nodes"].as_object().unwrap().values().any(|node|node["preview"].as_str().is_some_and(|text|text.contains("Readable behavior contract"))));assert!(invalid.scan.is_none());
  request.sources.clear();std::fs::remove_file(root.join("source.md")).unwrap();let deleted=build(root,&request).unwrap();assert_eq!(deleted.value["verification_available"],false);assert!(!deleted.value["graph"]["checks"].as_array().unwrap().is_empty());assert!(deleted.value["impact"]["construction_error"].is_string());
 }
}
</ui:copy>

## Continue a captured source reading without replacing the editor

Abbreviations are not a complete reading. The bound page request revalidates workspace, context, graph fingerprint, node and whole-text hash before returning a scalar-indexed page. A unique observed parser span locates a declared fragment; missing or ambiguous declarations remain explicit. Constructed and deleted-baseline bytes come from compiler snapshots, while supplied drafts, saved sources and retained observation summaries retain separate origins.

<ui:copy id="behavior-readings-source">
#[derive(Clone,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRequest {pub version:u32,pub workspace_id:String,pub reading_fingerprint:String,pub node_id:String,pub source_hash:String,pub context:ReadingRequest,pub offset:Option<usize>}
fn compiled_text(scan:&source_tests::Scan,entry:&str,path:&str)->Option<Vec<u8>>{let base=Path::new(entry).parent().unwrap_or(Path::new(""));scan.tree.as_ref()?.entries().iter().find(|(key,_)|source_tests::relative(base,key).ok().as_deref()==Some(path)).and_then(|(_,entry)|matches!(entry.kind,hick_lang::EntryKind::File).then(||entry.bytes.to_vec()))}
fn source_page(root:&Path,request:SourceRequest)->Result<Value,String>{
 if request.version!=1{return Err("Unsupported bound source-reading protocol.".into())}let built=build(root,&request.context)?;if built.value["workspace_id"]!=request.workspace_id||built.value["reading_fingerprint"]!=request.reading_fingerprint{return Err("Captured construction, metadata or workspace changed. Existing text remains readable; refresh before reading another page.".into())}
 let node=built.value["graph"]["nodes"].get(&request.node_id).ok_or("This node is not in the captured reading.")?;if node["hash"]!=request.source_hash{return Err("Captured node text changed or is unavailable. No later same-named source was substituted.".into())}
 let (bytes,origin)=if let Some(path)=node["source_path"].as_str(){
  if node["status"]=="saved-baseline-absent-from-draft"{let source=String::from_utf8(safe_read(root,&request.context.entry,8*1024*1024)?.ok_or("Saved construction entry disappeared.")?).map_err(|_|"Saved entry is not UTF-8.")?;let analysis=source_tests::Request{path:request.context.entry.clone(),source,project_entry:Some(request.context.entry.clone()),sources:BTreeMap::new(),capture:false};let saved=source_tests::scan_with_sources(root,&analysis,None)?;(compiled_text(&saved,&request.context.entry,path).ok_or("Retained saved baseline is unavailable; absence was not replaced by another source.")?,"saved-baseline")}
  else if node["status"]=="constructed"||node["kind"]=="check"||node["kind"]=="test-module"{if let Some(bytes)=built.scan.as_ref().and_then(|scan|compiled_text(scan,&request.context.entry,path)){(bytes,"constructed")}else if let Some(source)=request.context.sources.get(path){(source.as_bytes().to_vec(),"captured-draft")}else if let Some(source)=built.scan.as_ref().and_then(|scan|scan.source_texts.get(path)){(source.as_bytes().to_vec(),"saved-source")}else{(safe_read(root,path,8*1024*1024)?.ok_or("This source is missing; no replacement text was acquired.")?,"saved-source")}}
  else if let Some(source)=request.context.sources.get(path){(source.as_bytes().to_vec(),"captured-draft")}else if let Some(source)=built.scan.as_ref().and_then(|scan|scan.source_texts.get(path)){(source.as_bytes().to_vec(),"saved-source")}else{(safe_read(root,path,8*1024*1024)?.ok_or("This source is missing; no replacement text was acquired.")?,"saved-source")}
 }else{let record=built.value["evidence"]["records"].as_array().into_iter().flatten().find(|record|record["id"].as_str().is_some_and(|record_id|model::id("evidence",&format!("coverage:{record_id}"))==request.node_id)).or_else(||built.value["receipts"]["records"].as_array().into_iter().flatten().find(|record|record["request_id"].as_str().is_some_and(|id|model::id("verification",&format!("request:{id}"))==request.node_id))).ok_or("Retained observation summary changed or disappeared.")?;(record.to_string().into_bytes(),"retained-summary")};
 if hash(&bytes)!=request.source_hash{return Err("Captured text hash changed. Existing text remains readable; no newer same-named text was substituted.".into())}let source=String::from_utf8(bytes).map_err(|_|"Binary data has no source-text reading.")?;
 let mut target_status="whole-source";let mut start=0usize;if let Some(fragment)=node["fragment_id"].as_str(){let starts:Vec<_>=hick_lang::parse(&source).ok().map(|document|document.all_tags().into_iter().filter(|tag|tag.get_attribute("id")==Some(fragment)).filter_map(|tag|tag.source_span.filter(|span|span.file_id.is_none()).map(|span|span.start)).collect()).unwrap_or_default();if starts.len()==1{if let Some(prefix)=source.get(..starts[0]){start=prefix.chars().count().saturating_sub(800);target_status="observed-fragment-context";}}else{target_status=if starts.is_empty(){"missing-declared-fragment"}else{"ambiguous-declared-fragment"};}}
 let total=source.chars().count();let offset=request.offset.unwrap_or(start);if offset>total{return Err("Source page offset exceeds the captured scalar range.".into())}let text:String=source.chars().skip(offset).take(4000).collect();let count=text.chars().count();let next=(offset+count<total).then_some(offset+count);let page_hash=hash(text.as_bytes());Ok(json!({"version":1,"behavior_protocol":1,"workspace_id":request.workspace_id,"reading_fingerprint":request.reading_fingerprint,"node_id":request.node_id,"source_hash":request.source_hash,"page_hash":page_hash,"source_path":node["source_path"],"fragment_id":node["fragment_id"],"origin":origin,"target_status":target_status,"state":if origin=="saved-baseline"{"historical-deleted-baseline"}else{"captured"},"units":"unicode-scalars","offset":offset,"total":total,"next":next,"text":text,"notice":"Captured text only; current editor changes and later disk bytes are not silently adopted. Retained observation summaries are not raw collector reports or test verdicts."}))
}
pub async fn source(State(state):State<LocalState>,Json(request):Json<SourceRequest>)->ApiResult<Json<Value>>{let engine=root_lease(&state)?;let root=state.index.root().to_path_buf();let value=tokio::task::spawn_blocking(move||source_page(&root,request).map(Json)).await.map_err(|error|ApiError::internal(error.to_string()))?.map_err(ApiError::conflict)?;root_current(&state,&engine)?;Ok(value)}
</ui:copy>

## Prove full readings across late fragments, scalar limits and stale context

The owned source fixture places the declared explanation beyond the abbreviated
preview. Tests page non-BMP text without splitting scalars, reject foreign/stale
identities and ranges, retain invalid-draft contract readings, and reconstruct a
deleted draft output only from its saved compiler baseline without materializing it.

<ui:copy id="behavior-readings-source-tests">
#[cfg(test)] mod source_page_tests {
 use super::*;
 fn fixture()->(tempfile::TempDir,ReadingRequest){let temporary=tempfile::tempdir().unwrap();let root=temporary.path();std::fs::create_dir(root.join("hickory-dogfood")).unwrap();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"literal#source.md\"/></h:doc>").unwrap();let source=format!("<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n{}\n## Explanation beyond the abbreviation\n\nThis is the explicitly declared later explanation.\n<h:copy id=\"later\">\ndef answer():\n    return 41\n</h:copy>\n<h:file path=\"a.py\"><h:paste select=\"#later\"/></h:file>\n</h:doc>","Earlier retained prose 😀.\n".repeat(240));std::fs::write(root.join("literal#source.md"),source).unwrap();std::fs::write(root.join("contract#literal.md"),format!("{}end","😀".repeat(4100))).unwrap();std::fs::write(root.join("hickory-dogfood/behavior-map.json"),json!({"format":1,"contracts":{"answer":{"page":"contract#literal.md","disposition":"retained-requirement","title":"Answer behavior","references":["a.py"]}}}).to_string()).unwrap();std::fs::write(root.join("hickory-dogfood/narrative-map.json"),json!({"format":1,"fragments":[{"id":"later","page":"literal#source.md","output":"a.py","symbol":"answer","basis":"needs-explanation"}]}).to_string()).unwrap();(temporary,ReadingRequest{version:1,unaddressed_sources:BTreeMap::new(),saved_guards:BTreeMap::new(),entry:"project.md".into(),sources:BTreeMap::new(),missing_sources:BTreeSet::new(),focus:Some("a.py".into())})}
 fn query(root:&Path,context:&ReadingRequest,kind:&str)->SourceRequest{let built=build(root,context).unwrap();let node=built.value["graph"]["nodes"].as_object().unwrap().values().find(|node|node["kind"]==kind).unwrap();SourceRequest{version:1,workspace_id:built.value["workspace_id"].as_str().unwrap().into(),reading_fingerprint:built.value["reading_fingerprint"].as_str().unwrap().into(),node_id:node["id"].as_str().unwrap().into(),source_hash:node["hash"].as_str().unwrap().into(),context:context.clone(),offset:None}}
 #[test] fn source_pages_reach_later_declared_context_and_preserve_literal_paths(){let (temporary,context)=fixture();let value=source_page(temporary.path(),query(temporary.path(),&context,"explanation")).unwrap();assert!(value["offset"].as_u64().unwrap()>2000);assert!(value["text"].as_str().unwrap().contains("explicitly declared later explanation"));assert_eq!(value["source_path"],"literal#source.md");assert_eq!(value["fragment_id"],"later");assert_eq!(value["target_status"],"observed-fragment-context");assert_eq!(value["page_hash"],hash(value["text"].as_str().unwrap().as_bytes()));assert!(!temporary.path().join("a.py").exists());}
 #[test] fn scalar_pages_are_bounded_and_refuse_foreign_stale_and_out_of_range_reads(){let (temporary,context)=fixture();let request=query(temporary.path(),&context,"contract");let value=source_page(temporary.path(),request.clone()).unwrap();assert_eq!(value["text"].as_str().unwrap().chars().count(),4000);assert_eq!(value["next"],4000);let mut next=request.clone();next.offset=Some(4000);let tail=source_page(temporary.path(),next).unwrap();assert_eq!(tail["text"].as_str().unwrap().chars().count(),103);for change in ["workspace","reading","node","source","offset"]{let mut changed=request.clone();match change{"workspace"=>changed.workspace_id="f".repeat(64),"reading"=>changed.reading_fingerprint="f".repeat(64),"node"=>changed.node_id="unknown".into(),"source"=>changed.source_hash="f".repeat(64),_=>changed.offset=Some(999999)}assert!(source_page(temporary.path(),changed).is_err());}std::fs::write(temporary.path().join("contract#literal.md"),"newer same-named disk text").unwrap();assert!(source_page(temporary.path(),request).is_err());}
 #[test] fn deleted_draft_outputs_read_saved_compiler_baselines_and_invalid_drafts_keep_contracts_readable(){let (temporary,mut context)=fixture();let original=std::fs::read_to_string(temporary.path().join("literal#source.md")).unwrap();context.sources.insert("literal#source.md".into(),original.replace("path=\"a.py\"","path=\"b.py\""));let built=build(temporary.path(),&context).unwrap();let node=built.value["graph"]["nodes"].as_object().unwrap().values().find(|node|node["status"]=="saved-baseline-absent-from-draft").unwrap();let request=SourceRequest{version:1,workspace_id:built.value["workspace_id"].as_str().unwrap().into(),reading_fingerprint:built.value["reading_fingerprint"].as_str().unwrap().into(),node_id:node["id"].as_str().unwrap().into(),source_hash:node["hash"].as_str().unwrap().into(),context:context.clone(),offset:None};let value=source_page(temporary.path(),request).unwrap();assert_eq!(value["origin"],"saved-baseline");assert_eq!(value["state"],"historical-deleted-baseline");assert!(value["text"].as_str().unwrap().contains("return 41"));context.sources.insert("literal#source.md".into(),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"../escape\">invalid</h:file></h:doc>".into());let built=build(temporary.path(),&context).unwrap();assert_eq!(built.value["verification_available"],false);assert!(source_page(temporary.path(),query(temporary.path(),&context,"contract")).unwrap()["text"].as_str().unwrap().contains('😀'));}
}
</ui:copy>
## Focused service evidence

The canonical shared behavior-reading suite passed thirteen focused cases on
2026-11-16, including typed compiler anchors, literal fragment identity,
independent declared readings after construction failure/deletion, full-intent
request binding, known-output freshness and bounded complete scalar pages.
The owned real-engine browser witness additionally passed selected candidate
and explicit profile execution, genuine collector/JUnit observations, stale and
foreign refusal, full later-fragment reading and retained process-only outcomes.
Only the fixture's receipts were inspected; no checkout post-commit or broad
regression state was read. Browser/native physical-input and accessibility
acceptance are separate and remain unverified by these checks.

</ui:doc>
