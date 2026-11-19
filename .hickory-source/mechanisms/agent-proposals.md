<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Review one agent intention before changing source

For maintainers connecting an agent tool to the existing guarded change adapters.
The conversation's explanation is a claim, not verification evidence. Freeze that
claim, every implementation patch, saved/draft construction consequences and the
engine's connected readings into one review. Preparing or validating a review
does not apply, save, stage or run anything. Hosts must still hold their complete
editor lease and use the existing coordinated change mechanism after validation.
Sealed historical readings survive an engine restart under the original physical root, while admission still requires a current in-process Review. Host-reported application remains an unverified host claim, never an engine-observed source effect. Browser managed-tool review/application and Android caller-provided and managed-tool controller flows have focused actual acceptance. Apple host acceptance is in progress. Scripted providers and unexecuted physical input do not establish broader provider or accessibility acceptance.

## Keep agent statements separate from engine observations

Bound the complete intention before analysis. A requested check must identify an
engine-discovered relationship; arbitrary agent text cannot manufacture a passing
result or executable command. The returned readings retain their own unknown and
historical evidence labels.

Managed proposals also seal the original editor identities and monotonic source
revisions received from the client. Revision observations are canonical decimal
strings; no renderer may round them or fill in a missing revision from a later
capture. The tool overwrites any agent-supplied binding. Hosts can therefore
refuse an edit/undo or independent handle replacement that happened before the
review opened, even when file bytes and physical guards are unchanged.

<ui:copy id="agent-proposal-model">
//! One immutable agent intention; source effects remain with existing publishers.
use super::{LocalState, api::{ApiError,ApiResult}, resource_inputs, resource_changes, resource_candidates, behavior_readings};
use axum::{Json,extract::State};
use serde::{Deserialize,Serialize};
use serde_json::{Value,json};
use std::{collections::{BTreeMap,BTreeSet,VecDeque},sync::{Arc,Mutex,LazyLock}};
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundEditor {pub id:Option<String>,pub path:Option<String>,pub source_hash:String,pub save_guard:Option<String>,pub source_revision:Option<String>}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditorSnapshotBinding {pub live:bool,pub buffers:Vec<BoundEditor>}
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
 #[serde(default,skip_serializing_if="Option::is_none")]pub editor_snapshot:Option<EditorSnapshotBinding>,
}
fn ordinary(state:&LocalState)->ApiResult<()>{if state.private_prediction.is_some()||crate::engine::private_policy::active().map_err(|error|ApiError::conflict(error.to_string()))?{return Err(ApiError::conflict("Private prediction requires its isolated proposal adapter; ordinary proposal readings, tools and application admission are unavailable."))}Ok(())}
fn hash(value:&impl Serialize)->Result<String,String>{serde_json::to_vec(value).map(|bytes|super::workspace_fs::view::hash(&bytes)).map_err(|error|error.to_string())}
fn bounded(value:&Intention)->Result<(),String>{
 if value.version!=1||!super::resource_saves::hex(&value.request_id,32)||value.request_id.bytes().any(|b|b.is_ascii_uppercase()){return Err("Proposal requires version1 and a lowercase intention identity.".into())}
 if value.title.trim().is_empty()||value.title.len()>512||value.origin.is_empty()||value.origin.len()>1024||value.explanation.trim().is_empty()||value.explanation.len()>65536||value.claims.len()>128||value.claims.iter().any(|claim|claim.len()>8192)||value.check_ids.len()>256{return Err("Proposal explanation or claim set is missing or exceeds its review budget.".into())}
 if let Some(snapshot)=&value.editor_snapshot{let mut ids=BTreeSet::new();if snapshot.buffers.iter().filter_map(|row|row.id.as_ref()).any(|id|!ids.insert(id)){return Err("Original editor snapshot repeats a handle identity.".into())}if snapshot.buffers.len()>4096||snapshot.buffers.iter().any(|row|row.id.as_ref().is_some_and(|id|id.is_empty()||id.len()>1024)||row.path.as_ref().is_some_and(|path|path.len()>16384)||!super::resource_saves::hex(&row.source_hash,64)||row.save_guard.as_ref().is_some_and(|guard|guard.len()>16384)||row.source_revision.as_ref().is_some_and(|revision|!revision_text(revision))){return Err("Original editor snapshot binding is malformed or exceeds its budget.".into())}}
 if serde_json::to_vec(value).map_err(|e|e.to_string())?.len()>32*1024*1024{return Err("Proposal exceeds its bounded complete-intention budget.".into())}
 if value.check_ids.iter().collect::<BTreeSet<_>>().len()!=value.check_ids.len(){return Err("Proposal repeats a check identity.".into())}
 Ok(())
}
#[derive(Clone,Serialize)]
struct Review {
 token:String,fingerprint:String,material:String,origin_authority:String,
 intention:Intention,workspace:String,namespace:String,plan:resource_changes::ResourcePlan,
 #[serde(skip)] recorded_diff: Option<String>,
}
#[derive(Default)]
struct Retained {order:VecDeque<String>,records:BTreeMap<String,(Arc<Review>,usize)>,applications:BTreeMap<String,Value>,durability:BTreeMap<String,Value>,bytes:usize}
static REVIEWS:LazyLock<Mutex<Retained>>=LazyLock::new(||Mutex::new(Retained::default()));
fn nonce()->Result<String,String>{let mut bytes=[0u8;16];getrandom::fill(&mut bytes).map_err(|e|e.to_string())?;Ok(hex::encode(bytes))}
fn envelope(review:&Review,application:Option<&Value>,durability:Option<&Value>)->Value{json!({"version":1,"proposal_protocol":1,"token":review.token,"fingerprint":review.fingerprint,"workspace":review.workspace,"namespace":review.namespace,"material":review.material,"summary":{"title":review.intention.title,"origin":review.intention.origin,"origin_authority":review.origin_authority},"source_effects":false,"checks_executed":false,"host_application":application,"historical":false,"durability":durability})}
#[derive(Serialize)]
struct Material<'a>{version:u32,intention:&'a Intention,origin_authority:&'a str,workspace:&'a str,namespace:&'a str,inputs:&'a resource_inputs::Inputs,plan:&'a resource_changes::ResourcePlan,consequences:&'a Value,before:&'a Value,after:&'a Value,saved_guards:&'a BTreeMap<String,String>}
fn retain(review:Review)->Result<Arc<Review>,String>{
 let size=review.material.len();if size>32*1024*1024{return Err("Complete proposal review exceeds its retained budget.".into())}let cost=size.saturating_mul(2).saturating_add(2*1024*1024);
 let mut held=REVIEWS.lock().map_err(|_|"Proposal retention unavailable")?;
 while held.records.len()>=32||held.bytes.saturating_add(cost)>128*1024*1024{let id=held.order.pop_front().ok_or("Proposal retention budget unavailable")?;if let Some((_,cost))=held.records.remove(&id){held.bytes-=cost}held.applications.remove(&id);held.durability.remove(&id);}
 let review=Arc::new(review);held.bytes+=cost;held.order.push_back(review.token.clone());held.records.insert(review.token.clone(),(review.clone(),cost));Ok(review)
}
</ui:copy>

