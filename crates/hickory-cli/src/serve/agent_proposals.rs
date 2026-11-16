//! One immutable agent intention; source effects remain with existing publishers.
use super::{LocalState, api::{ApiError,ApiResult}, resource_inputs, resource_changes, resource_candidates, behavior_readings};
use axum::{Json,extract::State};
use serde::{Deserialize,Serialize};
use serde_json::{Value,json};
use std::{collections::{BTreeMap,BTreeSet,VecDeque},sync::{Arc,Mutex,LazyLock}};
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intention {
 pub version:u32,
 pub request_id:String,
 pub origin:String,
 pub title:String,
 pub explanation:String,
 #[serde(default)] pub claims:Vec<String>,
 #[serde(default)] pub check_ids:Vec<String>,
 pub capture:resource_inputs::Request,
 pub reading:behavior_readings::ReadingRequest,
}
fn ordinary(state:&LocalState)->ApiResult<()>{if state.private_prediction.is_some()||crate::engine::private_policy::active().map_err(|error|ApiError::conflict(error.to_string()))?{return Err(ApiError::conflict("Private prediction requires its isolated proposal adapter; ordinary proposal readings, tools and application admission are unavailable."))}Ok(())}
fn hash(value:&impl Serialize)->Result<String,String>{serde_json::to_vec(value).map(|bytes|super::workspace_fs::view::hash(&bytes)).map_err(|error|error.to_string())}
fn bounded(value:&Intention)->Result<(),String>{
 if value.version!=1||!super::resource_saves::hex(&value.request_id,32)||value.request_id.bytes().any(|b|b.is_ascii_uppercase()){return Err("Proposal requires version1 and a lowercase intention identity.".into())}
 if value.title.trim().is_empty()||value.title.len()>512||value.origin.is_empty()||value.origin.len()>1024||value.explanation.trim().is_empty()||value.explanation.len()>65536||value.claims.len()>128||value.claims.iter().any(|claim|claim.len()>8192)||value.check_ids.len()>256{return Err("Proposal explanation or claim set is missing or exceeds its review budget.".into())}
 if serde_json::to_vec(value).map_err(|e|e.to_string())?.len()>32*1024*1024{return Err("Proposal exceeds its bounded complete-intention budget.".into())}
 if value.check_ids.iter().collect::<BTreeSet<_>>().len()!=value.check_ids.len(){return Err("Proposal repeats a check identity.".into())}
 Ok(())
}
#[derive(Clone,Serialize)]
struct Review {
 token:String,fingerprint:String,material:String,origin_authority:String,
 intention:Intention,workspace:String,namespace:String,plan:resource_changes::ResourcePlan,
}
#[derive(Default)]
struct Retained {order:VecDeque<String>,records:BTreeMap<String,(Arc<Review>,usize)>,applications:BTreeMap<String,Value>,bytes:usize}
static REVIEWS:LazyLock<Mutex<Retained>>=LazyLock::new(||Mutex::new(Retained::default()));
fn nonce()->Result<String,String>{let mut bytes=[0u8;16];getrandom::fill(&mut bytes).map_err(|e|e.to_string())?;Ok(hex::encode(bytes))}
fn envelope(review:&Review,application:Option<&Value>)->Value{json!({"version":1,"proposal_protocol":1,"token":review.token,"fingerprint":review.fingerprint,"workspace":review.workspace,"namespace":review.namespace,"material":review.material,"summary":{"title":review.intention.title,"origin":review.intention.origin,"origin_authority":review.origin_authority},"source_effects":false,"checks_executed":false,"host_application":application})}
#[derive(Serialize)]
struct Material<'a>{version:u32,intention:&'a Intention,origin_authority:&'a str,workspace:&'a str,namespace:&'a str,inputs:&'a resource_inputs::Inputs,plan:&'a resource_changes::ResourcePlan,consequences:&'a Value,before:&'a Value,after:&'a Value,saved_guards:&'a BTreeMap<String,String>}
fn retain(review:Review)->Result<Arc<Review>,String>{
 let size=review.material.len();if size>32*1024*1024{return Err("Complete proposal review exceeds its retained budget.".into())}let cost=size.saturating_mul(2).saturating_add(2*1024*1024);
 let mut held=REVIEWS.lock().map_err(|_|"Proposal retention unavailable")?;
 while held.records.len()>=32||held.bytes.saturating_add(cost)>128*1024*1024{let id=held.order.pop_front().ok_or("Proposal retention budget unavailable")?;if let Some((_,cost))=held.records.remove(&id){held.bytes-=cost}held.applications.remove(&id);}
 let review=Arc::new(review);held.bytes+=cost;held.order.push_back(review.token.clone());held.records.insert(review.token.clone(),(review.clone(),cost));Ok(review)
}
fn guards(state:&LocalState,inputs:&resource_inputs::Inputs)->Result<BTreeMap<String,String>,String>{
 let mut values=BTreeMap::new();for(path,saved)in &inputs.saved_sources{if let resource_inputs::SavedSource::File{text,..}=saved{
  let(full,current)=resource_inputs::saved_target(state.index.root(),path)?;if &current!=saved{return Err("Proposal source changed while capturing authority.".into())}
  let read=super::source_history::read_for_api(state,&full,path)?;if read.source!=*text{return Err("Proposal source read changed.".into())}
  values.insert(path.clone(),read.save_guard.ok_or("Proposal source has no exact physical authority; retain the draft and choose its authored source.")?);
 }}
 for path in inputs.ownership.saved_sources.keys(){if !values.contains_key(path){let(full,_)=resource_inputs::saved_target(state.index.root(),path)?;let read=super::source_history::read_for_api(state,&full,path)?;values.insert(path.clone(),read.save_guard.ok_or("Construction source has no exact physical review authority.")?);}}
 Ok(values)
}
static ANALYSES:tokio::sync::Semaphore=tokio::sync::Semaphore::const_new(1);
async fn analyze(state:&LocalState,intention:&Intention,origin_authority:&str)->ApiResult<Review>{
 ordinary(state)?;
 let _permit=ANALYSES.try_acquire().map_err(|_|ApiError::conflict("A complete proposal analysis is already running. Keep the current review and retry after it finishes."))?;
 bounded(intention).map_err(ApiError::conflict)?;
 let mut inputs=resource_inputs::inspect(State(state.clone()),Json(intention.capture.clone())).await?.0;
 if !inputs.refusals.is_empty(){return Err(ApiError::conflict("A complete proposal source namespace is unavailable. No partial proposal was applied."))}
 let saved_guards=guards(state,&inputs).map_err(ApiError::conflict)?;
 for(path,source)in &mut inputs.sources{if let resource_changes::SourceState::File{identity,..}=source{*identity=hash(&(&inputs.workspace,path,saved_guards.get(path))).map_err(ApiError::internal)?;}}
 let plan=resource_changes::prepare(resource_changes::Request{context:inputs.context.clone(),title:intention.title.clone(),edit:intention.capture.edit.clone(),sources:inputs.sources.clone()}).map_err(ApiError::conflict)?;
 if plan.patches.is_empty(){return Err(ApiError::conflict("Proposal has no implementation changes to review."))}
 let facts=resource_candidates::inspect(State(state.clone()),Json(resource_candidates::Request{inputs:inputs.clone(),capture:intention.capture.clone(),plan:plan.clone()})).await?.0;
 if !facts.protected_targets.is_empty(){return Err(ApiError::conflict("Proposal targets generated output; revise its authored construction instead."))}
 let mut before_context=intention.reading.clone();for(path,text)in &intention.capture.ownership_sources{before_context.sources.insert(path.clone(),text.clone());}
 for(path,source)in &inputs.sources{match source{resource_changes::SourceState::File{text,..}=>{before_context.sources.insert(path.clone(),text.clone());before_context.missing_sources.remove(path);},resource_changes::SourceState::Missing{..}=>{before_context.sources.remove(path);before_context.missing_sources.insert(path.clone());}}}
 let mut after_context=before_context.clone();for patch in &plan.patches{match &patch.after{resource_changes::SourceState::File{text,..}=>{after_context.sources.insert(patch.path.clone(),text.clone());after_context.missing_sources.remove(&patch.path);},resource_changes::SourceState::Missing{..}=>{after_context.sources.remove(&patch.path);after_context.missing_sources.insert(patch.path.clone());}}}
 let before=behavior_readings::reading(State(state.clone()),Json(before_context)).await?.0;
 let after=behavior_readings::reading(State(state.clone()),Json(after_context)).await?.0;
 for id in &intention.check_ids{if !after["graph"]["checks"].as_array().is_some_and(|checks|checks.iter().any(|check|check["id"]==*id)){return Err(ApiError::conflict("An agent-requested check is absent from the current connected reading; no check was run."))}}
 if guards(state,&inputs).map_err(ApiError::conflict)?!=saved_guards{return Err(ApiError::conflict("Proposal source was replaced during analysis. Retain the existing editors and prepare again."))}
 let namespace=state.index.root().canonicalize().map_err(|e|ApiError::conflict(e.to_string()))?.to_string_lossy().into_owned();
 let consequences=serde_json::to_value(facts).map_err(|e|ApiError::internal(e.to_string()))?;
 ordinary(state)?;
 let material=serde_json::to_string(&Material{version:1,intention,origin_authority,workspace:&inputs.workspace,namespace:&namespace,inputs:&inputs,plan:&plan,consequences:&consequences,before:&before,after:&after,saved_guards:&saved_guards}).map_err(|error|ApiError::internal(error.to_string()))?;
 let fingerprint=super::workspace_fs::view::hash(material.as_bytes());
 Ok(Review{token:String::new(),fingerprint,material,origin_authority:origin_authority.into(),intention:intention.clone(),workspace:inputs.workspace.clone(),namespace,plan})
}
pub async fn prepare(State(state):State<LocalState>,Json(intention):Json<Intention>)->ApiResult<Json<Value>>{prepare_with_origin(state,intention,"caller-provided intention; not an agent attestation").await}
async fn prepare_with_origin(state:LocalState,intention:Intention,origin_authority:&str)->ApiResult<Json<Value>>{
 let mut review=analyze(&state,&intention,origin_authority).await?;review.token=nonce().map_err(ApiError::internal)?;
 let held=retain(review).map_err(ApiError::conflict)?;Ok(Json(envelope(&held,None)))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {pub version:u32,pub token:String,pub fingerprint:String}
pub async fn validate(State(state):State<LocalState>,Json(body):Json<Admission>)->ApiResult<Json<Value>>{
 ordinary(&state)?;
 if body.version!=1{return Err(ApiError::conflict("Unsupported proposal validation protocol."))}
 let held=REVIEWS.lock().map_err(|_|ApiError::conflict("Proposal retention unavailable"))?.records.get(&body.token).map(|(review,_)|review.clone()).ok_or_else(||ApiError::conflict("This retained proposal is unavailable. Prepare it again; nothing was applied."))?;
 if held.fingerprint!=body.fingerprint{return Err(ApiError::conflict("The complete proposal intention changed. Review it again."))}
 let fresh=analyze(&state,&held.intention,&held.origin_authority).await?;if fresh.fingerprint!=held.fingerprint{return Err(ApiError::conflict("Proposal source, ownership, generated consequences or evidence changed. No partial Apply was admitted."))}
 Ok(Json(json!({"version":1,"proposal_protocol":1,"token":body.token,"fingerprint":body.fingerprint,"workspace":fresh.workspace,"namespace":fresh.namespace,"admitted":true,"source_effects":false,"checks_executed":false,"authority":"current engine observation; host editor lease still required"})))
}
pub fn summary(review:&Value)->Value{json!({"version":1,"proposal_protocol":1,"token":review["token"],"fingerprint":review["fingerprint"],"title":review["summary"]["title"],"workspace":review["workspace"],"namespace":review["namespace"],"origin_authority":review["summary"]["origin_authority"],"pending_review":review["host_application"].is_null(),"host_application":review["host_application"],"applied":false,"saved":false,"checks_executed":false,"notice":"Retained proposal only. Open agent proposal review and explicitly Apply through the current host; do not infer implementation or verification from this tool result."})}
pub fn catalogue()->Vec<Value>{vec![json!({"name":"propose_workspace_changes","description":"Prepare one coherent agent proposal for explicit client review. Provide intention version1, request_id (32 lowercase hex), origin, title, explanation, claims, check_ids, capture (resource edit and construction context) and reading (connected reading context). This tool applies/saves/stages/runs nothing. Its pending review must be accepted by the host through guarded coordinated Apply; do not report implementation complete from this result.","inputSchema":{"type":"object","properties":{"intention":{"type":"object"}},"required":["intention"],"additionalProperties":false}})]}
pub async fn from_agent(state:LocalState,edits:&super::acp::edits::Edits,origin:String,args:&Value)->ApiResult<Json<Value>>{
 ordinary(&state)?;
 let mut intention: Intention=serde_json::from_value(args.get("intention").cloned().ok_or_else(||ApiError::conflict("Provide a complete proposal intention."))?).map_err(|e|ApiError::conflict(e.to_string()))?;
 intention.origin=origin;
 edits.refresh().await.map_err(|e|ApiError::conflict(e.to_string()))?;
 let buffers=edits.buffers.lock().map_err(|_|ApiError::conflict("Live agent editor capture unavailable"))?.clone();
 let targets: BTreeSet<_>=resource_changes::targets(&intention.capture.edit).map_err(ApiError::conflict)?.into_iter().collect();
 for(index,buffer)in buffers.into_iter().enumerate(){
  let unaddressed=buffer.path.as_deref().is_none_or(|path|std::path::Path::new(path).is_absolute());
  if unaddressed{intention.capture.unaddressed_sources.insert(buffer.id.unwrap_or_else(||format!("captured-editor-{index}")),buffer.content);continue}
  let path=buffer.path.expect("address checked above");resource_changes::validate_path(&path).map_err(ApiError::conflict)?;
  if intention.capture.ownership_sources.get(&path).is_some_and(|existing|existing!=&buffer.content){return Err(ApiError::conflict("Agent proposal and retained editor disagree about a source draft. Refresh the complete intention before review."))}
  intention.capture.ownership_sources.insert(path.clone(),buffer.content.clone());
  if let Some(guard)=&buffer.save_guard{if intention.capture.saved_guards.get(&path).is_some_and(|existing|existing!=guard){return Err(ApiError::conflict("Agent intention and live editor disagree about the original physical baseline. Refresh the complete proposal."))}intention.capture.saved_guards.insert(path.clone(),guard.clone());}else{intention.capture.saved_guards.remove(&path);}
  if targets.contains(&path)&&buffer.save_guard.is_none(){return Err(ApiError::conflict("This proposed editor target has no originally loaded physical guard. Keep its text and wait for or refresh its source authority; no later disk read was adopted."))}
  if targets.contains(&path){let saved=super::source_tests::saved_source(state.index.root(),&path).map_err(ApiError::conflict)?;intention.capture.drafts.insert(path,resource_inputs::Draft{text:buffer.content,base_sha256:super::workspace_fs::view::hash(saved.as_bytes()),version:None,writable:true,save_guard:buffer.save_guard.clone()});}
 }
 prepare_with_origin(state,intention,"engine-managed agent tool; claims remain unverified").await
}
pub async fn list(State(state):State<LocalState>)->ApiResult<Json<Value>>{
 ordinary(&state)?;
 let workspace=state.resource_root.check(state.index.root()).map_err(ApiError::conflict)?;
 let namespace=state.index.root().canonicalize().map_err(|e|ApiError::conflict(e.to_string()))?.to_string_lossy().into_owned();
 let held=REVIEWS.lock().map_err(|_|ApiError::conflict("Proposal retention unavailable"))?;
 let reviews:Vec<_>=held.order.iter().rev().filter_map(|token|held.records.get(token).map(|(review,_)|review)).filter(|review|review.workspace==workspace&&review.namespace==namespace).map(|review|json!({"token":review.token,"fingerprint":review.fingerprint,"title":review.intention.title,"origin":review.intention.origin,"origin_authority":review.origin_authority,"request_id":review.intention.request_id,"applied":false,"saved":false,"host_application":held.applications.get(&review.token)})).collect();
 Ok(Json(json!({"version":1,"proposal_protocol":1,"workspace":workspace,"namespace":namespace,"retention":"current engine process","reviews":reviews})))
}
pub async fn read(State(state):State<LocalState>,axum::extract::Path(token):axum::extract::Path<String>)->ApiResult<Json<Value>>{
 ordinary(&state)?;
 if !super::resource_saves::hex(&token,32){return Err(ApiError::conflict("Invalid retained proposal token"))}
 let workspace=state.resource_root.check(state.index.root()).map_err(ApiError::conflict)?;
 let namespace=state.index.root().canonicalize().map_err(|e|ApiError::conflict(e.to_string()))?.to_string_lossy().into_owned();
 let review=REVIEWS.lock().map_err(|_|ApiError::conflict("Proposal retention unavailable"))?.records.get(&token).map(|(review,_)|review.clone()).ok_or_else(||ApiError::conflict("Proposal is no longer retained. Prepare it again; nothing was applied."))?;
 if review.workspace!=workspace||review.namespace!=namespace{return Err(ApiError::conflict("Proposal belongs to another workspace. No same-named source was substituted."))}
 let application=REVIEWS.lock().map_err(|_|ApiError::conflict("Proposal retention unavailable"))?.applications.get(&token).cloned();Ok(Json(envelope(&review,application.as_ref())))
}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostReport {pub version:u32,pub token:String,pub fingerprint:String,pub workspace:String,pub namespace:String,pub request_id:String,pub host:String,pub after_hashes:BTreeMap<String,String>}
pub async fn report(State(state):State<LocalState>,Json(body):Json<HostReport>)->ApiResult<Json<Value>>{
 ordinary(&state)?;
 if body.version!=1||!super::resource_saves::hex(&body.request_id,32)||body.host.is_empty()||body.host.len()>128||body.after_hashes.len()>4096||serde_json::to_vec(&body).map_err(|e|ApiError::conflict(e.to_string()))?.len()>1024*1024{return Err(ApiError::conflict("Invalid bounded host draft report."))}
 let workspace=state.resource_root.check(state.index.root()).map_err(ApiError::conflict)?;let namespace=state.index.root().canonicalize().map_err(|e|ApiError::conflict(e.to_string()))?.to_string_lossy().into_owned();
 let mut held=REVIEWS.lock().map_err(|_|ApiError::conflict("Proposal retention unavailable"))?;let review=held.records.get(&body.token).map(|(review,_)|review.clone()).ok_or_else(||ApiError::conflict("Proposal is no longer retained. Keep the actual host drafts; do not reapply them to recover an acknowledgment."))?;
 if body.workspace!=workspace||body.namespace!=namespace||review.workspace!=workspace||review.namespace!=namespace||body.fingerprint!=review.fingerprint{return Err(ApiError::conflict("Host report belongs to another proposal or namespace."))}
 let expected:BTreeMap<_,_>=review.plan.patches.iter().map(|patch|(patch.path.clone(),match &patch.after{resource_changes::SourceState::File{text,..}=>super::workspace_fs::view::hash(text.as_bytes()),resource_changes::SourceState::Missing{..}=>"missing".into()})).collect();
 if expected!=body.after_hashes{return Err(ApiError::conflict("Host report does not describe the complete reviewed after namespace. Actual drafts were not changed by this acknowledgment."))}
 let intention=hash(&body).map_err(ApiError::internal)?;if let Some(previous)=held.applications.get(&body.token){if previous["report_fingerprint"]!=intention{return Err(ApiError::conflict("A different host application report is already retained. No source action was repeated."))}return Ok(Json(previous.clone()))}
 let receipt=json!({"version":1,"proposal_protocol":1,"token":body.token,"fingerprint":body.fingerprint,"workspace":workspace,"namespace":namespace,"request_id":body.request_id,"host":body.host,"after_hashes":body.after_hashes,"report_fingerprint":intention,"status":"host-reported draft application","evidence_authority":"host report; not independently observed by the engine","saved":false,"checks_executed":false});held.applications.insert(body.token,receipt.clone());Ok(Json(receipt))
}
#[cfg(test)]
mod tests {
 use super::*;
 pub(super) fn intention()->Intention{serde_json::from_value(json!({"version":1,"request_id":"0123456789abcdef0123456789abcdef","origin":"owned provider fixture","title":"Change note","explanation":"Keep the change and its reasoning together.","claims":["Expected behavior; not an observed verdict"],"check_ids":[],"capture":{"context":"context","edit":{"changes":{}},"drafts":{},"ownership_sources":{},"project_entry":null},"reading":{"version":1,"entry":"project.md","sources":{},"missing_sources":[],"focus":null}})).unwrap()}
 #[test] fn complete_intention_identity_includes_reasoning_implementation_and_checks(){let original=intention();bounded(&original).unwrap();let base=hash(&original).unwrap();let mut changed=original.clone();changed.explanation.push_str(" Revised.");assert_ne!(hash(&changed).unwrap(),base);changed=original.clone();changed.capture.edit=json!({"changes":{"hick:///a.py":[]}});assert_ne!(hash(&changed).unwrap(),base);changed=original.clone();changed.check_ids.push("declared-check".into());assert_ne!(hash(&changed).unwrap(),base);changed=original.clone();changed.claims[0]="Agent says passed; still only a claim".into();assert_ne!(hash(&changed).unwrap(),base);}
 #[test] fn malformed_or_amplified_claims_refuse_before_any_source_effect(){let original=intention();let mut value=original.clone();value.request_id="ABCDEF0123456789abcdef0123456789".into();assert!(bounded(&value).is_err());value=original.clone();value.claims=vec!["x".repeat(8193)];assert!(bounded(&value).is_err());value=original.clone();value.check_ids=vec!["same".into(),"same".into()];assert!(bounded(&value).is_err());let mut wire=serde_json::to_value(original).unwrap();wire["passed"]=json!(true);assert!(serde_json::from_value::<Intention>(wire).is_err());}
}
#[cfg(test)] pub(super) async fn assert_private_refused(state:&LocalState){
 let value=tests::intention();let token="0123456789abcdef0123456789abcdef".to_string();
 let actions=[prepare(State(state.clone()),Json(value.clone())).await,validate(State(state.clone()),Json(Admission{version:1,token:token.clone(),fingerprint:"unknown".into()})).await,list(State(state.clone())).await,read(State(state.clone()),axum::extract::Path(token.clone())).await,report(State(state.clone()),Json(HostReport{version:1,token,fingerprint:"unknown".into(),workspace:"unknown".into(),namespace:"unknown".into(),request_id:"0123456789abcdef0123456789abcdef".into(),host:"private refusal witness".into(),after_hashes:BTreeMap::new()})).await,from_agent(state.clone(),&super::acp::edits::Edits::default(),"private".into(),&Value::Null).await];
 for action in actions{match action{Ok(_)=>panic!("Ordinary proposal entry admitted private prediction state"),Err(error)=>assert!(error.message().contains("Private prediction"),"{}",error.message())}}
}
