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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) entry_selection: Option<super::workspace_entry_authority::EntrySelection>,
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
pub(crate) fn check_selected_entry(
    state: &LocalState,
    address: &str,
    proof: Option<&super::workspace_entry_authority::EntrySelection>,
) -> ApiResult<()> {
    if let Some(proof) = proof {
        if proof.entry.path.as_deref() != Some(address) {
            return Err(ApiError::conflict("The original workspace entry proof belongs to another construction address."));
        }
        super::workspace_entry_authority::validate_entry_selection(state, proof)
            .map_err(ApiError::conflict)?;
    }
    Ok(())
}
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
    check_selected_entry(&state, &request.context.entry, request.context.entry_selection.as_ref())?;
    let root = state.index.root().to_path_buf();
    let context = request.context.clone();
    let build_root = root.clone();
    let built = tokio::task::spawn_blocking(move || build(&build_root, &context))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map_err(ApiError::unprocessable)?;
    root_current(&state,&engine)?;
    check_selected_entry(&state, &request.context.entry, request.context.entry_selection.as_ref())?;
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
    let value=tokio::task::spawn_blocking(move||runtime.block_on(super::launch_profiles::list(State(provider),Json(query)))).await.map_err(|error|ApiError::internal(error.to_string()))??;root_current(&state,&engine)?;
    check_selected_entry(&state, &request.context.entry, request.context.entry_selection.as_ref())?;
    Ok(Json(verification_profiles(value.0).map_err(ApiError::unprocessable)?))
}
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
            entry_selection: None,
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
fn checked_execution_parts(root:&Path,context:&ReadingRequest,sources:&BTreeMap<String,String>,tree:Option<&hick_lang::ProjectTree>,physical_outputs:bool)->Result<BTreeMap<String,String>,String>{
 retained_source_authority(root,context,sources)?;
 let mut checked=source_tests::check_saved_sources(root,sources).map_err(|error|error.detail().to_owned())?;
 if physical_outputs {use sha2::{Digest,Sha256};use std::io::Read;let base=Path::new(&context.entry).parent().unwrap_or(Path::new(""));let tree=tree.ok_or("Construction input authority is unavailable.")?;for (path,entry) in tree.entries(){if !matches!(entry.kind,hick_lang::EntryKind::File){continue}let path=source_tests::relative(base,path)?;let saved=crate::coverage::safe_path(root,&path).map_err(|error|format!("Known constructed output {path} is missing or foreign: {error}. Materialize the reviewed saved construction or explicitly choose a captured profile."))?;let metadata=std::fs::metadata(&saved).map_err(|error|error.to_string())?;if !metadata.is_file()||metadata.len()!=entry.bytes.len() as u64{return Err(format!("Known constructed output {path} differs from the reviewed construction. No verification was started."))}let mut file=std::fs::File::open(saved).map_err(|error|error.to_string())?;let mut digest=Sha256::new();let mut buffer=[0u8;65536];loop{let count=file.read(&mut buffer).map_err(|error|error.to_string())?;if count==0{break}digest.update(&buffer[..count]);}let actual=hex::encode(digest.finalize());if actual!=hash(&entry.bytes){return Err(format!("Known constructed output {path} changed. Refresh/materialize the reviewed construction before verification."))}checked.insert(path,actual);}}
 Ok(checked)
}
fn terminal_receipt(value:&Value)->Value {let fields=["id","title","cwd","state","since_ms","monitor","cwd_is_live","exit_code"];Value::Object(fields.into_iter().filter_map(|key|value.get(key).map(|value|(key.to_string(),if let Some(text)=value.as_str(){json!(text.chars().take(if key=="cwd"{4096}else{512}).collect::<String>())}else{value.clone()}))).collect())}
pub async fn reading(State(state):State<LocalState>,Json(request):Json<ReadingRequest>)->ApiResult<Json<Value>>{
 let engine=root_lease(&state)?;let root=state.index.root().to_path_buf();
 let entry=request.entry.clone();
 let proof=request.entry_selection.clone();
 check_selected_entry(&state, &entry, proof.as_ref())?;
 let mut value=tokio::task::spawn_blocking(move||build(&root,&request).map(|b|b.value)).await.map_err(|e|ApiError::internal(e.to_string()))?.map_err(ApiError::unprocessable)?;
 root_current(&state,&engine)?;
 check_selected_entry(&state, &entry, proof.as_ref())?;
 value["engine_workspace_id"]=json!(engine);Ok(Json(value))
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
    check_selected_entry(&state, &request.context.entry, request.context.entry_selection.as_ref())?;
    let root = state.index.root().to_path_buf();
    let context = request.context.clone();
    let build_root = root.clone();
    let built = tokio::task::spawn_blocking(move || build(&build_root, &context))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map_err(ApiError::unprocessable)?;
    root_current(&state,&engine)?;
    check_selected_entry(&state, &request.context.entry, request.context.entry_selection.as_ref())?;
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
    check_selected_entry(&state, &request.context.entry, request.context.entry_selection.as_ref())?;
    let mut receipt = json!({"version":1,"engine_workspace_id":engine,"workspace_identity_kind":"physical-root-v1","request_id":request.request_id,"request_fingerprint":request_fingerprint,"workspace_id":request.workspace_id,"reading_fingerprint":request.reading_fingerprint,"check_id":request.check_id,"label":check["label"],"profile":request.profile,"relationship":if request.profile.is_some(){"user-declared profile for this check"}else{"detected declaration in explicitly related module"},"started_ms":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis(),"status":"admitting","outcome":"unknown","source_hashes":checked_sources,"execution_inputs_frozen":false,"notice":"Process-only verification; no collected test verdict is inferred."});
    if let Some(owned)=store::claim_owned(&state.resource_root,&state.terminals,&engine,&request.request_id,&receipt).map_err(ApiError::conflict)? {return observe_owned(&state,&request.request_id,owned).map(Json);}
    if let Err(error)=store::reserve(&root,&request.request_id,&receipt){receipt["status"]=json!("request-refused");receipt["error"]=json!(error);store::update_owned(&state.resource_root,&state.terminals,&request.request_id,None,receipt.clone(),None,false).map_err(ApiError::internal)?;return Ok(Json(receipt));}
    store::disk_reserved(&state.resource_root,&state.terminals,&request.request_id).map_err(ApiError::internal)?;
    if let Err(error)=root_current(&state,&engine){receipt["status"]=json!("request-refused");receipt["error"]=json!(error.detail());let owned=store::update_owned(&state.resource_root,&state.terminals,&request.request_id,None,receipt,None,false).map_err(ApiError::internal)?;return observe_owned(&state,&request.request_id,owned).map(Json);}

    let profile=request.profile.clone();let query=if profile.is_some(){Some(profile_query(&request.context,&built)?)}else{None};let provider=state.clone();let provider_engine=engine.clone();let runtime=tokio::runtime::Handle::current();let fingerprint=scan.fingerprint.clone();let analysis=built.analysis;
    let entry_address = request.context.entry.clone();
    let entry_selection = request.context.entry_selection.clone();
    let result=tokio::task::spawn_blocking(move||runtime.block_on(async move {
    root_current(&provider,&provider_engine)?;
    check_selected_entry(&provider, &entry_address, entry_selection.as_ref())?;
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
pub async fn source(State(state):State<LocalState>,Json(request):Json<SourceRequest>)->ApiResult<Json<Value>>{let engine=root_lease(&state)?;
 let address=request.context.entry.clone();let proof=request.context.entry_selection.clone();
 check_selected_entry(&state, &address, proof.as_ref())?;
 let root=state.index.root().to_path_buf();
 let value=tokio::task::spawn_blocking(move||source_page(&root,request).map(Json)).await.map_err(|error|ApiError::internal(error.to_string()))?.map_err(ApiError::conflict)?;
 root_current(&state,&engine)?;
 check_selected_entry(&state, &address, proof.as_ref())?;
 Ok(value)}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn valid_original_guard_does_not_authorize_unsaved_unselected_plain_text(){let temp=tempfile::tempdir().unwrap();let root=temp.path();std::fs::write(root.join("notes.txt"),"saved").unwrap();let guard=super::super::source_history::read_observed(root,&root.join("notes.txt"),"notes.txt").unwrap().save_guard.unwrap();let mut context=ReadingRequest{version:1,entry_selection:None,entry:"project.md".into(),sources:BTreeMap::from([("notes.txt".into(),"saved".into())]),missing_sources:BTreeSet::new(),unaddressed_sources:BTreeMap::new(),saved_guards:BTreeMap::from([("notes.txt".into(),guard)]),focus:None};assert!(retained_source_authority(root,&context,&BTreeMap::new()).is_ok());context.sources.insert("notes.txt".into(),"unsaved".into());assert!(retained_source_authority(root,&context,&BTreeMap::new()).unwrap_err().contains("unsaved"));assert_eq!(std::fs::read_to_string(root.join("notes.txt")).unwrap(),"saved");}
 #[test] fn attach_profiles_remain_explicitly_unavailable_for_relevant_run(){let value=verification_profiles(json!({"fingerprint":"inventory","profiles":[{"id":"external","name":"Inspect existing process","attach":{"host":"127.0.0.1","port":5678}},{"id":"manual","name":"Run declared profile"}],"notices":["Provider notice"]})).unwrap();assert_eq!(value["profiles"].as_array().unwrap().len(),1);assert_eq!(value["profiles"][0]["id"],"manual");assert!(value["notices"].as_array().unwrap().iter().any(|notice|notice.as_str().is_some_and(|notice|notice.contains("attach profile unavailable"))));}
 #[test] fn retained_constructions_require_selected_named_context_and_original_guards(){let root=tempfile::tempdir().unwrap();let mut context=ReadingRequest{version:1,entry_selection:None,entry:"project.md".into(),sources:BTreeMap::new(),missing_sources:BTreeSet::new(),unaddressed_sources:BTreeMap::new(),saved_guards:BTreeMap::new(),focus:None};context.unaddressed_sources.insert("opaque-owner".into(),"ordinary unsaved notes".into());assert!(retained_source_authority(root.path(),&context,&BTreeMap::new()).is_ok());let declaration="<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:include path=\"elsewhere.md\"/></h:doc>";context.unaddressed_sources.insert("opaque-owner".into(),declaration.into());assert!(retained_source_authority(root.path(),&context,&BTreeMap::new()).unwrap_err().contains("unaddressed"));context.unaddressed_sources.clear();context.sources.insert("construction.unusual".into(),declaration.into());assert!(retained_source_authority(root.path(),&context,&BTreeMap::new()).unwrap_err().contains("Originally loaded"));context.saved_guards.insert("construction.unusual".into(),"original-token".into());assert!(retained_source_authority(root.path(),&context,&BTreeMap::new()).unwrap_err().contains("outside the selected"));}
 #[test] fn exact_profile_observation_anchor_uses_a_typed_regular_compiler_output(){let temporary=tempfile::tempdir().unwrap();let root=temporary.path();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\"/></h:doc>").unwrap();std::fs::write(root.join("part.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"literal%2F #.py\">value=41</h:file></h:doc>").unwrap();let request=ReadingRequest{version:1,entry_selection:None,unaddressed_sources:BTreeMap::new(),saved_guards:BTreeMap::new(),entry:"project.md".into(),sources:BTreeMap::new(),missing_sources:BTreeSet::new(),focus:Some("literal%2F #.py".into())};let built=build(root,&request).unwrap();assert_eq!(built.value["profile_source"],"literal%2F #.py","{}",built.value);let query=profile_query(&request,&built).unwrap();assert_eq!(query.program.as_deref(),Some("literal%2F #.py"));assert_eq!(query.uri,"hick:///project.md");}
 #[test] fn saved_verification_rejects_stale_known_outputs_and_hashes_binary_assets_without_text_coercion(){let temporary=tempfile::tempdir().unwrap();let root=temporary.path();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\"/></h:doc>").unwrap();std::fs::write(root.join("part.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"a.py\">value=41</h:file></h:doc>").unwrap();std::fs::write(root.join("a.py"),"value=41").unwrap();let context=ReadingRequest{version:1,entry_selection:None,unaddressed_sources:BTreeMap::new(),saved_guards:BTreeMap::new(),entry:"project.md".into(),sources:BTreeMap::new(),missing_sources:BTreeSet::new(),focus:Some("a.py".into())};let mut built=build(root,&context).unwrap();let scan=built.scan.as_mut().unwrap();let binary=vec![0,255,128,1];scan.tree.as_mut().unwrap().insert("asset.bin",hick_lang::ProjectEntry::file(binary.clone())).unwrap();std::fs::write(root.join("asset.bin"),binary).unwrap();assert!(checked_execution_parts(root,&context,&scan.source_hashes,scan.tree.as_ref(),true).unwrap().contains_key("asset.bin"));std::fs::write(root.join("a.py"),"value=99").unwrap();assert!(checked_execution_parts(root,&context,&scan.source_hashes,scan.tree.as_ref(),true).is_err());assert!(checked_execution_parts(root,&context,&scan.source_hashes,scan.tree.as_ref(),false).is_ok());}
 #[test] fn request_identity_binds_the_complete_typed_context_and_profile(){let context=json!({"version":1,"entry":"project.md","sources":{"a.md":"one"},"missing_sources":[],"focus":"a.py"});let base=json!({"version":1,"engine_workspace_id":"engine","workspace_id":"workspace","reading_fingerprint":"reading","check_id":"check","request_id":"0123456789abcdef0123456789abcdef","context":context,"profile":{"id":"one","fingerprint":"inventory"}});let fingerprint=intent_fingerprint(&serde_json::from_value(base.clone()).unwrap()).unwrap();for mutation in ["profile","context","check","workspace"]{let mut changed=base.clone();match mutation{"profile"=>changed["profile"]["id"]=json!("two"),"context"=>changed["context"]["sources"]["a.md"]=json!("two"),"check"=>changed["check_id"]=json!("different"),_=>changed["workspace_id"]=json!("different")};assert_ne!(intent_fingerprint(&serde_json::from_value(changed).unwrap()).unwrap(),fingerprint);}}
 #[test] fn invalid_drafts_and_deleted_contributors_keep_declared_readings_but_retire_execution(){
  let temporary=tempfile::tempdir().unwrap();let root=temporary.path();std::fs::create_dir(root.join("hickory-dogfood")).unwrap();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"source.md\"/></h:doc>").unwrap();std::fs::write(root.join("source.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"a.py\">value=41</h:file></h:doc>").unwrap();std::fs::write(root.join("behavior#%2F.md"),"Readable behavior contract").unwrap();std::fs::write(root.join("hickory-dogfood/behavior-map.json"),json!({"format":1,"contracts":{"c":{"page":"behavior#%2F.md","title":"Readable behavior","references":["source.md"],"tests":[{"runner":"external","file":"unavailable","name":"declared check"}]}}}).to_string()).unwrap();
  let mut request=ReadingRequest{version:1,entry_selection:None,unaddressed_sources:BTreeMap::new(),saved_guards:BTreeMap::new(),entry:"project.md".into(),sources:BTreeMap::from([("source.md".into(),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"../outside.py\">value=41</h:file></h:doc>".into())]),missing_sources:BTreeSet::new(),focus:Some("source.md".into())};
  let invalid=build(root,&request).unwrap();assert_eq!(invalid.value["verification_available"],false);assert_eq!(invalid.value["construction"]["status"],"unavailable");assert!(invalid.value["graph"]["nodes"].as_object().unwrap().values().any(|node|node["preview"].as_str().is_some_and(|text|text.contains("Readable behavior contract"))));assert!(invalid.scan.is_none());
  request.sources.clear();std::fs::remove_file(root.join("source.md")).unwrap();let deleted=build(root,&request).unwrap();assert_eq!(deleted.value["verification_available"],false);assert!(!deleted.value["graph"]["checks"].as_array().unwrap().is_empty());assert!(deleted.value["impact"]["construction_error"].is_string());
 }
}
#[cfg(test)] mod source_page_tests {
 use super::*;
 fn fixture()->(tempfile::TempDir,ReadingRequest){let temporary=tempfile::tempdir().unwrap();let root=temporary.path();std::fs::create_dir(root.join("hickory-dogfood")).unwrap();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"literal#source.md\"/></h:doc>").unwrap();let source=format!("<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n{}\n## Explanation beyond the abbreviation\n\nThis is the explicitly declared later explanation.\n<h:copy id=\"later\">\ndef answer():\n    return 41\n</h:copy>\n<h:file path=\"a.py\"><h:paste select=\"#later\"/></h:file>\n</h:doc>","Earlier retained prose 😀.\n".repeat(240));std::fs::write(root.join("literal#source.md"),source).unwrap();std::fs::write(root.join("contract#literal.md"),format!("{}end","😀".repeat(4100))).unwrap();std::fs::write(root.join("hickory-dogfood/behavior-map.json"),json!({"format":1,"contracts":{"answer":{"page":"contract#literal.md","disposition":"retained-requirement","title":"Answer behavior","references":["a.py"]}}}).to_string()).unwrap();std::fs::write(root.join("hickory-dogfood/narrative-map.json"),json!({"format":1,"fragments":[{"id":"later","page":"literal#source.md","output":"a.py","symbol":"answer","basis":"needs-explanation"}]}).to_string()).unwrap();(temporary,ReadingRequest{version:1,entry_selection:None,unaddressed_sources:BTreeMap::new(),saved_guards:BTreeMap::new(),entry:"project.md".into(),sources:BTreeMap::new(),missing_sources:BTreeSet::new(),focus:Some("a.py".into())})}
 fn query(root:&Path,context:&ReadingRequest,kind:&str)->SourceRequest{let built=build(root,context).unwrap();let node=built.value["graph"]["nodes"].as_object().unwrap().values().find(|node|node["kind"]==kind).unwrap();SourceRequest{version:1,workspace_id:built.value["workspace_id"].as_str().unwrap().into(),reading_fingerprint:built.value["reading_fingerprint"].as_str().unwrap().into(),node_id:node["id"].as_str().unwrap().into(),source_hash:node["hash"].as_str().unwrap().into(),context:context.clone(),offset:None}}
 #[test] fn source_pages_reach_later_declared_context_and_preserve_literal_paths(){let (temporary,context)=fixture();let value=source_page(temporary.path(),query(temporary.path(),&context,"explanation")).unwrap();assert!(value["offset"].as_u64().unwrap()>2000);assert!(value["text"].as_str().unwrap().contains("explicitly declared later explanation"));assert_eq!(value["source_path"],"literal#source.md");assert_eq!(value["fragment_id"],"later");assert_eq!(value["target_status"],"observed-fragment-context");assert_eq!(value["page_hash"],hash(value["text"].as_str().unwrap().as_bytes()));assert!(!temporary.path().join("a.py").exists());}
 #[test] fn scalar_pages_are_bounded_and_refuse_foreign_stale_and_out_of_range_reads(){let (temporary,context)=fixture();let request=query(temporary.path(),&context,"contract");let value=source_page(temporary.path(),request.clone()).unwrap();assert_eq!(value["text"].as_str().unwrap().chars().count(),4000);assert_eq!(value["next"],4000);let mut next=request.clone();next.offset=Some(4000);let tail=source_page(temporary.path(),next).unwrap();assert_eq!(tail["text"].as_str().unwrap().chars().count(),103);for change in ["workspace","reading","node","source","offset"]{let mut changed=request.clone();match change{"workspace"=>changed.workspace_id="f".repeat(64),"reading"=>changed.reading_fingerprint="f".repeat(64),"node"=>changed.node_id="unknown".into(),"source"=>changed.source_hash="f".repeat(64),_=>changed.offset=Some(999999)}assert!(source_page(temporary.path(),changed).is_err());}std::fs::write(temporary.path().join("contract#literal.md"),"newer same-named disk text").unwrap();assert!(source_page(temporary.path(),request).is_err());}
 #[test] fn deleted_draft_outputs_read_saved_compiler_baselines_and_invalid_drafts_keep_contracts_readable(){let (temporary,mut context)=fixture();let original=std::fs::read_to_string(temporary.path().join("literal#source.md")).unwrap();context.sources.insert("literal#source.md".into(),original.replace("path=\"a.py\"","path=\"b.py\""));let built=build(temporary.path(),&context).unwrap();let node=built.value["graph"]["nodes"].as_object().unwrap().values().find(|node|node["status"]=="saved-baseline-absent-from-draft").unwrap();let request=SourceRequest{version:1,workspace_id:built.value["workspace_id"].as_str().unwrap().into(),reading_fingerprint:built.value["reading_fingerprint"].as_str().unwrap().into(),node_id:node["id"].as_str().unwrap().into(),source_hash:node["hash"].as_str().unwrap().into(),context:context.clone(),offset:None};let value=source_page(temporary.path(),request).unwrap();assert_eq!(value["origin"],"saved-baseline");assert_eq!(value["state"],"historical-deleted-baseline");assert!(value["text"].as_str().unwrap().contains("return 41"));context.sources.insert("literal#source.md".into(),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"../escape\">invalid</h:file></h:doc>".into());let built=build(temporary.path(),&context).unwrap();assert_eq!(built.value["verification_available"],false);assert!(source_page(temporary.path(),query(temporary.path(),&context,"contract")).unwrap()["text"].as_str().unwrap().contains('😀'));}
}