## Reuse namespace and construction authority

Acquire the complete source namespace with the existing resource-input provider,
then prepare its ordered plan and candidate consequences. A writable source's
physical read guard is captured separately from equal-text hashes. Derive review-local
logical identities from those guards so a fresh observation does not invent new
identities and invalidate an otherwise unchanged review. Recheck it
after asynchronous readings and before admitting a later Apply. Missing targets
remain missing states governed by the existing plan rather than invented files.

Derived before/after readings carry the capture's original named save guards and
anonymous drafts as well as text. A conflicting reading guard or anonymous body
refuses the coherent intention; analysis must not silently replace it with newly
read proof. These derived maps leave the sealed caller intention unchanged.

<ui:copy id="agent-proposal-analysis">
fn guards(state:&LocalState,inputs:&resource_inputs::Inputs)->Result<BTreeMap<String,String>,String>{
 let mut values=BTreeMap::new();for(path,saved)in &inputs.saved_sources{if let resource_inputs::SavedSource::File{text,..}=saved{
  let(full,current)=resource_inputs::saved_target(state.index.root(),path)?;if &current!=saved{return Err("Proposal source changed while capturing authority.".into())}
  let read=super::source_history::read_for_api(state,&full,path)?;if read.source!=*text{return Err("Proposal source read changed.".into())}
  values.insert(path.clone(),read.save_guard.ok_or("Proposal source has no exact physical authority; retain the draft and choose its authored source.")?);
 }}
 for path in inputs.ownership.saved_sources.keys(){if !values.contains_key(path){let(full,_)=resource_inputs::saved_target(state.index.root(),path)?;let read=super::source_history::read_for_api(state,&full,path)?;values.insert(path.clone(),read.save_guard.ok_or("Construction source has no exact physical review authority.")?);}}
 Ok(values)
}
// Carry original host observations into derived readings; never replace a
// conflicting proof with a newer one acquired while analyzing the proposal.
fn carry_context(reading:&mut behavior_readings::ReadingRequest,capture:&resource_inputs::Request)->ApiResult<()>{
 for(path,guard)in capture.saved_guards.iter().chain(capture.drafts.iter().filter_map(|(path,draft)|draft.save_guard.as_ref().map(|guard|(path,guard)))){
  if reading.saved_guards.get(path).is_some_and(|original|original!=guard){return Err(ApiError::conflict("Proposal source and connected-reading guards differ. Retain the current review and prepare one coherent intention."))}
  reading.saved_guards.insert(path.clone(),guard.clone());
 }
 for(id,text)in &capture.unaddressed_sources{
  if reading.unaddressed_sources.get(id).is_some_and(|original|original!=text){return Err(ApiError::conflict("Proposal anonymous source and connected reading differ. Prepare one coherent intention."))}
  reading.unaddressed_sources.insert(id.clone(),text.clone());
 }
 Ok(())
}
static ANALYSES:tokio::sync::Semaphore=tokio::sync::Semaphore::const_new(1);
fn recorded_patch_input(
    sources: &BTreeMap<String, resource_changes::SourceState>,
) -> Result<hick_lang::ProjectTree, String> {
    use resource_changes::SourceState;
    let mut starting = hick_lang::ProjectTree::default();
    for (path, source) in sources {
        if let SourceState::File { text, .. } = source {
            starting.insert(path, hick_lang::ProjectEntry::file(text.as_bytes()))?;
        }
    }
    Ok(starting)
}
// A fence can describe an edit, but it cannot grant authority over any path.
// Compare the exact replay with the complete guarded transition before sealing.
fn verify_recorded_patch(
    diff: &str,
    plan: &resource_changes::ResourcePlan,
    sources: &BTreeMap<String, resource_changes::SourceState>,
) -> Result<(), String> {
    use resource_changes::SourceState;
    let starting = recorded_patch_input(sources)?;
    let replay = hick_lang::apply_project_patch(&starting, diff, "recorded-message", 0)?;
    let targets: BTreeSet<_> = plan
        .patches
        .iter()
        .map(|patch| patch.path.clone())
        .collect();
    if replay
        .before
        .compare(&replay.after)
        .into_iter()
        .map(|difference| difference.path)
        .collect::<BTreeSet<_>>()
        != targets
    {
        return Err("The proposed source set differs from the recorded patch. Review all its targets together.".into());
    }
    for patch in &plan.patches {
        if sources.get(&patch.path) != Some(&patch.before) {
            return Err(format!(
                "The draft preimage changed at {:?}. Prepare the whole patch again.",
                patch.path
            ));
        }
        let expected = replay
            .after
            .get(&patch.path)
            .map(|entry| entry.bytes.as_ref());
        let proposed = match &patch.after {
            SourceState::File { text, .. } => Some(text.as_bytes()),
            SourceState::Missing { .. } => None,
        };
        if expected != proposed {
            return Err(format!(
                "The proposed source differs from the recorded patch at {:?}. Refresh the patch and try again.",
                patch.path
            ));
        }
    }
    Ok(())
}
fn recorded_path_uri(path: &str) -> String {
    let mut uri = reqwest::Url::parse("hick:///").expect("static workspace URI");
    uri.path_segments_mut()
        .expect("workspace URI has path segments")
        .pop_if_empty()
        .extend(path.split('/'));
    uri.into()
}
fn recorded_patch_edit(
    diff: &str,
    targets: &[hick_lang::PatchTarget],
    sources: &BTreeMap<String, resource_changes::SourceState>,
) -> Result<Value, String> {
    use resource_changes::SourceState;
    let starting = recorded_patch_input(sources)?;
    let replay = hick_lang::apply_project_patch(&starting, diff, "recorded-message", 0)?;
    let mut operations = Vec::new();
    for target in targets {
        let before = target.before.as_deref();
        let after = target.after.as_deref();
        match (before, after) {
            (Some(from), Some(to)) if from != to => operations.push(json!({"kind":"rename","oldUri":recorded_path_uri(from),"newUri":recorded_path_uri(to)})),
            (None, Some(to)) => operations.push(json!({"kind":"create","uri":recorded_path_uri(to)})),
            (Some(from), None) => operations.push(json!({"kind":"delete","uri":recorded_path_uri(from)})),
            _ => (),
        }
        if let Some(path) = after {
            let previous = match before.and_then(|name| sources.get(name)) {
                Some(SourceState::File { text, .. }) => text.as_str(),
                _ => "",
            };
            let entry = replay
                .after
                .get(path)
                .ok_or("No recorded output for the proposed source")?;
            let next = std::str::from_utf8(&entry.bytes).map_err(|error| error.to_string())?;
            if previous != next {
                let (line, tail) =
                    previous
                        .rsplit_once('\n')
                        .map_or((0, previous), |(prefix, tail)| {
                            (
                                prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
                                tail,
                            )
                        });
                operations.push(json!({"textDocument":{"uri":recorded_path_uri(path),"version":null},"edits":[{
                    "range":{"start":{"line":0,"character":0},"end":{"line":line,"character":tail.encode_utf16().count()}},"newText":next
                }]}));
            }
        }
    }
    Ok(json!({"documentChanges":operations}))
}

