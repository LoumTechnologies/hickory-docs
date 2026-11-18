<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep the main construction identity while addresses change

For maintainers binding optional compiler readings to the workspace main role without turning text availability into a capability prerequisite.

## Retain the role separately from its address

A stable identity and checked generation distinguish the main construction from a same-named replacement. Initial no-main discovery is explicitly different from absence after a known role: only the first may discover a newly authored default. Transition declarations below are internal policy models; this reading endpoint does not yet admit source moves or publish inverse transactions.

<ui:copy id="workspace-entry-model">
use serde::{Serialize,Deserialize};
use std::collections::BTreeMap;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EntryStamp {pub version:u32,pub identity:String,pub generation:u32,pub path:Option<String>}
impl EntryStamp {
 fn validate(&self)->Result<(),String>{if self.version!=1||self.identity.len()!=32||!self.identity.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){return Err("Workspace entry identity is malformed.".into())}if let Some(path)=&self.path{super::resource_changes::validate_path(path)?;}Ok(())}
 fn advance(&self,path:Option<String>)->Result<Self,String>{self.validate()?;let result=Self{version:1,identity:self.identity.clone(),generation:self.generation.checked_add(1).ok_or("Workspace entry generation capacity exhausted; no prior guard can be reused.")?,path};result.validate()?;Ok(result)}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub(crate) enum TransitionKind {Move,Create,Restoration,Repair}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EntryChange {pub version:u32,pub kind:TransitionKind,pub before:EntryStamp,pub after:EntryStamp,pub source_identity:Option<String>}