async fn analyze(state:&LocalState,intention:&Intention,origin_authority:&str,diff:Option<&str>)->ApiResult<Review>{
 ordinary(state)?;
 let _permit=ANALYSES.try_acquire().map_err(|_|ApiError::conflict("A complete proposal analysis is already running. Keep the current review and retry after it finishes."))?;
 bounded(intention).map_err(ApiError::conflict)?;
 let mut intention = intention.clone();
 let targets = diff.map(hick_lang::project_patch_targets).transpose().map_err(ApiError::conflict)?;
 if let Some(targets) = &targets {
    let paths: BTreeSet<_> = targets.iter().flat_map(|target| [&target.before, &target.after])
        .filter_map(Option::as_deref).collect();
    intention.capture.drafts.retain(|path, _| paths.contains(path.as_str()));
    let uris: serde_json::Map<String, Value> = paths.into_iter().map(|path| (recorded_path_uri(path), json!([]))).collect();
    intention.capture.edit = json!({"changes":uris});
 }
 let mut inputs=resource_inputs::inspect(State(state.clone()),Json(intention.capture.clone())).await?.0;
 if !inputs.refusals.is_empty(){return Err(ApiError::conflict("A complete proposal source namespace is unavailable. No partial proposal was applied."))}
 if let (Some(diff), Some(targets)) = (diff, &targets) {
    intention.capture.edit = recorded_patch_edit(diff, targets, &inputs.sources).map_err(ApiError::conflict)?;
    bounded(&intention).map_err(ApiError::conflict)?;
 }
 let saved_guards=guards(state,&inputs).map_err(ApiError::conflict)?;
 for(path,source)in &mut inputs.sources{if let resource_changes::SourceState::File{identity,..}=source{*identity=hash(&(&inputs.workspace,path,saved_guards.get(path))).map_err(ApiError::internal)?;}}
 let plan=resource_changes::prepare(resource_changes::Request{context:inputs.context.clone(),title:intention.title.clone(),edit:intention.capture.edit.clone(),sources:inputs.sources.clone()}).map_err(ApiError::conflict)?;
 if plan.patches.is_empty(){return Err(ApiError::conflict("Proposal has no implementation changes to review."))}
 if let Some(diff) = diff { verify_recorded_patch(diff, &plan, &inputs.sources).map_err(ApiError::conflict)?; }
 let facts=resource_candidates::inspect(State(state.clone()),Json(resource_candidates::Request{inputs:inputs.clone(),capture:intention.capture.clone(),plan:plan.clone()})).await?.0;
 if !facts.protected_targets.is_empty(){return Err(ApiError::conflict("Proposal targets generated output; revise its authored construction instead."))}
 let mut before_context=intention.reading.clone();carry_context(&mut before_context,&intention.capture)?;for(path,text)in &intention.capture.ownership_sources{before_context.sources.insert(path.clone(),text.clone());}
 for(path,source)in &inputs.sources{match source{resource_changes::SourceState::File{text,..}=>{before_context.sources.insert(path.clone(),text.clone());before_context.missing_sources.remove(path);},resource_changes::SourceState::Missing{..}=>{before_context.sources.remove(path);before_context.missing_sources.insert(path.clone());}}}
 let mut after_context=before_context.clone();for patch in &plan.patches{match &patch.after{resource_changes::SourceState::File{text,..}=>{after_context.sources.insert(patch.path.clone(),text.clone());after_context.missing_sources.remove(&patch.path);},resource_changes::SourceState::Missing{..}=>{after_context.sources.remove(&patch.path);after_context.missing_sources.insert(patch.path.clone());}}}
 let before=behavior_readings::reading(State(state.clone()),Json(before_context)).await?.0;
 let after=behavior_readings::reading(State(state.clone()),Json(after_context)).await?.0;
 for id in &intention.check_ids{if !after["graph"]["checks"].as_array().is_some_and(|checks|checks.iter().any(|check|check["id"]==*id)){return Err(ApiError::conflict("An agent-requested check is absent from the current connected reading; no check was run."))}}
 if guards(state,&inputs).map_err(ApiError::conflict)?!=saved_guards{return Err(ApiError::conflict("Proposal source was replaced during analysis. Retain the existing editors and prepare again."))}
 let namespace=state.index.root().canonicalize().map_err(|e|ApiError::conflict(e.to_string()))?.to_string_lossy().into_owned();
 let consequences=serde_json::to_value(facts).map_err(|e|ApiError::internal(e.to_string()))?;
 ordinary(state)?;
 let material=serde_json::to_string(&Material{version:1,intention:&intention,origin_authority,workspace:&inputs.workspace,namespace:&namespace,inputs:&inputs,plan:&plan,consequences:&consequences,before:&before,after:&after,saved_guards:&saved_guards}).map_err(|error|ApiError::internal(error.to_string()))?;
 let fingerprint=super::workspace_fs::view::hash(material.as_bytes());
 Ok(Review{token:String::new(),fingerprint,material,origin_authority:origin_authority.into(),intention,workspace:inputs.workspace.clone(),namespace,plan,recorded_diff:diff.map(str::to_owned)})
}
</ui:copy>

## Validate the complete retained intention before host application

A token does not authorize publication. Validation rebuilds the same intention,
including construction metadata and evidence, and compares the complete receipt.
The envelope carries one exact JSON material string with its UTF-8 SHA256. Hosts
verify those bytes before displaying any consequence or patch; they do not have
to canonicalize JSON or round-trip opaque provider numbers. Validation names only
the immutable token and material hash, never a reconstructed mutable intention.
Caller-provided origins remain separate from engine-managed agent-tool provenance.
Hosts must synchronously recheck their own editor lease after this reply, then
apply the complete plan through their existing coordinated undo path. A lost or
evicted review fails closed; refreshing a review never silently applies it.

<ui:copy id="agent-proposal-routes">
pub async fn prepare(State(state):State<LocalState>,Json(intention):Json<Intention>)->ApiResult<Json<Value>>{prepare_with_origin(state,intention,"caller-provided intention; not an agent attestation",None).await}
pub(super) async fn prepare_recorded_patch(state:LocalState,intention:Intention,origin_authority:&str,diff:&str)->ApiResult<Json<Value>>{
 prepare_with_origin(state,intention,origin_authority,Some(diff)).await
}
async fn prepare_with_origin(state:LocalState,intention:Intention,origin_authority:&str,diff:Option<&str>)->ApiResult<Json<Value>>{
 let mut review=analyze(&state,&intention,origin_authority,diff).await?;review.token=nonce().map_err(ApiError::internal)?;
 let held=retain(review).map_err(ApiError::conflict)?;
 let durability=archive_review(&state,held.clone(),None).await;
 {let mut retention=REVIEWS.lock().map_err(|_|ApiError::conflict("Proposal retention unavailable"))?;if retention.records.contains_key(&held.token){retention.durability.insert(held.token.clone(),durability.clone());}}
 Ok(Json(envelope(&held,None,Some(&durability))))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {pub version:u32,pub token:String,pub fingerprint:String}
pub async fn validate(State(state):State<LocalState>,Json(body):Json<Admission>)->ApiResult<Json<Value>>{
 ordinary(&state)?;
 if body.version!=1{return Err(ApiError::conflict("Unsupported proposal validation protocol."))}
 let held=REVIEWS.lock().map_err(|_|ApiError::conflict("Proposal retention unavailable"))?.records.get(&body.token).map(|(review,_)|review.clone()).ok_or_else(||ApiError::conflict("This retained proposal is unavailable. Prepare it again; nothing was applied."))?;
 let workspace=state.resource_root.check(state.index.root()).map_err(ApiError::conflict)?;
 let namespace=state.index.root().canonicalize().map_err(|e|ApiError::conflict(e.to_string()))?.to_string_lossy().into_owned();
 if held.workspace!=workspace||held.namespace!=namespace{return Err(ApiError::conflict("Historical proposal readings cannot admit application. Prepare a new live intention in the current workspace."))}
 if held.fingerprint!=body.fingerprint{return Err(ApiError::conflict("The complete proposal intention changed. Review it again."))}
 let fresh=analyze(&state,&held.intention,&held.origin_authority,held.recorded_diff.as_deref()).await?;if fresh.fingerprint!=held.fingerprint{return Err(ApiError::conflict("Proposal source, ownership, generated consequences or evidence changed. No partial Apply was admitted."))}
 Ok(Json(json!({"version":1,"proposal_protocol":1,"token":body.token,"fingerprint":body.fingerprint,"workspace":fresh.workspace,"namespace":fresh.namespace,"admitted":true,"source_effects":false,"checks_executed":false,"authority":"current engine observation; host editor lease still required"})))
}
</ui:copy>

## Let an agent offer a review without applying its own proposal

The existing private tool bridge supplies the live editor capture and records the
review token. The tool returns a pending intention, never a claim of application.
It refreshes retained buffers before preparing; unnamed and outside buffers stay
in the typed unaddressed source set and never receive filesystem identities. Their
ordinary text remains allowed; construction declarations require an authored
namespace before this workspace proposal can proceed. Hosts list small summaries first, then read
one bounded review in the existing floating agent surface. Retention is currently
process-local; an engine restart or eviction reports unavailable and requires a
new review rather than reconstructing authority from an old token.

A non-live send-time snapshot cannot answer a later UI capture request. Keep it
explicitly non-live and retain its original guards; request refresh only from a
client that advertises the live capture bridge. Both paths remain subject to the
complete current host roster and source guard checks before application.

<ui:copy id="agent-proposal-tool-discovery">
pub fn summary(review:&Value)->Value{json!({"version":1,"proposal_protocol":1,"token":review["token"],"fingerprint":review["fingerprint"],"title":review["summary"]["title"],"workspace":review["workspace"],"namespace":review["namespace"],"origin_authority":review["summary"]["origin_authority"],"pending_review":review["host_application"].is_null(),"host_application":review["host_application"],"applied":false,"saved":false,"checks_executed":false,"notice":"Retained proposal only. Open agent proposal review and explicitly Apply through the current host; do not infer implementation or verification from this tool result."})}
pub fn catalogue()->Vec<Value>{vec![json!({"name":"propose_workspace_changes","description":"Prepare one coherent agent proposal for explicit client review. Provide intention version1, request_id (32 lowercase hex), origin, title, explanation, claims, check_ids, capture (resource edit and construction context) and reading (connected reading context). This tool applies/saves/stages/runs nothing. Its pending review must be accepted by the host through guarded coordinated Apply; do not report implementation complete from this result.","inputSchema":{"type":"object","properties":{"intention":{"type":"object"}},"required":["intention"],"additionalProperties":false}})]}
fn revision_text(value:&str)->bool{!value.is_empty()&&value.len()<=20&&value.bytes().all(|byte|byte.is_ascii_digit())&&(value=="0"||!value.starts_with('0'))&&value.parse::<u64>().is_ok()}
fn bind_editors(buffers:&[super::agent_context::EditorBuffer],live:bool)->ApiResult<EditorSnapshotBinding>{
 let mut bound=Vec::new();
 for buffer in buffers{
  let mut revision:Option<String>=None;
  for observation in &buffer.observations{if observation["kind"]=="editor_revision"{
   let value=observation["source_revision"].as_str().filter(|value|revision_text(value)).ok_or_else(||ApiError::conflict("Original editor revision observation is invalid; recapture the complete intention."))?;
   if observation.get("version").and_then(Value::as_u64)!=Some(1)||revision.as_deref().is_some_and(|old|old!=value){return Err(ApiError::conflict("Original editor revision observations disagree or are unsupported."))}revision=Some(value.to_owned());
  }}
  let path=buffer.path.clone().filter(|path|!std::path::Path::new(path).is_absolute());
  bound.push(BoundEditor{id:buffer.id.clone(),save_guard:if path.is_some(){buffer.save_guard.clone()}else{None},path,source_hash:super::workspace_fs::view::hash(buffer.content.as_bytes()),source_revision:revision});
 }
 Ok(EditorSnapshotBinding{live,buffers:bound})
}
async fn captured_agent_buffers(edits:&super::acp::edits::Edits)->ApiResult<(Vec<super::agent_context::EditorBuffer>,bool)>{
 let live=edits.live_context.load(std::sync::atomic::Ordering::Acquire);
 if live{edits.refresh().await.map_err(|e|ApiError::conflict(e.to_string()))?;if !edits.live_context.load(std::sync::atomic::Ordering::Acquire){return Err(ApiError::conflict("Live editor capture was withdrawn; retain the proposal and prepare again."))}}
 let buffers=edits.buffers.lock().map_err(|_|ApiError::conflict("Retained agent editor snapshot is unavailable"))?.clone();
 Ok((buffers,live))
}
pub async fn from_agent(state:LocalState,edits:&super::acp::edits::Edits,origin:String,args:&Value)->ApiResult<Json<Value>>{
 ordinary(&state)?;
 let mut intention: Intention=serde_json::from_value(args.get("intention").cloned().ok_or_else(||ApiError::conflict("Provide a complete proposal intention."))?).map_err(|e|ApiError::conflict(e.to_string()))?;
 intention.origin=origin;
 let (buffers,live)=captured_agent_buffers(edits).await?;
 let worker=state.clone();
 let intention=tokio::task::spawn_blocking(move||->ApiResult<Intention>{
 ordinary(&worker)?;
 intention.editor_snapshot=Some(bind_editors(&buffers,live)?);
 let targets: BTreeSet<_>=resource_changes::targets(&intention.capture.edit).map_err(ApiError::conflict)?.into_iter().collect();
 for(index,buffer)in buffers.into_iter().enumerate(){
  let unaddressed=buffer.path.as_deref().is_none_or(|path|std::path::Path::new(path).is_absolute());
  if unaddressed{intention.capture.unaddressed_sources.insert(buffer.id.unwrap_or_else(||format!("captured-editor-{index}")),buffer.content);continue}
  let path=buffer.path.expect("address checked above");resource_changes::validate_path(&path).map_err(ApiError::conflict)?;
  if intention.capture.ownership_sources.get(&path).is_some_and(|existing|existing!=&buffer.content){return Err(ApiError::conflict("Agent proposal and retained editor disagree about a source draft. Refresh the complete intention before review."))}
  intention.capture.ownership_sources.insert(path.clone(),buffer.content.clone());
  if let Some(guard)=&buffer.save_guard{if intention.capture.saved_guards.get(&path).is_some_and(|existing|existing!=guard){return Err(ApiError::conflict("Agent intention and live editor disagree about the original physical baseline. Refresh the complete proposal."))}intention.capture.saved_guards.insert(path.clone(),guard.clone());}else{intention.capture.saved_guards.remove(&path);}
  if targets.contains(&path)&&buffer.save_guard.is_none(){return Err(ApiError::conflict("This proposed editor target has no originally loaded physical guard. Keep its text and wait for or refresh its source authority; no later disk read was adopted."))}
  if targets.contains(&path){let saved=super::source_tests::saved_source(worker.index.root(),&path).map_err(ApiError::conflict)?;intention.capture.drafts.insert(path,resource_inputs::Draft{text:buffer.content,base_sha256:super::workspace_fs::view::hash(saved.as_bytes()),version:None,writable:true,save_guard:buffer.save_guard.clone()});}
 }
 ordinary(&worker)?;
 Ok(intention)
 }).await.map_err(|error|ApiError::internal(error.to_string()))??;
 prepare_with_origin(state,intention,if live{"engine-managed agent tool; live editor capture at preparation; claims remain unverified"}else{"engine-managed agent tool; non-live submitted editor snapshot; claims remain unverified"},None).await
}
pub async fn list(State(state):State<LocalState>)->ApiResult<Json<Value>>{
 ordinary(&state)?;
 let workspace=state.resource_root.check(state.index.root()).map_err(ApiError::conflict)?;
 let namespace=state.index.root().canonicalize().map_err(|e|ApiError::conflict(e.to_string()))?.to_string_lossy().into_owned();
 let archived=archive_readings(&state,&workspace,&namespace).await;
 let held=REVIEWS.lock().map_err(|_|ApiError::conflict("Proposal retention unavailable"))?;
 let reviews:Vec<_>=held.order.iter().rev().filter_map(|token|held.records.get(token).map(|(review,_)|review)).filter(|review|review.workspace==workspace&&review.namespace==namespace).map(|review|json!({"token":review.token,"fingerprint":review.fingerprint,"title":review.intention.title,"origin":review.intention.origin,"origin_authority":review.origin_authority,"request_id":review.intention.request_id,"historical":false,"applied":false,"saved":false,"host_application":held.applications.get(&review.token),"durability":held.durability.get(&review.token)})).collect();
 let(reviews,notice)=merge_proposal_catalogue(reviews,archived);
 Ok(Json(json!({"version":1,"proposal_protocol":1,"workspace":workspace,"namespace":namespace,"retention":"current live reviews and sealed historical readings","retention_notice":notice,"reviews":reviews})))
}
pub async fn read(State(state):State<LocalState>,axum::extract::Path(token):axum::extract::Path<String>)->ApiResult<Json<Value>>{
 ordinary(&state)?;
 if !super::resource_saves::hex(&token,32){return Err(ApiError::conflict("Invalid retained proposal token"))}
 let workspace=state.resource_root.check(state.index.root()).map_err(ApiError::conflict)?;
 let namespace=state.index.root().canonicalize().map_err(|e|ApiError::conflict(e.to_string()))?.to_string_lossy().into_owned();
 let live={let held=REVIEWS.lock().map_err(|_|ApiError::conflict("Proposal retention unavailable"))?;held.records.get(&token).filter(|(review,_)|review.workspace==workspace&&review.namespace==namespace).map(|(review,_)|(review.clone(),held.applications.get(&token).cloned(),held.durability.get(&token).cloned()))};
 if let Some((review,application,durability))=live{return Ok(Json(envelope(&review,application.as_ref(),durability.as_ref())))}
 let readings=archive_readings(&state,&workspace,&namespace).await.map_err(ApiError::conflict)?;
 let row=readings.into_iter().find(|row|row["token"]==token).ok_or_else(||ApiError::conflict("Proposal reading is no longer retained. Preserve existing drafts; no archived review was promoted to live authority."))?;
 state.resource_root.check(state.index.root()).map_err(ApiError::conflict)?;Ok(Json(row))
}
</ui:copy>

## Retain a host report without converting it into engine evidence

After the host atomically applies the complete source plan, it may retain an exact
request-addressed report. The report is explicitly host-reported draft application,
not saved source, generated output or an executed check. A lost acknowledgment can
be retried with the same body without reapplying the source plan. The historical
report never authorizes another Apply, including after an independent undo.

<ui:copy id="agent-proposal-host-report">
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostReport {pub version:u32,pub token:String,pub fingerprint:String,pub workspace:String,pub namespace:String,pub request_id:String,pub host:String,pub after_hashes:BTreeMap<String,String>}
pub async fn report(State(state):State<LocalState>,Json(body):Json<HostReport>)->ApiResult<Json<Value>>{
 ordinary(&state)?;
 if body.version!=1||!super::resource_saves::hex(&body.request_id,32)||body.host.is_empty()||body.host.len()>128||body.after_hashes.len()>4096||serde_json::to_vec(&body).map_err(|e|ApiError::conflict(e.to_string()))?.len()>1024*1024{return Err(ApiError::conflict("Invalid bounded host draft report."))}
 let workspace=state.resource_root.check(state.index.root()).map_err(ApiError::conflict)?;let namespace=state.index.root().canonicalize().map_err(|e|ApiError::conflict(e.to_string()))?.to_string_lossy().into_owned();
 let (review,receipt)={
 let mut held=REVIEWS.lock().map_err(|_|ApiError::conflict("Proposal retention unavailable"))?;let review=held.records.get(&body.token).map(|(review,_)|review.clone()).ok_or_else(||ApiError::conflict("Proposal is no longer retained. Keep the actual host drafts; do not reapply them to recover an acknowledgment."))?;
 if body.workspace!=workspace||body.namespace!=namespace||review.workspace!=workspace||review.namespace!=namespace||body.fingerprint!=review.fingerprint{return Err(ApiError::conflict("Host report belongs to another proposal or namespace."))}
 let expected:BTreeMap<_,_>=review.plan.patches.iter().map(|patch|(patch.path.clone(),match &patch.after{resource_changes::SourceState::File{text,..}=>super::workspace_fs::view::hash(text.as_bytes()),resource_changes::SourceState::Missing{..}=>"missing".into()})).collect();
 if expected!=body.after_hashes{return Err(ApiError::conflict("Host report does not describe the complete reviewed after namespace. Actual drafts were not changed by this acknowledgment."))}
 let intention=hash(&body).map_err(ApiError::internal)?;if let Some(previous)=held.applications.get(&body.token){if previous["report_fingerprint"]!=intention{return Err(ApiError::conflict("A different host application report is already retained. No source action was repeated."))}return Ok(Json(previous.clone()))}
 let receipt=json!({"version":1,"proposal_protocol":1,"token":body.token,"fingerprint":body.fingerprint,"workspace":workspace,"namespace":namespace,"request_id":body.request_id,"host":body.host,"after_hashes":body.after_hashes,"report_fingerprint":intention,"status":"host-reported draft application","evidence_authority":"host report; not independently observed by the engine","saved":false,"checks_executed":false});held.applications.insert(body.token.clone(),receipt.clone());(review,receipt)
 };
 let durability=archive_review(&state,review,Some(receipt.clone())).await;
 let mut receipt=receipt;if durability["retained"]!=true{receipt["durability"]=durability.clone();}
 let mut held=REVIEWS.lock().map_err(|_|ApiError::conflict("Proposal retention unavailable"))?;if held.records.contains_key(&body.token){held.applications.insert(body.token.clone(),receipt.clone());held.durability.insert(body.token,durability);}Ok(Json(receipt))
}
</ui:copy>

## Refuse a changed or unbounded intention before analysis

The focused wire cases bind explanation, implementation and requested checks in
one fingerprint. They exercise malformed identities, duplicate check references
and bounded claims; they do not establish a usable host Apply path.

<ui:copy id="agent-proposal-tests">
#[cfg(test)]
mod tests {
 use super::*;
 pub(super) fn intention()->Intention{serde_json::from_value(json!({"version":1,"request_id":"0123456789abcdef0123456789abcdef","origin":"owned provider fixture","title":"Change note","explanation":"Keep the change and its reasoning together.","claims":["Expected behavior; not an observed verdict"],"check_ids":[],"capture":{"context":"context","edit":{"changes":{}},"drafts":{},"ownership_sources":{},"project_entry":null},"reading":{"version":1,"entry":"project.md","sources":{},"missing_sources":[],"focus":null}})).unwrap()}
 #[test]
 fn recorded_diff_requires_every_exact_draft_transition_before_review() {
    let fixture = |identity: &str, text: &str| resource_changes::SourceState::File {
        identity: identity.into(), text: text.into(), writable: true, removable: true, version: None,
    };
    let mut plan = resource_changes::ResourcePlan { version: 1, context: "context".into(), title: "Review".into(),
        annotations: Vec::new(), patches: vec![
            resource_changes::ResourcePatch { path: "a.py".into(), before: fixture("1", "old\r\n"), after: fixture("1", "new\r\n") },
            resource_changes::ResourcePatch { path: "b.py".into(), before: fixture("2", "before\n"), after: fixture("2", "after\n") },
        ] };
    let diff = "--- a/a.py\n+++ b/a.py\n@@ -1 +1 @@\n-old\r\n+new\r\n--- a/b.py\n+++ b/b.py\n@@ -1 +1 @@\n-before\n+after\n";
    let before = plan.patches.iter().map(|patch| (patch.path.clone(), patch.before.clone())).collect();
    verify_recorded_patch(diff, &plan, &before).unwrap();
    plan.patches[0].before = fixture("1", "old\n");
    assert!(verify_recorded_patch(diff, &plan, &before).is_err());
    plan.patches[0].before = fixture("1", "old\r\n");
    plan.patches[0].after = fixture("1", "forged addition\r\n");
    assert!(verify_recorded_patch(diff, &plan, &before).is_err());
    plan.patches.pop();
    assert!(verify_recorded_patch(diff, &plan, &before).is_err());
 }
 #[test]
 fn recorded_patch_derives_guarded_edit_and_rename_from_exact_preimages() {
    use resource_changes::SourceState;
    let sources = BTreeMap::from([
        ("from #%.py".into(), SourceState::File { identity: "stable".into(), text: "😀=1\r\n".into(), writable: true, removable: true, version: Some(17) }),
        ("to #%.py".into(), SourceState::Missing { creatable: true }),
    ]);
    let diff = "--- a/from #%.py\n+++ b/to #%.py\n@@ -1 +1 @@\n-😀=1\r\n+😀=2\r\n";
    let targets = hick_lang::project_patch_targets(diff).unwrap();
    let edit = recorded_patch_edit(diff, &targets, &sources).unwrap();
    let plan = resource_changes::prepare(resource_changes::Request { context: "fixture".into(), title: "Move and edit".into(), edit, sources: sources.clone() }).unwrap();
    verify_recorded_patch(diff, &plan, &sources).unwrap();
    let destination = plan.patches.iter().find(|patch| patch.path == "to #%.py").unwrap();
    assert!(matches!(&destination.after, SourceState::File { identity, text, .. } if identity == "stable" && text == "😀=2\r\n"));
 }
 #[test] fn derived_readings_keep_original_named_and_anonymous_capture_authority(){
  let mut value=intention();value.capture.saved_guards.insert("a.txt".into(),"original saved guard".into());value.capture.unaddressed_sources.insert("retained-note".into(),"ordinary retained note".into());
  value.capture.drafts.insert("b.txt".into(),serde_json::from_value(json!({"text":"draft","base_sha256":"a".repeat(64),"version":null,"writable":true,"save_guard":"original target guard"})).unwrap());
  let original=serde_json::to_value(&value).unwrap();let mut reading=value.reading.clone();carry_context(&mut reading,&value.capture).unwrap();
  assert_eq!(reading.saved_guards["a.txt"],"original saved guard");assert_eq!(reading.saved_guards["b.txt"],"original target guard");assert_eq!(reading.unaddressed_sources["retained-note"],"ordinary retained note");assert_eq!(serde_json::to_value(&value).unwrap(),original);
  reading.saved_guards.insert("a.txt".into(),"different proof".into());assert!(carry_context(&mut reading,&value.capture).is_err());
  let mut reading=value.reading.clone();reading.unaddressed_sources.insert("retained-note".into(),"different draft".into());assert!(carry_context(&mut reading,&value.capture).is_err());
 }
 #[tokio::test]async fn static_agent_snapshot_never_waits_for_an_unimplemented_live_capture_bridge(){
  let edits=super::super::acp::edits::Edits::default();edits.buffers.lock().unwrap().push(serde_json::from_value(json!({"id":"buffer","name":"note.py","path":"note.py","content":"retained static draft","save_guard":"original guard"})).unwrap());
  let(buffers,live)=tokio::time::timeout(std::time::Duration::from_secs(1),captured_agent_buffers(&edits)).await.unwrap().unwrap();assert!(!live);assert_eq!(buffers[0].content,"retained static draft");assert_eq!(buffers[0].save_guard.as_deref(),Some("original guard"));assert!(edits.snapshot()["captureRequest"].is_null());
 }
 #[tokio::test]async fn live_agent_capture_requires_its_exact_requested_ui_reply(){
  let edits=Arc::new(super::super::acp::edits::Edits::default());edits.live_context.store(true,std::sync::atomic::Ordering::Release);let worker=edits.clone();let task=tokio::spawn(async move{captured_agent_buffers(&worker).await});
  let id=tokio::time::timeout(std::time::Duration::from_secs(2),async{loop{if let Some(id)=edits.snapshot()["captureRequest"].as_str(){break id.to_string()}tokio::task::yield_now().await;}}).await.unwrap();assert!(edits.refreshed("old-reply",vec![]).is_err());
  edits.refreshed(&id,vec![serde_json::from_value(json!({"id":"buffer","name":"note.py","path":"note.py","content":"current live draft","save_guard":"original live guard"})).unwrap()]).unwrap();let(buffers,live)=task.await.unwrap().unwrap();assert!(live);assert_eq!(buffers[0].content,"current live draft");
 }
 #[test]fn managed_editor_binding_keeps_original_identity_revision_and_raw_source_hash(){
  let mut buffer:super::super::agent_context::EditorBuffer=serde_json::from_value(json!({"id":"original-handle","name":"a.py","path":"a.py","content":"value=41\n","save_guard":"original-physical-proof","observations":[{"kind":"editor_revision","version":1,"source_revision":"7"}]})).unwrap();let bound=bind_editors(&[buffer.clone()],false).unwrap();assert!(!bound.live);assert_eq!(bound.buffers[0].id.as_deref(),Some("original-handle"));assert_eq!(bound.buffers[0].source_revision.as_deref(),Some("7"));assert_eq!(bound.buffers[0].source_hash,super::super::workspace_fs::view::hash(buffer.content.as_bytes()));
  buffer.observations.push(json!({"kind":"editor_revision","version":1,"source_revision":"8"}));assert!(bind_editors(&[buffer.clone()],false).is_err());buffer.observations=vec![json!({"kind":"editor_revision","version":1,"source_revision":"07"})];assert!(bind_editors(&[buffer],false).is_err());
 }
 #[test] fn complete_intention_identity_includes_reasoning_implementation_and_checks(){let original=intention();bounded(&original).unwrap();let base=hash(&original).unwrap();let mut changed=original.clone();changed.explanation.push_str(" Revised.");assert_ne!(hash(&changed).unwrap(),base);changed=original.clone();changed.capture.edit=json!({"changes":{"hick:///a.py":[]}});assert_ne!(hash(&changed).unwrap(),base);changed=original.clone();changed.check_ids.push("declared-check".into());assert_ne!(hash(&changed).unwrap(),base);changed=original.clone();changed.claims[0]="Agent says passed; still only a claim".into();assert_ne!(hash(&changed).unwrap(),base);}
 #[test] fn malformed_or_amplified_claims_refuse_before_any_source_effect(){let original=intention();let mut value=original.clone();value.request_id="ABCDEF0123456789abcdef0123456789".into();assert!(bounded(&value).is_err());value=original.clone();value.claims=vec!["x".repeat(8193)];assert!(bounded(&value).is_err());value=original.clone();value.check_ids=vec!["same".into(),"same".into()];assert!(bounded(&value).is_err());let mut wire=serde_json::to_value(original).unwrap();wire["passed"]=json!(true);assert!(serde_json::from_value::<Intention>(wire).is_err());}
}
</ui:copy>