impl EntryChange {
 fn validate_shape(&self)->Result<(),String>{self.before.validate()?;self.after.validate()?;if self.version!=1||self.before.identity!=self.after.identity||self.before.generation.checked_add(1)!=Some(self.after.generation){return Err("Workspace entry transition must advance one retained identity without reusing an old generation.".into())}Ok(())}
 fn validate(&self,plan:&super::resource_changes::ResourcePlan)->Result<(),String>{
  self.validate_shape()?;
  use super::resource_changes::SourceState;
  if let Some(before)=&self.before.path{
   let source=plan.patches.iter().find(|patch|patch.path==*before).ok_or("Workspace entry transition lacks its original source patch.")?;
   let identity=match &source.before{SourceState::File{identity,..}=>identity,_=>return Err("Workspace entry transition requires the original authored source identity.".into())};
   if self.source_identity.as_deref()!=Some(identity.as_str()){return Err("Workspace entry transition changed its source identity.".into())}
   if self.after.path.as_ref()!=Some(before)&&!matches!(source.after,SourceState::Missing{..}){return Err("Original workspace entry was not removed by its reviewed transition.".into())}
   if let Some(after)=&self.after.path{let destination=plan.patches.iter().find(|patch|patch.path==*after).ok_or("Workspace entry destination is absent from the reviewed namespace.")?;if !matches!(&destination.after,SourceState::File{identity:current,..}if current==identity){return Err("Workspace entry destination does not retain the reviewed source identity.".into())}}
  }else if let Some(after)=&self.after.path{
   let destination=plan.patches.iter().find(|patch|patch.path==*after).ok_or("Workspace entry creation is absent from the reviewed namespace.")?;
   if !matches!(&destination.before,SourceState::Missing{..})||!matches!(&destination.after,SourceState::File{identity,..}if self.source_identity.as_deref()==Some(identity.as_str())){return Err("Workspace entry creation does not match the reviewed source identity.".into())}
  }else if self.kind!=TransitionKind::Repair{return Err("Only a reviewed entry repair can advance an absent entry without creating it.".into())}
  Ok(())
 }
}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {request_id:String,manifest_sha256:String,change:EntryChange,after_guard:Option<String>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedTransition {request_id:String,manifest_sha256:String,change:EntryChange,after_guard:Option<String>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {version:u32,physical_root:String,namespace:String,physical_store:String,current:EntryStamp,bootstrap_default:bool,pending:Option<Pending>,history:Vec<RetainedTransition>}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedBinding {payload:Binding,seal:String}
fn digest(bytes:&[u8])->String{super::workspace_fs::view::hash(bytes)}
fn initial_stamp(path:Option<String>)->Result<EntryStamp,String>{let mut bytes=[0;16];getrandom::fill(&mut bytes).map_err(|error|error.to_string())?;Ok(EntryStamp{version:1,identity:hex::encode(bytes),generation:0,path})}
fn history_transition(binding:&Binding,record:&super::resource_journal::Recovery)->Result<EntryChange,String>{
 let known=binding.history.iter().find(|old|old.request_id==record.id&&old.manifest_sha256==record.manifest_sha256).ok_or("This retained record has no sealed workspace-entry transition authority; it remains readable evidence only.")?;
 if binding.current!=known.change.after{return Err("Workspace entry changed since this retained namespace. Restore its latest transition first.".into())}
 let before=binding.current.clone();
 let inverse=record.manifest.reviewed_plan.inverse();
 // Follow the surviving construction identity, not the old address. Adopting an
 // existing construction as main must not discard its role when that adoption
 // is reversed; an actual created source may instead be removed completely.
 let identity=if let Some(path)=&before.path{inverse.patches.iter().find(|patch|&patch.path==path).and_then(|patch|match &patch.before{super::resource_changes::SourceState::File{identity,..}=>Some(identity.clone()),_=>None}).ok_or("Retained entry inverse lacks its current original source identity.")?}else{known.change.source_identity.clone().ok_or("Retained absent entry lacks its original construction identity.")?};
 let destinations:Vec<_>=inverse.patches.iter().filter_map(|patch|match &patch.after{super::resource_changes::SourceState::File{identity:after,..}if after==&identity=>Some(patch.path.clone()),_=>None}).collect();
 if destinations.len()>1{return Err("Retained entry inverse has ambiguous surviving construction identities.".into())}
 let after=before.advance(destinations.into_iter().next())?;
 let change=EntryChange{version:1,kind:TransitionKind::Restoration,before,after,source_identity:Some(identity)};
 change.validate(&inverse)?;Ok(change)
}
</ui:copy>

## Authenticate the original physical root and namespace

Use a distinct private namespace and HMAC domain. Finding the physical bucket after a folder rename must produce a namespace refusal, not a fresh project.md default. Missing metadata with an existing key refuses; damaged or substituted storage cannot create a new role. Opening a readonly snapshot never promotes the initial default.

<ui:copy id="workspace-entry-storage">
#[cfg(unix)]
mod storage {
 use super::*;
 use super::super::private_record::{HeldRecord,Spec};
 const SPEC:Spec=Spec{namespace:"workspace-entry-v1",key:"entry.key",lock:"entry.lock",record:"entry.json",limit:1024*1024,physical_bucket:true};
 const DOMAIN:&[u8]=b"hickory.workspace-entry.binding.v1";
 pub(super) struct Bound {pub held:HeldRecord,pub payload:Binding,original:Option<Vec<u8>>}
 impl Bound {
  pub(super) fn open(root:&std::path::Path,exclusive:bool)->Result<Self,String>{
   let base=hickory_workspace::data_root().map_err(|error|error.to_string())?;
   Self::open_under(&base,root,exclusive)
  }
  pub(super) fn open_under(base:&std::path::Path,root:&std::path::Path,exclusive:bool)->Result<Self,String>{
   let held=if exclusive{HeldRecord::under(base,root,SPEC)?}else{HeldRecord::snapshot(base,root,SPEC)?};
   let original=held.read()?;
   Self::decode(root,held,original)
  }
  fn decode(root:&std::path::Path,held:HeldRecord,original:Option<Vec<u8>>)->Result<Self,String>{
   let bytes=original.as_ref().ok_or("Workspace entry has not been initialized; no missing record grants default-entry authority.")?;
   let record:SealedBinding=serde_json::from_slice(bytes).map_err(|_|"Workspace-entry binding is damaged; source remains readable but writes need its original private record.")?;
   let signed=serde_json::to_vec(&record.payload).map_err(|error|error.to_string())?;
   super::super::private_record::verify(&held.key,Some(DOMAIN),&signed,&record.seal)?;
   let payload=record.payload;
   if payload.version!=1||payload.physical_root!=held.root_id||payload.physical_store!=held.store_id||payload.namespace!=root.canonicalize().map_err(|error|error.to_string())?.to_str().ok_or("Workspace namespace is not exact UTF-8")?{return Err("Original workspace entry belongs to another physical root or namespace; no project.md fallback was selected.".into())}
   payload.current.validate()?;
   if payload.history.len()>256{return Err("Workspace-entry retained transition bound is invalid.".into())}
   let mut ids=std::collections::BTreeSet::new();
   for row in &payload.history{row.change.validate_shape()?;if !ids.insert(&row.request_id)||!super::super::resource_saves::hex(&row.request_id,32)||!super::super::resource_saves::hex(&row.manifest_sha256,64){return Err("Workspace-entry retained transition identities are malformed or repeated.".into())}}
   if let Some(pending)=&payload.pending{pending.change.validate_shape()?;if pending.change.before!=payload.current{return Err("Pending entry publication does not begin at its retained current generation.".into())}}
   held.check()?;Ok(Self{held,payload,original})
  }
  pub(super) fn resolve(root:&std::path::Path)->Result<Self,String>{
   let base=hickory_workspace::data_root().map_err(|error|error.to_string())?;Self::resolve_under(&base,root)
  }
  pub(super) fn resolve_under(base:&std::path::Path,root:&std::path::Path)->Result<Self,String>{
   let _coordination=super::leases::coordination_under(base,root,true)?;
   let existing=HeldRecord::try_snapshot(base,root,SPEC)?;let absent=existing.is_none();drop(existing);
   if absent&&super::leases::role_loss_hint(root)?{return Err("Retained workspace-entry history has lost its original private binding. Restore that binding; no default entry was inferred.".into())}
   let held=HeldRecord::under(base,root,SPEC)?;let original=held.read()?;
   let mut bound=if original.is_some(){Self::decode(root,held,original)?}else{
    if !held.created_key{return Err("Workspace-entry binding is missing from its existing private store; missing metadata does not restore project.md authority.".into())}
    let path=legacy_main(root)?;let namespace=root.canonicalize().map_err(|error|error.to_string())?.to_str().ok_or("Workspace namespace is not exact UTF-8")?.to_owned();
    let payload=Binding{version:1,physical_root:held.root_id.clone(),namespace,physical_store:held.store_id.clone(),bootstrap_default:path.is_none(),current:initial_stamp(path)?,pending:None,history:vec![]};
    let mut bound=Self{held,payload,original};bound.persist()?;bound
   };
   if bound.payload.bootstrap_default&&bound.payload.pending.is_none(){if let Some(path)=legacy_main(root)?{bound.payload.current=bound.payload.current.advance(Some(path))?;bound.payload.bootstrap_default=false;bound.persist()?;}}
   Ok(bound)
  }
  pub(super) fn optional_under(base:&std::path::Path,root:&std::path::Path)->Result<Option<Self>,String>{match HeldRecord::try_snapshot(base,root,SPEC)?{Some(held)=>{let original=held.read()?;Self::decode(root,held,original).map(Some)},None=>{if super::leases::role_loss_hint(root)?{Err("Original workspace-entry binding is unavailable while its retained transition hint remains. No missing-entry permission was granted.".into())}else{Ok(None)}}}}
  pub(super) fn snapshot_or_initialize(root:&std::path::Path)->Result<Self,String>{let base=hickory_workspace::data_root().map_err(|error|error.to_string())?;Self::snapshot_or_initialize_under(&base,root)}
  pub(super) fn snapshot_or_initialize_under(base:&std::path::Path,root:&std::path::Path)->Result<Self,String>{
   if let Some(bound)=Self::optional_under(base,root)?{if !bound.payload.bootstrap_default||bound.payload.pending.is_some()||super::legacy_main(root)?.is_none(){return Ok(bound)}drop(bound);}
   drop(Self::resolve_under(base,root)?);Self::optional_under(base,root)?.ok_or("Workspace-entry initialization did not produce its original binding.".into())
  }
  pub(super) fn unchanged(&self)->Result<(),String>{self.held.check()?;if self.held.read()?!=self.original{return Err("Workspace-entry record changed after its original lease. No new role was selected.".into())}Ok(())}
  pub(super) fn persist(&mut self)->Result<(),String>{
   let signed=serde_json::to_vec(&self.payload).map_err(|error|error.to_string())?;
   let seal=super::super::private_record::seal(&self.held.key,Some(DOMAIN),&signed)?;
   let typed=SealedBinding{payload:self.payload.clone(),seal};
   let bytes=serde_json::to_vec(&typed).map_err(|error|error.to_string())?;
   self.held.replace(&self.original,&bytes)?;self.original=Some(bytes);Ok(())
  }
 }
}
</ui:copy>

## Bind a reading to the original source observation

A reading proof binds runtime, namespace, stable role generation and the exact physical source guard. Before and after compiler work, consumers validate the original proof instead of refreshing it. Confined source reads reject symlink parents; edit/Undo and equal-byte replacement cannot revive an earlier guard. Private prediction cannot borrow this ordinary store.

<ui:copy id="workspace-entry-selection">
fn ordinary(state:&super::LocalState)->Result<(),String>{if state.private_prediction.is_some()||crate::engine::private_policy::active().map_err(|error|error.to_string())?{return Err("Private prediction cannot use ordinary workspace-entry records or proofs.".into())}Ok(())}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EntrySelection {
 pub version:u32,pub entry_protocol:u32,pub workspace:String,pub namespace:String,
 pub entry:EntryStamp,pub guard:String,
 #[serde(default,skip_serializing_if="Option::is_none")]pub source_guard:Option<String>,
 #[serde(default,skip_serializing_if="Option::is_none")]pub source_hash:Option<String>,
 pub movable:bool,pub source_effects:bool,
}
fn entry_observation(root:&std::path::Path,path:&str)->Result<super::source_history::SourceRead,String>{
 let(full,before)=super::resource_inputs::saved_target(root,path)?;
 let super::resource_inputs::SavedSource::File{text,..}=before else{return Err("Configured workspace entry is missing; no fallback entry was selected.".into())};
 let observed=super::source_history::read_observed(root,&full,path)?;
 let(_,after)=super::resource_inputs::saved_target(root,path)?;
 if observed.source!=text||!matches!(after,super::resource_inputs::SavedSource::File{text:current,..}if current==observed.source){return Err("Workspace entry changed while its confined source was captured.".into())}
 if let Some(guard)=&observed.save_guard{super::source_history::validate_observed_guard(root,&full,path,guard)?;}
 Ok(observed)
}
fn selection_material(proof:&EntrySelection)->Result<Vec<u8>,String>{serde_json::to_vec(&(proof.version,proof.entry_protocol,&proof.workspace,&proof.namespace,&proof.entry,&proof.source_guard,&proof.source_hash,proof.movable,proof.source_effects)).map_err(|error|error.to_string())}
#[cfg(unix)]
pub(crate) fn validate_entry_selection(state:&super::LocalState,proof:&EntrySelection)->Result<(),String>{
 let _ordinary=crate::engine::private_policy::ordinary();ordinary(state)?;
 let root=state.index.root();
 let bound=storage::Bound::open(root,false)?;let result=validate_selection_snapshot(root,&state.resource_root,proof,&bound);ordinary(state)?;result
}
#[cfg(unix)]
fn validate_selection_snapshot(root:&std::path::Path,identity:&super::resource_inputs::RootIdentity,proof:&EntrySelection,bound:&storage::Bound)->Result<(),String>{
 if proof.version!=1||proof.entry_protocol!=1||proof.source_effects||proof.workspace!=identity.check(root)?||proof.namespace!=bound.payload.namespace{return Err("Workspace-entry selection belongs to another runtime or namespace. Preserve the current source and select the entry again.".into())}
 proof.entry.validate()?;
 if bound.payload.pending.is_some()||bound.payload.current!=proof.entry{return Err("Workspace entry changed or publication remains unconfirmed. Original entry selection was not renewed.".into())}
 super::private_record::verify(&bound.held.key,Some(b"hickory.workspace-entry.selection.v1"),&selection_material(proof)?,&proof.guard)?;
 if let Some(path)=&proof.entry.path{
  let observed=entry_observation(root,path)?;
  if proof.source_guard.is_none()||proof.source_guard!=observed.save_guard||proof.source_hash.as_deref()!=Some(format!("sha256:{}",digest(observed.source.as_bytes())).as_str()){return Err("Workspace entry source changed after this selection. Its old compiler closure cannot authorize the current request.".into())}
 }else if proof.source_guard.is_some()||proof.source_hash.is_some()||proof.movable{return Err("An absent workspace entry cannot carry a source or move capability.".into())}
 bound.held.check()?;identity.check(root)?;Ok(())
}
#[cfg(not(unix))]
pub(crate) fn validate_entry_selection(_state:&super::LocalState,_proof:&EntrySelection)->Result<(),String>{Err("Durable workspace-entry proofs are unavailable on this engine platform.".into())}
</ui:copy>

## Issue optional capability evidence without blocking source text

Only this optional endpoint waits for root readiness. Ordinary file opening remains independent. A malformed main retains its role and exact readable bytes but offers no move capability. An unconfirmed transition refuses a new reading scope; it never selects an arbitrary fallback entry.

<ui:copy id="workspace-entry-reading">
fn legacy_main(root:&std::path::Path)->Result<Option<String>,String>{
 let(_,source)=super::resource_inputs::saved_target(root,"project.md")?;
 match source{super::resource_inputs::SavedSource::Missing=>Ok(None),super::resource_inputs::SavedSource::File{text,..}=>match hick_lang::parse(&text){Ok(document)=>Ok((!document.find_tags("project").is_empty()).then(||"project.md".to_owned())),Err(_)=>Ok(Some("project.md".to_owned()))}}
}
#[cfg(unix)]
fn read_selection(state:&super::LocalState)->Result<EntrySelection,String>{
 let _ordinary=crate::engine::private_policy::ordinary();ordinary(state)?;
 let root=state.index.root();state.resource_root.check(root)?;let bound=storage::Bound::snapshot_or_initialize(root)?;
 let proof=issue_selection(root,&state.resource_root,&bound)?;ordinary(state)?;Ok(proof)
}
#[cfg(unix)]
fn issue_selection(root:&std::path::Path,identity:&super::resource_inputs::RootIdentity,bound:&storage::Bound)->Result<EntrySelection,String>{
 if bound.payload.pending.is_some(){return Err("Workspace-entry publication remains unconfirmed. Read source and retained recovery before changing its scope.".into())}
 let entry=bound.payload.current.clone();
 let(source_guard,source_hash,movable)=if let Some(path)=&entry.path{
  let observed=entry_observation(root,path)?;
  let declared=hick_lang::parse(&observed.source).map(|document|!document.find_tags("project").is_empty()).unwrap_or(false);
  let writable=std::fs::metadata(root.join(path)).map_err(|error|error.to_string())?.permissions().readonly()==false;
  (Some(observed.save_guard.ok_or_else(||observed.save_guard_reason.unwrap_or("Physical entry source identity is unavailable.".into()))?),Some(format!("sha256:{}",digest(observed.source.as_bytes()))),declared&&writable)
 }else{(None,None,false)};
 let mut proof=EntrySelection{version:1,entry_protocol:1,workspace:identity.check(root)?.to_owned(),namespace:bound.payload.namespace.clone(),entry,guard:String::new(),source_guard,source_hash,movable,source_effects:false};
 proof.guard=super::private_record::seal(&bound.held.key,Some(b"hickory.workspace-entry.selection.v1"),&selection_material(&proof)?)?;
 bound.held.check()?;identity.check(root)?;Ok(proof)
}
pub(crate) async fn get(axum::extract::State(state):axum::extract::State<super::LocalState>)->super::api::ApiResult<axum::Json<EntrySelection>>{
 ordinary(&state).map_err(super::api::ApiError::conflict)?;
 state.resource_root.ready().await.map_err(super::api::ApiError::conflict)?;
 #[cfg(unix)]{tokio::task::spawn_blocking(move||read_selection(&state)).await.map_err(|error|super::api::ApiError::internal(error.to_string()))?.map(axum::Json).map_err(super::api::ApiError::conflict)}
 #[cfg(not(unix))]{let _=state;Err(super::api::ApiError::conflict("Durable workspace-entry proofs are unavailable on this engine platform. Source remains readable."))}
}
</ui:copy>

## Exercise restart, stale proofs and missing authority

These focused witnesses issue and validate real sealed snapshots under owned roots. They distinguish a fresh runtime from the retained role, exercise edit/Undo and inode replacement, reject tampered metadata and changed namespace, and keep malformed text readable. Source relocation, reviewed inverse/redo and host interaction remain separate implementation and acceptance work.

<ui:copy id="workspace-entry-reading-witness">
#[cfg(all(test,unix))]
mod tests {
 use super::*;
 use std::path::{Path,PathBuf};
 fn declared()->&'static str{"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project/></h:doc>\n"}
 fn fixture(main:bool)->(tempfile::TempDir,PathBuf,PathBuf){let temporary=tempfile::tempdir().unwrap();let root=temporary.path().join("workspace");std::fs::create_dir(&root).unwrap();let root=root.canonicalize().unwrap();if main{std::fs::write(root.join("project.md"),declared()).unwrap();}(temporary,root.clone(),root.parent().unwrap().join("state"))}
 fn issue(base:&Path,root:&Path,identity:&super::super::resource_inputs::RootIdentity)->EntrySelection{let bound=storage::Bound::resolve_under(base,root).unwrap();issue_selection(root,identity,&bound).unwrap()}
 fn validate(base:&Path,root:&Path,identity:&super::super::resource_inputs::RootIdentity,proof:&EntrySelection)->Result<(),String>{let bound=storage::Bound::open_under(base,root,false)?;validate_selection_snapshot(root,identity,proof,&bound)}
 #[test]
 fn restart_retains_role_but_never_revives_old_runtime_proof(){let(_temporary,root,base)=fixture(true);let first=super::super::resource_inputs::RootIdentity::bind(&root);let old=issue(&base,&root,&first);validate(&base,&root,&first,&old).unwrap();let restarted=super::super::resource_inputs::RootIdentity::bind(&root);assert!(validate(&base,&root,&restarted,&old).is_err());let fresh=issue(&base,&root,&restarted);assert_eq!(fresh.entry,old.entry);assert_ne!(fresh.guard,old.guard);assert_eq!(fresh.source_hash,old.source_hash);assert!(!fresh.source_effects);validate(&base,&root,&restarted,&fresh).unwrap();assert_eq!(std::fs::read_to_string(root.join("project.md")).unwrap(),declared());}
 #[test]
 fn entry_edits_undo_and_equal_byte_replacement_retire_original_proofs(){let(_temporary,root,base)=fixture(true);let identity=super::super::resource_inputs::RootIdentity::bind(&root);let old=issue(&base,&root,&identity);std::fs::write(root.join("project.md"),"changed").unwrap();assert!(validate(&base,&root,&identity,&old).is_err());std::fs::write(root.join("project.md"),declared()).unwrap();assert!(validate(&base,&root,&identity,&old).is_err());let current=issue(&base,&root,&identity);let replacement=root.join("replacement");std::fs::write(&replacement,declared()).unwrap();std::fs::rename(replacement,root.join("project.md")).unwrap();assert!(validate(&base,&root,&identity,&current).is_err());let fresh=issue(&base,&root,&identity);validate(&base,&root,&identity,&fresh).unwrap();assert_eq!(fresh.entry,current.entry);}
 #[test]
 fn no_main_discovery_happens_once_and_missing_configured_main_never_falls_back(){let(_temporary,root,base)=fixture(false);let identity=super::super::resource_inputs::RootIdentity::bind(&root);let absent=issue(&base,&root,&identity);assert!(absent.entry.path.is_none());assert!(!absent.movable);std::fs::write(root.join("project.md"),declared()).unwrap();let configured=issue(&base,&root,&identity);assert_eq!(configured.entry.identity,absent.entry.identity);assert_eq!(configured.entry.generation,absent.entry.generation+1);assert!(validate(&base,&root,&identity,&absent).is_err());std::fs::remove_file(root.join("project.md")).unwrap();let bound=storage::Bound::resolve_under(&base,&root).unwrap();assert!(!bound.payload.bootstrap_default);assert_eq!(bound.payload.current,configured.entry);assert!(issue_selection(&root,&identity,&bound).is_err());}
 #[test]
 fn modified_binding_or_changed_root_namespace_cannot_borrow_the_original_role(){let(temporary,root,base)=fixture(true);let bound=storage::Bound::resolve_under(&base,&root).unwrap();let record=bound.held.path.join("entry.json");drop(bound);let bytes=std::fs::read(&record).unwrap();let mut forged:serde_json::Value=serde_json::from_slice(&bytes).unwrap();forged["payload"]["current"]["generation"]=serde_json::json!(99);std::fs::write(&record,serde_json::to_vec(&forged).unwrap()).unwrap();assert!(storage::Bound::resolve_under(&base,&root).is_err());std::fs::write(&record,bytes).unwrap();let renamed=temporary.path().join("renamed");std::fs::rename(&root,&renamed).unwrap();assert!(storage::Bound::resolve_under(&base,&renamed).is_err());}
 #[test]
 fn malformed_main_is_retained_as_readable_role_without_move_authority(){let(_temporary,root,base)=fixture(true);drop(storage::Bound::resolve_under(&base,&root).unwrap());std::fs::write(root.join("project.md"),"<h:doc").unwrap();let identity=super::super::resource_inputs::RootIdentity::bind(&root);let proof=issue(&base,&root,&identity);assert_eq!(proof.entry.path.as_deref(),Some("project.md"));assert!(!proof.movable);validate(&base,&root,&identity,&proof).unwrap();assert_eq!(std::fs::read_to_string(root.join("project.md")).unwrap(),"<h:doc");}
 #[test]
 fn forged_selection_fields_and_unconfirmed_publication_refuse_whole_reading_scope(){let(_temporary,root,base)=fixture(true);let identity=super::super::resource_inputs::RootIdentity::bind(&root);let proof=issue(&base,&root,&identity);for mutation in ["namespace","source","generation","effects"]{let mut forged=proof.clone();match mutation{"namespace"=>forged.namespace.push_str("/other"),"source"=>forged.source_hash=Some(format!("sha256:{}","0".repeat(64))),"generation"=>forged.entry.generation+=1,_=>forged.source_effects=true};assert!(validate(&base,&root,&identity,&forged).is_err());}let mut bound=storage::Bound::open_under(&base,&root,true).unwrap();bound.payload.pending=Some(Pending{request_id:"1".repeat(32),manifest_sha256:"2".repeat(64),change:EntryChange{version:1,kind:TransitionKind::Move,before:proof.entry.clone(),after:proof.entry.advance(Some("next.md".into())).unwrap(),source_identity:Some("original".into())},after_guard:None});bound.persist().unwrap();assert!(issue_selection(&root,&identity,&bound).is_err());drop(bound);assert!(validate(&base,&root,&identity,&proof).is_err());}
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/workspace_entry_authority.rs"><ui:paste select="#workspace-entry-model"/><ui:paste select="#workspace-entry-storage"/><ui:paste select="#workspace-entry-selection"/><ui:paste select="#workspace-entry-reading"/><ui:paste select="#workspace-entry-coordination-provider"/><ui:paste select="#workspace-entry-read-lease-admission"/><ui:paste select="#workspace-entry-source-coordination"/><ui:paste select="#workspace-entry-source-batch-coordination"/><ui:paste select="#workspace-entry-direct-namespace-coordination"/><ui:paste select="#workspace-entry-read-lease-witness"/><ui:paste select="#workspace-entry-reading-witness"/></ui:file>
</ui:doc>