## Exercise review admission against an owned engine

The HTTP witness checks the usable engine path, exact retained intention and
same-byte recreation refusal. It does not substitute for a host lease, coordinated
Apply or real agent-provider acceptance. Saved files remain unchanged throughout.

<ui:copy id="agent-proposal-http-witness">
#!/usr/bin/env python3
"""Owned-engine coherent proposal preparation and complete-intention admission."""
import hashlib,importlib.util,json,os,tempfile,urllib.error,urllib.request,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def call(origin,method,route,body=None):
 raw=None if body is None else json.dumps(body).encode()
 with urllib.request.urlopen(urllib.request.Request(origin+route,data=raw,method=method,headers={'Content-Type':'application/json'}),timeout=30) as response:return json.load(response)
def refused(action):
 try:action()
 except urllib.error.HTTPError as error:
  assert error.code in (400,403,404,409,422),(error.code,error.read());return
 raise AssertionError('A stale or forged proposal was admitted')
def suite(origin,root):
 before={path.name:path.read_bytes() for path in root.iterdir() if path.is_file()}
 capture={'context':'owned-proposal','project_entry':'project.md','drafts':{},'ownership_sources':{},'unaddressed_sources':{},'saved_guards':{},'edit':{'changes':{'hick:///note.py':[{'range':{'start':{'line':0,'character':0},'end':{'line':1,'character':0}},'newText':'value = 41\n'}]}}}
 intention={'version':1,'request_id':uuid.uuid4().hex,'origin':'owned provider fixture','title':'Explain and change one source','explanation':'Change the implementation together with its expected behavior. This is a proposal, not an observed verdict.','claims':['Agent claims tests passed; this remains unverified agent text.'],'check_ids':[],'capture':capture,'reading':{'version':1,'entry':'project.md','sources':{},'missing_sources':[],'unaddressed_sources':{},'saved_guards':{},'focus':'note.py'}}
 review=call(origin,'POST','/api/agent-proposals/prepare',intention);assert review['proposal_protocol']==1 and not review['source_effects'] and not review['checks_executed'];assert hashlib.sha256(review['material'].encode()).hexdigest()==review['fingerprint'];material=json.loads(review['material']);assert material['intention']==intention and material['namespace']==str(root) and material['workspace']==review['workspace'];assert material['origin_authority'].startswith('caller-provided')
 assert material['plan']['patches'][0]['before']['text']=='value = 1\n' and material['plan']['patches'][0]['after']['text']=='value = 41\n'
 assert material['consequences']['protected_targets']==[] and material['before']['version']==1 and material['after']['version']==1
 admission={'version':1,'token':review['token'],'fingerprint':review['fingerprint']}
 confirmed=call(origin,'POST','/api/agent-proposals/validate',admission);assert confirmed['admitted'] and confirmed['fingerprint']==review['fingerprint'] and confirmed['namespace']==str(root) and not confirmed['source_effects']
 refused(lambda:call(origin,'POST','/api/agent-proposals/validate',{**admission,'fingerprint':'f'*64}))
 listed=call(origin,'GET','/api/agent-proposals');assert listed['namespace']==str(root) and any(row['token']==review['token'] for row in listed['reviews']);assert call(origin,'GET','/api/agent-proposals/'+review['token'])==review
 report={'version':1,'token':review['token'],'fingerprint':review['fingerprint'],'workspace':review['workspace'],'namespace':str(root),'request_id':uuid.uuid4().hex,'host':'owned protocol fixture (host claim only)','after_hashes':{'note.py':hashlib.sha256(b'value = 41\n').hexdigest()}}
 refused(lambda:call(origin,'POST','/api/agent-proposals/report',{**report,'after_hashes':{}}))
 claimed=call(origin,'POST','/api/agent-proposals/report',report);assert claimed['status']=='host-reported draft application' and not claimed['saved'] and not claimed['checks_executed'];assert call(origin,'POST','/api/agent-proposals/report',report)==claimed
 refused(lambda:call(origin,'POST','/api/agent-proposals/report',{**report,'request_id':uuid.uuid4().hex}));assert call(origin,'GET','/api/agent-proposals/'+review['token'])['host_application']==claimed
 assert (root/'note.py').read_text()=='value = 1\n','A host report must not apply or save source'
 for field,value in [('explanation','changed reasoning'),('claims',['forged verdict']),('check_ids',['not-in-engine-reading'])]:
  refused(lambda field=field,value=value:call(origin,'POST','/api/agent-proposals/validate',{**admission,'intention':{**intention,field:value}}))
 refused(lambda:call(origin,'POST','/api/agent-proposals/prepare',{**intention,'request_id':uuid.uuid4().hex,'check_ids':['invented-check']}))
 generated={**intention,'request_id':uuid.uuid4().hex,'capture':{**capture,'edit':{'changes':{'hick:///generated.py':[{'range':{'start':{'line':0,'character':0},'end':{'line':1,'character':0}},'newText':'forbidden\n'}]}}}}
 refused(lambda:call(origin,'POST','/api/agent-proposals/prepare',generated))
 replacement=root/'replacement.tmp';replacement.write_bytes((root/'note.py').read_bytes());os.replace(replacement,root/'note.py');refused(lambda:call(origin,'POST','/api/agent-proposals/validate',admission))
 assert {path.name:path.read_bytes() for path in root.iterdir() if path.is_file()}==before
 print('PASS coherent proposal HTTP: complete explanation/implementation/consequence/readings, exact unchanged admission, sealed byte-exact material, immutable admission, retained review/list and host reports, changed claims/checks refusal, generated ownership and same-byte replacement refusal, no source/save/check effects',flush=True)
def main():
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-agent-proposal-') as folder:
  temporary=Path(folder);root=temporary/'workspace';root.mkdir();root=root.resolve()
  for path,text in {'project.md':'<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="construction.md"/></h:doc>\n','construction.md':'<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="generated.py">generated = 1\n</h:file></h:doc>\n','generated.py':'generated = 1\n','note.py':'value = 1\n'}.items():(root/path).write_text(text)
  process=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(temporary/'state')})
  try:suite(fixture.origin(process),root)
  finally:fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-agent-proposals.py"><ui:paste select="#agent-proposal-http-witness"/></ui:file>

## Exercise the private-policy boundary before lookup or provider work

The existing prepared-private-session safety fixture calls every proposal entry
point. Even a cache token or malformed tool argument must refuse at the private
boundary before looking up ordinary retained content, refreshing editors or
starting construction analysis. This extends that focused fixture, not a broad
regression claim.

<ui:copy id="agent-proposal-private-witness">
#[cfg(test)] pub(super) async fn assert_private_refused(state:&LocalState){
 let value=tests::intention();let token="0123456789abcdef0123456789abcdef".to_string();
 let actions=[prepare(State(state.clone()),Json(value.clone())).await,validate(State(state.clone()),Json(Admission{version:1,token:token.clone(),fingerprint:"unknown".into()})).await,list(State(state.clone())).await,read(State(state.clone()),axum::extract::Path(token.clone())).await,report(State(state.clone()),Json(HostReport{version:1,token,fingerprint:"unknown".into(),workspace:"unknown".into(),namespace:"unknown".into(),request_id:"0123456789abcdef0123456789abcdef".into(),host:"private refusal witness".into(),after_hashes:BTreeMap::new()})).await,from_agent(state.clone(),&super::acp::edits::Edits::default(),"private".into(),&Value::Null).await];
 for action in actions{match action{Ok(_)=>panic!("Ordinary proposal entry admitted private prediction state"),Err(error)=>assert!(error.message().contains("Private prediction"),"{}",error.message())}}
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/agent_proposals.rs"><ui:paste select="#agent-proposal-model"/><ui:paste select="#agent-proposal-archive"/><ui:paste select="#agent-proposal-archive-witness"/><ui:paste select="#agent-proposal-archive-api"/><ui:paste select="#agent-proposal-archive-routes"/><ui:paste select="#agent-proposal-analysis"/><ui:paste select="#agent-proposal-routes"/><ui:paste select="#agent-proposal-tool-discovery"/><ui:paste select="#agent-proposal-host-report"/><ui:paste select="#agent-proposal-tests"/><ui:paste select="#agent-proposal-private-witness"/></ui:file>
</ui:doc>
