use super::{LocalState,api::{ApiError,ApiResult},resource_inputs::{RootIdentity,SavedSource},resource_journal::{Batch,Manifest,Receipt},resource_changes::{ResourcePlan,ResourcePatch,SourceState}};
use axum::{Json,extract::{Query,State}};
use serde::{Deserialize,Serialize};
use serde_json::{Value,json};
use std::{collections::{BTreeMap,BTreeSet},path::Path,sync::Mutex};
const KIND:&str="ordinary-source-save-v1";
static WRITERS:Mutex<()>=Mutex::new(());
fn relative_name(root:&Path,path:&Path)->Result<String,String>{
 let canonical=root.canonicalize().map_err(|e|e.to_string())?;
 let actual=if path.starts_with(root){root.to_path_buf()}else if path.starts_with(&canonical){canonical.clone()}else{
  path.ancestors().skip(1).find(|ancestor|ancestor.canonicalize().ok().as_deref()==Some(canonical.as_path())&&same_file::is_same_file(ancestor,root).unwrap_or(false)).ok_or("Source is outside this history workspace.")?.to_path_buf()
 };
 let relative=path.strip_prefix(&actual).map_err(|_|"Source is outside this history workspace.")?;
 let mut names=Vec::new();for component in relative.components(){match component{std::path::Component::Normal(name)=>names.push(name.to_str().ok_or("Source path is not UTF-8.")?),_=>return Err("Source history requires one exact workspace-relative source address.".into())}}
 let name=names.join("/");super::resource_changes::validate_path(&name)?;Ok(name)
}
pub(super) fn storage(root:&Path)->Result<std::path::PathBuf,String>{
 #[cfg(test)] let store=hickory_workspace::WorkspaceStore::under(&root.join(".history-state"),root).map_err(|e|e.to_string())?;
 #[cfg(not(test))] let store=hickory_workspace::WorkspaceStore::for_project(root).map_err(|e|e.to_string())?;
 let directory=store.dir().join("source-history");super::resource_journal::regular_directory(&directory)?;Ok(directory)
}
fn workspace(root:&Path)->Result<String,String>{RootIdentity::fingerprint(root)}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Capture {kind:String,path:String,journal_path:String,workspace:String,before_identity:String,request_fingerprint:String,at:u64}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {version:u32,manifest_sha256:String,outcome:String,identity_proven:bool,after_identity:Option<String>,observed_mode:Option<FileMode>,error:Option<String>}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct FileMode{readonly:bool,executable:bool,mode:Option<u32>}
fn file_mode(saved:&SavedSource)->Option<FileMode>{match saved{SavedSource::File{readonly,executable,mode,..}=>Some(FileMode{readonly:*readonly,executable:*executable,mode:*mode}),_=>None}}
fn observed_source(intended:&SavedSource,evidence:&Evidence)->SavedSource{let mut saved=intended.clone();if let(SavedSource::File{readonly,executable,mode,..},Some(observed))=(&mut saved,&evidence.observed_mode){*readonly=observed.readonly;*executable=observed.executable;*mode=observed.mode}saved}
fn stamp(path:&Path)->Result<String,String>{
 let meta=std::fs::symlink_metadata(path).map_err(|e|e.to_string())?;
 if !meta.is_file()||meta.file_type().is_symlink(){return Err("History requires an ordinary source file.".into())}
 #[cfg(unix)] {use std::os::unix::fs::MetadataExt;Ok(format!("{}:{}:{}:{}:{}",meta.dev(),meta.ino(),meta.ctime(),meta.ctime_nsec(),meta.len()))}
 #[cfg(not(unix))] {let _=meta;Err("Persistent physical source identity is unavailable on this engine platform. History remains readable.".into())}
}
fn source_address(root:&Path,path:&Path)->Result<String,String>{relative_name(root,path).or_else(|_|{if !path.is_absolute(){return Err("Opened source requires its exact absolute address.".into())}path.to_str().map(str::to_owned).ok_or("Opened source address is not UTF-8.".into())})}
fn journal_name(address:&str)->String{if Path::new(address).is_absolute(){format!("opened/{}",super::workspace_fs::view::hash(address.as_bytes()))}else{address.into()}}
fn observe(root:&Path,address:&str)->Result<SavedSource,String>{
 let full=if !Path::new(address).is_absolute(){let(full,saved)=super::resource_inputs::saved_target(root,address)?;if matches!(saved,SavedSource::Missing){return Ok(saved)}full}else{Path::new(address).to_path_buf()};
 use std::io::Read;let path=full.as_path();let metadata=match std::fs::symlink_metadata(path){Ok(meta)=>meta,Err(e)if e.kind()==std::io::ErrorKind::NotFound=>return Ok(SavedSource::Missing),Err(e)=>return Err(e.to_string())};
 if !metadata.is_file()||metadata.file_type().is_symlink(){return Err("Opened source is no longer its authorized ordinary file.".into())}
 let mut options=std::fs::OpenOptions::new();options.read(true);#[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK);}
 let file=options.open(path).map_err(|e|e.to_string())?;let metadata=file.metadata().map_err(|e|e.to_string())?;let identity=identity_for_metadata(&metadata)?;let mut bytes=Vec::new();file.take(8*1024*1024+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
 if bytes.len()>8*1024*1024||stamp(path)?!=identity{return Err("Opened source changed or exceeds the bounded history limit.".into())}
 let source=String::from_utf8(bytes).map_err(|_|"Opened source is not UTF-8 text.")?;
 #[cfg(unix)]let mode={use std::os::unix::fs::PermissionsExt;Some(metadata.permissions().mode()&0o7777)};
 #[cfg(not(unix))]let mode:Option<u32>=None;
 Ok(SavedSource::File{sha256:super::workspace_fs::view::hash(source.as_bytes()),text:source,readonly:metadata.permissions().readonly(),executable:mode.is_some_and(|mode|mode&0o111!=0),mode})
}
fn authorized_address(state:&LocalState,address:&str)->Result<std::path::PathBuf,String>{
 if !Path::new(address).is_absolute(){return super::resource_inputs::saved_target(state.index.root(),address).map(|(path,_)|path)}
 let path=Path::new(address);let registered=state.index.entries().iter().any(|(id,_)|state.index.absolute(id).as_deref()==Some(path));
 let opened=registered||(path.canonicalize().map_err(|e|e.to_string())?==path&&state.opened_files.read().map_err(|_|"Opened-file authority is unavailable.")?.contains(path));
 if !opened{return Err("Open this exact file explicitly before reading its private local history.".into())}Ok(path.to_path_buf())
}
fn nonce()->Result<String,String>{let mut bytes=[0u8;16];getrandom::fill(&mut bytes).map_err(|e|e.to_string())?;Ok(hex::encode(bytes))}
fn text(saved:&SavedSource)->Result<&str,String>{match saved{SavedSource::File{text,..}=>Ok(text),_=>Err("Ordinary save history requires an existing source.".into())}}
fn draft(saved:&SavedSource,identity:&str)->Result<SourceState,String>{Ok(match saved{SavedSource::Missing=>SourceState::Missing{creatable:true},SavedSource::File{text,..}=>SourceState::File{identity:identity.into(),text:text.clone(),writable:true,removable:true,version:None}})}
fn changed(saved:&SavedSource,source:&str)->Result<SavedSource,String>{let mut result=saved.clone();if let SavedSource::File{text,sha256,..}=&mut result{*text=source.into();*sha256=super::workspace_fs::view::hash(source.as_bytes());Ok(result)}else{Ok(SavedSource::File{text:source.into(),sha256:super::workspace_fs::view::hash(source.as_bytes()),readonly:false,executable:false,mode:None})}}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveGuard {version:u32,workspace:String,path:String,identity:String,source_hash:String}
pub(super) fn identity_for_metadata(meta:&std::fs::Metadata)->Result<String,String>{
 if !meta.is_file(){return Err("A physical save guard requires an ordinary file.".into())}
 #[cfg(unix)]{use std::os::unix::fs::MetadataExt;Ok(format!("{}:{}:{}:{}:{}",meta.dev(),meta.ino(),meta.ctime(),meta.ctime_nsec(),meta.len()))}
 #[cfg(not(unix))]{let _=meta;Err("This engine cannot provide a persistent physical source guard.".into())}
}
fn guard_text(root:&Path,path:&str,identity:&str,source:&str)->Result<String,String>{serde_json::to_string(&SaveGuard{version:1,workspace:workspace(root)?,path:path.into(),identity:identity.into(),source_hash:super::workspace_fs::view::hash(source.as_bytes())}).map_err(|e|e.to_string())}
pub(super) struct SourceRead {pub source:String,pub save_guard:Option<String>,pub save_guard_reason:Option<String>}
pub(super) fn read_observed(root:&Path,path:&Path,declared:&str)->Result<SourceRead,String>{
 use std::io::Read;
 let mut options=std::fs::OpenOptions::new();options.read(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK);}
 let mut file=match options.open(path){Ok(file)=>file,Err(error)=>{
  if std::fs::symlink_metadata(path).is_ok_and(|meta|meta.file_type().is_symlink()){
   let mut bytes=Vec::new();std::fs::File::open(path).map_err(|e|e.to_string())?.take(10*1024*1024+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
   if bytes.len()>10*1024*1024{return Err("Source exceeds the readable text limit.".into())}
   return Ok(SourceRead{source:String::from_utf8(bytes).map_err(|_|"Source is not UTF-8 text.")?,save_guard:None,save_guard_reason:Some("File aliases remain readable but have no physical save authority. Open their exact authored target.".into())})
  }return Err(error.to_string())
 }};
 let before=file.metadata().map_err(|e|e.to_string())?;if !before.is_file(){return Err("Choose an ordinary source file.".into())}
 let initial=identity_for_metadata(&before);let root_before=workspace(root);
 let mut bytes=Vec::new();file.by_ref().take(10*1024*1024+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
 if bytes.len()>10*1024*1024{return Err("Source exceeds the readable text limit.".into())}
 let source=String::from_utf8(bytes).map_err(|_|"Source is not UTF-8 text.")?;
 let token=(||->Result<String,String>{let identity=initial?;if identity_for_metadata(&file.metadata().map_err(|e|e.to_string())?)?!=identity||stamp(path)?!=identity||root_before?!=workspace(root)?{return Err("Source changed while reading. Its text is retained; reopen or compare before saving.".into())}guard_text(root,declared,&identity,&source)})();
 Ok(match token{Ok(save_guard)=>SourceRead{source,save_guard:Some(save_guard),save_guard_reason:None},Err(reason)=>SourceRead{source,save_guard:None,save_guard_reason:Some(reason)}})
}
pub(super) fn read_for_api(state:&LocalState,path:&Path,declared:&str)->Result<SourceRead,String>{read_observed(state.index.root(),path,declared)}
pub(super) fn writable_source(state:&LocalState,path:&Path,declared:&str)->Result<(),String>{
 state.resource_root.check(state.index.root())?;
 let metadata=std::fs::symlink_metadata(path).map_err(|e|e.to_string())?;if !metadata.is_file()||metadata.file_type().is_symlink()||metadata.permissions().readonly(){return Err("Source is read-only or its physical file changed.".into())}
 let claims=super::api::generated_outputs(state.index.root(),&state.index);
 if super::api::generated_by(&claims,declared).is_some()||claims.keys().any(|name|same_file::is_same_file(path,state.index.root().join(name)).unwrap_or(false)){return Err("Generated source or its physical alias cannot receive an ordinary authored save.".into())}
 if !Path::new(declared).is_absolute(){let(full,_)=super::resource_inputs::saved_target(state.index.root(),declared)?;if !same_file::is_same_file(&full,path).map_err(|e|e.to_string())?{return Err("The declared source address no longer names the read file.".into())}}
 Ok(())
}
pub(super) fn validate_observed_guard(root:&Path,path:&Path,declared:&str,raw:&str)->Result<(),String>{
 if raw.len()>16*1024{return Err("Physical save guard exceeds its bounded size.".into())}
 let guard:SaveGuard=serde_json::from_str(raw).map_err(|_|"Physical save guard is invalid or unsupported.")?;
 if guard.version!=1||guard.workspace!=workspace(root)?||guard.path!=declared||guard.identity!=stamp(path)?{return Err("The source was replaced or its physical revision changed. Preserve the draft and compare with the exact current file.".into())}
 let observed=read_observed(root,path,declared)?;
 if observed.save_guard.as_deref()!=Some(raw)||super::workspace_fs::view::hash(observed.source.as_bytes())!=guard.source_hash{return Err("The source changed after its physical save snapshot. Current disk bytes were retained.".into())}
 Ok(())
}
pub(super) fn validate_guard(state:&LocalState,path:&Path,declared:&str,raw:&str)->Result<(),String>{writable_source(state,path,declared)?;validate_observed_guard(state.index.root(),path,declared,raw)}
pub(super) fn issue_guard(state:&LocalState,path:&Path,source:&str)->Result<String,String>{
 let declared=relative_name(state.index.root(),path).or_else(|_|path.to_str().map(str::to_owned).ok_or_else(||"Source address is not UTF-8.".to_string()))?;
 let observed=read_for_api(state,path,&declared)?;if observed.source!=source{return Err("Published source changed before its guard was captured.".into())}observed.save_guard.ok_or_else(||observed.save_guard_reason.unwrap_or("Physical save guard is unavailable.".into()))
}
pub(super) fn guard_for_result(state:&LocalState,path:&Path,declared:&str,source:&str,id:&str,intent:&str)->Result<String,String>{
 let entry=read_entry(state.index.root(),id)?;let evidence=entry.evidence.ok_or("Save result has no confirmed physical publication.")?;
 if entry.capture.request_fingerprint!=intent||text(&entry.after)?!=source||evidence.outcome!="published"||evidence.after_identity.as_deref()!=Some(stamp(path)?.as_str()){return Err("Save publication no longer matches the current physical result.".into())}
 writable_source(state,path,declared)?;guard_text(state.index.root(),declared,evidence.after_identity.as_deref().unwrap(),source)
}
pub(super) fn replay(root:&Path,path:&Path,source:&str,request:Option<&str>,intent:Option<&str>)->Result<bool,String>{
 if request.is_some()!=intent.is_some(){return Err("Save request identity and canonical body fingerprint must be supplied together.".into())}
 let Some(id)=request else{return Ok(false)};
 if id.len()!=32||!id.bytes().all(|b|matches!(b,b'0'..=b'9'|b'a'..=b'f')){return Err("Save request must contain 32 lowercase hexadecimal characters.".into())}
 if !storage(root)?.join(".hick-cache/resource-changes").join(id).exists(){return Ok(false)}
 let entry=read_entry(root,id)?;let relative=source_address(root,path)?;
 let binding=RootIdentity::bind(root);binding.check(root)?;let workspace=workspace(root)?;let current=observe(root,&relative)?;let identity=stamp(path)?;
 let fingerprint=intent.map(str::to_owned).unwrap_or_else(||super::workspace_fs::view::hash(&serde_json::to_vec(&(&relative,source)).unwrap()));
 if entry.capture.request_fingerprint!=fingerprint||entry.capture.workspace!=workspace||entry.capture.path!=relative||text(&entry.after)?!=source||entry.after!=current||entry.evidence.as_ref().is_none_or(|e|e.outcome!="published"||!e.identity_proven||e.after_identity.as_deref()!=Some(identity.as_str())){return Err("This save identity belongs to a different body, uncertain publication or retired physical source. It was not replayed.".into())}
 Ok(true)
}
pub(super) fn write(root:&Path,path:&Path,source:&str,request:Option<&str>,intent:Option<&str>,writer:impl FnOnce()->Result<(),String>)->Result<(),String>{write_inner(root,path,source,request,intent,||writer().map(|_|None))}
pub(super) fn write_observed(root:&Path,path:&Path,source:&str,request:Option<&str>,intent:Option<&str>,writer:impl FnOnce()->Result<String,String>)->Result<(),String>{write_inner(root,path,source,request,intent,||writer().map(Some))}
fn write_inner(root:&Path,path:&Path,source:&str,request:Option<&str>,intent:Option<&str>,writer:impl FnOnce()->Result<Option<String>,String>)->Result<(),String>{
 if request.is_some()!=intent.is_some(){return Err("Save request identity and canonical body fingerprint must be supplied together.".into())}
 let _guard=WRITERS.lock().map_err(|_|"Source save history is unavailable.".to_string())?;
 let relative=source_address(root,path)?;
 let binding=RootIdentity::bind(root);binding.check(root)?;let workspace=workspace(root)?;
 let before=observe(root,&relative)?;
 // Creation keeps its explicit caller authority. The prepared mode is unknown;
 // the immutable outcome supplies the exact observed permissions after writing.
 let missing=matches!(before,SavedSource::Missing);let before_identity=if missing{format!("missing:{}",nonce()?)}else{stamp(path)?};
 let after=changed(&before,source)?;
 let request=request.map(str::to_owned).map(Ok).unwrap_or_else(nonce)?;
 if request.len()!=32||!request.bytes().all(|b|matches!(b,b'0'..=b'9'|b'a'..=b'f')){return Err("Save request must contain 32 lowercase hexadecimal characters.".into())}
 if storage(root)?.join(".hick-cache/resource-changes").join(&request).exists(){
  let retained=read_entry(root,&request)?;
  let fingerprint=intent.map(str::to_owned).unwrap_or_else(||super::workspace_fs::view::hash(&serde_json::to_vec(&(&relative,source)).unwrap()));
  if retained.capture.request_fingerprint!=fingerprint||retained.capture.workspace!=workspace||retained.capture.path!=relative||retained.after!=after||retained.evidence.as_ref().is_none_or(|value|value.outcome!="published"||!value.identity_proven||value.after_identity.as_deref()!=Some(before_identity.as_str()))||retained.after!=before{return Err("This save request already belongs to another body or retired source identity. It was not replayed.".into())}
  return Ok(())
 }
 let capture=Capture{kind:KIND.into(),path:relative.clone(),journal_path:journal_name(&relative),workspace:workspace.clone(),before_identity:before_identity.clone(),request_fingerprint:intent.map(str::to_owned).unwrap_or_else(||super::workspace_fs::view::hash(&serde_json::to_vec(&(&relative,source)).unwrap())),at:std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64};
 let context=serde_json::to_string(&capture).map_err(|e|e.to_string())?;
 let manifest=Manifest{version:2,workspace,context:context.clone(),title:"Save source".into(),request_id:Some(request.clone()),restore_of:None,reviewed_plan:ResourcePlan{version:1,context,title:"Save source".into(),annotations:vec![],patches:vec![ResourcePatch{path:journal_name(&relative),before:draft(&before,&before_identity)?,after:draft(&after,&before_identity)?}]},before:BTreeMap::from([(journal_name(&relative),before.clone())]),after:BTreeMap::from([(journal_name(&relative),after.clone())])};
 let batch=Batch::begin(&storage(root)?,&manifest).map_err(|error|format!("{error} Save request {request} was not retried; inspect its retained history."))?;
 binding.check(root)?;
 if observe(root,&relative)?!=before||(!missing&&stamp(path)?!=before_identity){return Err(format!("Source changed before publication. Request {request} remains prepared; no save was authorized."))}
 let result=writer();
 let observed=observe(root,&relative).and_then(|saved|{let identity=if matches!(saved,SavedSource::Missing){None}else{Some(stamp(path)?)};Ok((saved,identity))});
 let (outcome,identity,observed_after,error)=match (&result,observed){
  (Ok(proven),Ok((saved,identity))) if (proven.is_none()||proven.as_deref()==identity.as_deref())&&(saved==after || (missing&&text(&saved).ok()==Some(source)))=>("published",identity,Some(saved),None),
  (Err(error),Ok((saved,identity))) if saved==before&&(missing||identity.as_deref()==Some(before_identity.as_str()))=>("failed",identity,Some(saved),Some(error.clone())),
  (Err(error),_)=>("unknown",None,None,Some(error.clone())),
  _=>("unknown",None,None,Some("Writer returned without a confirmed exact saved source.".into())),
 };
 let receipt=Receipt{sequence:0,path:journal_name(&relative),state:match outcome{"published"=>"applied","failed"=>"refused",_=>"uncertain"}.into(),error:error.clone()};
 batch.receipt(&receipt)?;
 let evidence=Evidence{version:1,manifest_sha256:super::workspace_fs::view::hash(&serde_json::to_vec(&manifest).map_err(|e|e.to_string())?),outcome:outcome.into(),identity_proven:result.as_ref().is_ok_and(|identity|identity.is_some()),after_identity:identity,observed_mode:observed_after.as_ref().and_then(file_mode),error};
 batch.append_source_evidence(&serde_json::to_vec(&evidence).map_err(|e|e.to_string())?).map_err(|error|format!("{error} Save request {request} publication is not confirmed; preserve the draft."))?;
 if outcome!="published"{return Err(format!("Save request {request} is {outcome}. Preserve the draft and inspect Local history."))}
 binding.check(root)?;result.map(|_|())
}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRequest {pub version:u32,pub workspace:String,pub path:String,pub source:String,pub request_id:String,pub ownership:super::resource_inputs::Request,pub saved_guards:BTreeMap<String,String>,#[serde(default)]pub unaddressed_sources:BTreeMap<String,String>}
fn create_target(root:&Path,path:&str)->Result<std::path::PathBuf,String>{
 if Path::new(path).is_absolute(){let target=Path::new(path);if target.components().any(|part|matches!(part,std::path::Component::ParentDir|std::path::Component::CurDir)){return Err("Creation requires an exact source address without traversal.".into())}return Ok(target.to_path_buf())}
 super::resource_changes::validate_path(path)?;Ok(root.join(path))
}
fn ownership_guards(state:&LocalState,request:&CreateRequest)->Result<(),String>{
 if request.saved_guards.len()>4096||request.ownership.ownership_sources.len()>4096{return Err("Creation ownership capture exceeds its bounded source count.".into())}
 if request.unaddressed_sources.len()>4096{return Err("Unaddressed creation ownership exceeds its bounded source count.".into())}let mut cost=0usize;
 for source in request.unaddressed_sources.values(){if source.len()>8*1024*1024||source.len()>(32*1024*1024usize).saturating_sub(cost){return Err("Unaddressed creation ownership exceeds its bounded text capacity.".into())}cost+=source.len();if super::resource_inputs::contains_construction(source)?{return Err("Another retained construction draft has no current workspace source address. Assign its source, select its construction context or reconcile the deleted source before Save As. Its text is retained; no destination was created.".into())}
 }
 for(path,source)in &request.ownership.ownership_sources{
  if source.len()>8*1024*1024{return Err("Construction source exceeds its bounded text size.".into())}
  if !super::resource_inputs::contains_construction(source)?{continue}
  if Path::new(path).is_absolute(){return Err("Outside-file drafts are not workspace construction authority. Keep them readable and choose a workspace source context.".into())}
  let(full,saved)=super::resource_inputs::saved_target(state.index.root(),path)?;
  if matches!(saved,SavedSource::Missing){return Err(format!("Construction source has no saved physical baseline: {path}. Assign or save it before creating another namespace member."))}
  let guard=request.saved_guards.get(path).ok_or_else(||format!("Construction source has no captured physical guard: {path}"))?;
  let observed=read_for_api(state,&full,path)?;if observed.save_guard.as_deref()!=Some(guard){return Err(format!("Construction source changed physically after capture: {path}. Preserve drafts and prepare creation again."))}
  if source.len()>8*1024*1024{return Err("Construction source exceeds its bounded text size.".into())}
 }
 Ok(())
}
pub async fn create(State(state):State<LocalState>,Json(request):Json<CreateRequest>)->ApiResult<Json<Value>>{
 let writes=state.writes.clone();crate::engine::writes::during(&writes,async{
  if request.version!=1||request.source.len()>8*1024*1024||!super::resource_saves::hex(&request.request_id,32)||request.request_id.bytes().any(|b|b.is_ascii_uppercase()){return Err(ApiError::conflict("Source creation requires protocol1, a unique lowercase request identity and bounded text."))}
  let current_workspace=state.resource_root.check(state.index.root()).map_err(ApiError::conflict)?.to_string();
  let full=create_target(state.index.root(),&request.path).map_err(ApiError::conflict)?;
  let intent=super::workspace_fs::view::hash(&serde_json::to_vec(&("POST","/api/source-history/create",&request)).map_err(|e|ApiError::internal(e.to_string()))?);
  let replayed=replay(state.index.root(),&full,&request.source,Some(&request.request_id),Some(&intent)).map_err(ApiError::conflict)?;
  if !replayed{
   if request.workspace!=current_workspace{return Err(ApiError::conflict("Creation workspace changed. Retain the source and choose the destination again."))}
   ownership_guards(&state,&request).map_err(ApiError::conflict)?;
   if full.symlink_metadata().is_ok(){return Err(ApiError::conflict("Creation destination already exists. No existing bytes were overwritten."))}
   let id=state.index.id_for_path(&request.path);if state.rooms.get(&id).await.is_some(){return Err(ApiError::conflict("A retained live source owns this missing name. Reconcile it before creating another source there."))}
   let claims=super::api::generated_outputs(state.index.root(),&state.index);if super::api::generated_by(&claims,&request.path).is_some(){return Err(ApiError::conflict("A current generated output owns this destination."))}
   let relative=relative_name(state.index.root(),&full).ok();let mut capture=request.ownership.clone();capture.drafts.clear();capture.saved_guards=request.saved_guards.clone();
   capture.edit=if let Some(path)=&relative{let uri=crate::launch_profiles::inventory::workspace_source_uri(path).map_err(ApiError::conflict)?;json!({"documentChanges":[{"kind":"create","uri":uri},{"textDocument":{"uri":uri,"version":null},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":request.source}]}]})}else{json!({"changes":{}})};
   let mut fresh=super::resource_inputs::inspect(State(state.clone()),Json(capture.clone())).await?.0;
   if fresh.workspace!=request.workspace{return Err(ApiError::conflict("Creation workspace changed during ownership acquisition."))}
   if !fresh.refusals.is_empty(){
    if let Some(path)=&relative{
     if full.parent().is_some_and(|parent|!parent.exists()){
      let mut baseline=capture.clone();baseline.edit=json!({"changes":{}});let before=super::resource_inputs::inspect(State(state.clone()),Json(baseline.clone())).await?.0;
      let empty=super::resource_changes::prepare(super::resource_changes::Request{context:before.context.clone(),title:"Check source creation ownership".into(),edit:baseline.edit.clone(),sources:before.sources.clone()}).map_err(ApiError::conflict)?;
      let ownership=super::resource_candidates::analyze(state.index.root(),&before,&before,&baseline,&empty).map_err(ApiError::conflict)?;
      if ownership.saved.output_paths.contains(path)||ownership.draft.output_paths.contains(path){return Err(ApiError::conflict("Saved or draft construction owns the destination; no destination folder was created."))}
      let mut ancestor=full.parent().unwrap();while !ancestor.exists(){ancestor=ancestor.parent().ok_or_else(||ApiError::conflict("Creation destination has no existing parent"))?}
      let metadata=std::fs::symlink_metadata(ancestor).map_err(|e|ApiError::conflict(e.to_string()))?;if !metadata.is_dir()||metadata.file_type().is_symlink()||metadata.permissions().readonly(){return Err(ApiError::conflict("Creation parent is not an ordinary writable directory."))}
      fresh.refusals.remove(path);if let Some(SourceState::Missing{creatable})=fresh.sources.get_mut(path){*creatable=true}
     }
    }
   }
   if !fresh.refusals.is_empty(){return Err(ApiError::conflict("Saved or draft ownership refused this creation destination."))}
   let plan=super::resource_changes::prepare(super::resource_changes::Request{context:fresh.context.clone(),title:"Create source".into(),edit:capture.edit.clone(),sources:fresh.sources.clone()}).map_err(ApiError::conflict)?;
   let facts=super::resource_candidates::analyze(state.index.root(),&fresh,&fresh,&capture,&plan).map_err(ApiError::conflict)?;
   if !facts.protected_targets.is_empty(){return Err(ApiError::conflict("Saved or draft construction generates the chosen destination; edit its authored source instead."))}
   for(path,source)in &capture.ownership_sources{
    if super::resource_inputs::contains_construction(source).map_err(ApiError::conflict)?&&facts.draft.source_hashes.get(path)!=Some(&super::workspace_fs::view::hash(source.as_bytes())){return Err(ApiError::conflict("A retained construction source has no proven selected context. Choose its construction entry before creating another source."))}
   }
   ownership_guards(&state,&request).map_err(ApiError::conflict)?;
   super::source_tests::check_saved_sources(state.index.root(),&facts.checked_saved_sources).map_err(|e|ApiError::conflict(e.message().to_string()))?;
   if let Some(parent)=full.parent(){std::fs::create_dir_all(parent).map_err(|e|ApiError::internal(e.to_string()))?}
   write_observed(state.index.root(),&full,&request.source,Some(&request.request_id),Some(&intent),||{
    if state.resource_root.check(state.index.root())?!=request.workspace{return Err("Creation workspace changed before publication.".into())}
    ownership_guards(&state,&request)?;super::source_tests::check_saved_sources(state.index.root(),&facts.checked_saved_sources).map_err(|e|e.message().to_string())?;
    super::api::create_source_observed(&full,&request.source).map_err(|e|e.detail().to_string())
   }).map_err(ApiError::conflict)?;
  }
  let document=Path::new(&request.path).extension().and_then(|ext|ext.to_str()).is_some_and(|ext|ext.eq_ignore_ascii_case("md")||ext.eq_ignore_ascii_case("markdown"));
  let doc_id=document.then(||state.index.add(&request.path));
  if !document&&Path::new(&request.path).is_absolute(){state.opened_files.write().map_err(|_|ApiError::internal("Opened-file authority is unavailable"))?.insert(full.clone());}
  let guard=guard_for_result(&state,&full,&request.path,&request.source,&request.request_id,&intent).map_err(ApiError::conflict)?;
  Ok(Json(json!({"version":1,"history_protocol":1,"workspace":request.workspace,"namespace":state.index.root().canonicalize().map_err(|e|ApiError::internal(e.to_string()))?.to_string_lossy(),"path":request.path,"source":request.source,"source_hash":super::workspace_fs::view::hash(request.source.as_bytes()),"hash":super::plain_file::content_hash(request.source.as_bytes()),"request_id":request.request_id,"save_guard":guard,"doc_id":doc_id,"history":{"available":true,"outcome":"published","id":request.request_id}})))
 }).await
}
const LINEAGE_KIND:&str="ordinary-source-resource-lineage-v1";
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct LineageContext{kind:String,workspace:String,resource_id:String,manifest_sha256:String,before_identities:BTreeMap<String,String>,restored:BTreeMap<String,RestoredSource>}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct RestoredSource{identity:String,source:SavedSource}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct LineageEvidence{version:u32,manifest_sha256:String,private_manifest_sha256:String,after_identities:BTreeMap<String,String>}
pub(super) struct ResourceLineageCapture{batch:Batch,manifest:Manifest,context:LineageContext}
fn validate_physical_namespace(root:&Path,saved:&BTreeMap<String,SavedSource>,identities:&BTreeMap<String,String>)->Result<(),String>{
 let files:BTreeSet<_>=saved.iter().filter_map(|(path,state)|matches!(state,SavedSource::File{..}).then_some(path)).collect();
 if files!=identities.keys().collect(){return Err("Source lineage requires exactly one observed physical identity for every saved file.".into())}
 for(path,expected)in saved{let actual=observe(root,path)?;let identity=if matches!(actual,SavedSource::File{..}){Some(stamp(&root.join(path))?)}else{None};if &actual!=expected||matches!(actual,SavedSource::File{..})&&identities.get(path)!=identity.as_ref(){return Err(format!("Source lineage physical observation changed at {path}. No old history authority was attached."))}}
 Ok(())
}
/// A retained inverse may consume only the original producer's current result.
/// Fresh equal bytes or a freshly loaded guard do not prove that result's lineage.
pub(super) fn validate_resource_after(root:&Path,recovery:&super::resource_journal::Recovery)->Result<(),String>{
 if !recovery.all_paths_recorded_applied{return Err("The retained resource publication is incomplete; its inverse has no confirmed source authority.".into())}
 let store=storage(root)?;let workspace=workspace(root)?;let mut cursor=None;let mut found=None;let mut cost=0usize;
 for _ in 0..20{let page=super::resource_journal::list_recoveries(&store,cursor.as_deref())?;
  for id in page.ids{let record=super::resource_journal::read_recovery(&store,&id)?;
   let Ok(context)=serde_json::from_str::<LineageContext>(&record.manifest.context)else{continue};
   if context.kind!=LINEAGE_KIND||context.resource_id!=recovery.id{continue}
   cost=cost.saturating_add(serde_json::to_vec(&record.manifest).map_err(|e|e.to_string())?.len());if cost>32*1024*1024{return Err("Inverse source proof exceeds its bounded reading.".into())}
   if context.workspace!=workspace||context.manifest_sha256!=recovery.manifest_sha256||record.manifest.before!=recovery.manifest.before||record.manifest.after!=recovery.manifest.after||record.unrecognized_files.as_slice()!=["source-lineage.json"]{return Err("The retained original resource proof is foreign, prepared or changed. Its text remains readable.".into())}
   let bytes=super::resource_journal::read_regular(&store.join(".hick-cache/resource-changes").join(&id).join("source-lineage.json"),2*1024*1024)?;
   let evidence:LineageEvidence=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
   if evidence.version!=1||evidence.private_manifest_sha256!=record.manifest_sha256||evidence.manifest_sha256!=recovery.manifest_sha256||found.is_some(){return Err("Original resource source proof is corrupt or ambiguous.".into())}
   found=Some(evidence.after_identities);
  }cursor=page.next;if cursor.is_none(){break}
 }
 if cursor.is_some(){return Err("Original resource proof was not uniquely resolved within the bounded reading.".into())}
 let identities=found.ok_or("This historical resource has no retained producer source proof. Its inverse is unavailable; inspect its text without applying it.")?;
 validate_physical_namespace(root,&recovery.manifest.after,&identities)
}

pub(super) fn prepare_resource_lineage(root:&Path,resource_id:&str,manifest:&Manifest,before_identities:&BTreeMap<String,String>)->Result<ResourceLineageCapture,String>{
 let original=super::resource_journal::read_recovery(root,resource_id)?;
 let bytes=serde_json::to_vec(manifest).map_err(|e|e.to_string())?;
 if super::workspace_fs::view::hash(&bytes)!=original.manifest_sha256{return Err("Source lineage does not name the retained resource manifest.".into())}
 validate_physical_namespace(root,&manifest.before,before_identities)?;
 let mut restored=BTreeMap::new();
 if let Some(origin)=&manifest.restore_of{
  let prior=super::resource_journal::read_recovery(root,&origin.recovery_id)?;
  validate_resource_after(root,&prior)?;
  if prior.fingerprint!=origin.fingerprint||!prior.all_paths_recorded_applied{return Err("Restored history requires the exact original complete resource receipt.".into())}
  let mut cursor=None;let mut found=None;
  for _ in 0..20{let page=super::resource_journal::list_recoveries(&storage(root)?,cursor.as_deref())?;for id in page.ids{
   let Ok(record)=super::resource_journal::read_recovery(&storage(root)?,&id)else{continue};let Ok(context)=serde_json::from_str::<LineageContext>(&record.manifest.context)else{continue};
   if context.kind!=LINEAGE_KIND||context.workspace!=workspace(root)?||context.resource_id!=origin.recovery_id{continue}
   let bytes=super::resource_journal::read_regular(&storage(root)?.join(".hick-cache/resource-changes").join(&id).join("source-lineage.json"),2*1024*1024)?;let evidence:LineageEvidence=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
   if evidence.version!=1||evidence.private_manifest_sha256!=record.manifest_sha256||context.manifest_sha256!=prior.manifest_sha256||evidence.manifest_sha256!=prior.manifest_sha256{return Err("Original source lineage is corrupt; its text remains readable without restored authority.".into())}
   if found.is_some(){return Err("Original source lineage is ambiguous.".into())}found=Some(context);
  }cursor=page.next;if cursor.is_none(){break}}
  if let Some(context)=found{for(path,source)in &prior.manifest.before{if let Some(identity)=context.before_identities.get(path){if manifest.after.get(path)==Some(source){restored.insert(path.clone(),RestoredSource{identity:identity.clone(),source:source.clone()});}}}}
 }
 let context=LineageContext{kind:LINEAGE_KIND.into(),workspace:workspace(root)?,resource_id:resource_id.into(),manifest_sha256:original.manifest_sha256,before_identities:before_identities.clone(),restored};
 let mut private=manifest.clone();private.workspace=context.workspace.clone();private.context=serde_json::to_string(&context).map_err(|e|e.to_string())?;private.reviewed_plan.context=private.context.clone();private.request_id=None;
 let batch=Batch::begin(&storage(root)?,&private)?;
 Ok(ResourceLineageCapture{batch,manifest:manifest.clone(),context})
}
pub(super) fn finish_resource_lineage(root:&Path,manifest:&Manifest,capture:ResourceLineageCapture,after_identities:&BTreeMap<String,String>)->Result<(),String>{
 if workspace(root)?!=capture.context.workspace||serde_json::to_vec(manifest).map_err(|e|e.to_string())?!=serde_json::to_vec(&capture.manifest).map_err(|e|e.to_string())?{return Err("Source lineage workspace or reviewed publication changed.".into())}
 let original=super::resource_journal::read_recovery(root,&capture.context.resource_id)?;
 if original.manifest_sha256!=capture.context.manifest_sha256||!original.all_paths_recorded_applied{return Err("Source lineage requires the complete confirmed resource publication; partial or unknown outcomes remain unbound.".into())}
 validate_physical_namespace(root,&manifest.after,after_identities)?;
 let private=super::resource_journal::read_recovery(&storage(root)?,&capture.batch.id)?;
 let evidence=LineageEvidence{version:1,private_manifest_sha256:private.manifest_sha256,manifest_sha256:capture.context.manifest_sha256,after_identities:after_identities.clone()};
 capture.batch.append_resource_lineage(&serde_json::to_vec(&evidence).map_err(|e|e.to_string())?)
}
struct HistoryEdge{before_identity:String,before:SavedSource,after_identity:String,after:SavedSource}
fn push_history_edge(out:&mut Vec<HistoryEdge>,cost:&mut usize,before_identity:&str,before:&SavedSource,after_identity:&str,after:&SavedSource)->Result<(),String>{
 let bytes=128usize.saturating_add(before_identity.len()).saturating_add(after_identity.len()).saturating_add(match before{SavedSource::File{text,..}=>text.len(),_=>0}).saturating_add(match after{SavedSource::File{text,..}=>text.len(),_=>0});
 if bytes> (32*1024*1024usize).saturating_sub(*cost){return Err("Source-history connections exceed the bounded 32MiB reading. No partial authority was inferred.".into())}*cost+=bytes;
 out.push(HistoryEdge{before_identity:before_identity.into(),before:before.clone(),after_identity:after_identity.into(),after:after.clone()});Ok(())
}
fn resource_history_edges(root:&Path,workspace_id:&str)->Result<(Vec<HistoryEdge>,Vec<String>),String>{
 let mut cursor=None;let mut edges=Vec::new();let mut notices=Vec::new();let mut edge_cost=0usize;let mut scan_cost=0usize;
 for _ in 0..20{let page=super::resource_journal::list_recoveries(&storage(root)?,cursor.as_deref())?;
  for id in page.ids{let record=match super::resource_journal::read_recovery(&storage(root)?,&id){Ok(record)=>record,Err(error)=>{notices.push(format!("Unreadable resource-history connection {id}: {error}"));continue}};
   let bytes=record.manifest.before.values().chain(record.manifest.after.values()).fold(0usize,|sum,saved|sum.saturating_add(64).saturating_add(match saved{SavedSource::File{text,..}=>text.len(),_=>0}));if bytes>(32*1024*1024usize).saturating_sub(scan_cost){return Err("Retained history scan exceeds its bounded 32MiB reading. No absent history was inferred.".into())}scan_cost+=bytes;
   let Ok(context)=serde_json::from_str::<LineageContext>(&record.manifest.context)else{continue};if context.kind!=LINEAGE_KIND||context.workspace!=workspace_id{continue}
   let read=(||->Result<Vec<HistoryEdge>,String>{
    if record.unrecognized_files.as_slice()!=["source-lineage.json"]{return Err("Resource lineage is prepared or contains unrecognized evidence.".into())}
    let bytes=super::resource_journal::read_regular(&storage(root)?.join(".hick-cache/resource-changes").join(&id).join("source-lineage.json"),2*1024*1024)?;
    let evidence:LineageEvidence=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
    let original=super::resource_journal::read_recovery(root,&context.resource_id)?;
    if evidence.version!=1||evidence.private_manifest_sha256!=record.manifest_sha256||evidence.manifest_sha256!=context.manifest_sha256||original.manifest_sha256!=context.manifest_sha256||!original.all_paths_recorded_applied||original.manifest.before!=record.manifest.before||original.manifest.after!=record.manifest.after{return Err("Resource lineage evidence no longer confirms its retained publication.".into())}
    let mut result=Vec::new();
    for after_patch in &original.manifest.reviewed_plan.patches{if let SourceState::File{identity:logical,..}=&after_patch.after{
     let before_patch=original.manifest.reviewed_plan.patches.iter().find(|patch|matches!(&patch.before,SourceState::File{identity,..} if identity==logical));
     if let Some(before_patch)=before_patch{if let(Some(before),Some(after))=(context.before_identities.get(&before_patch.path),evidence.after_identities.get(&after_patch.path)){push_history_edge(&mut result,&mut edge_cost,before,&original.manifest.before[&before_patch.path],after,&original.manifest.after[&after_patch.path])?}}
    }}
    for(path,restored)in &context.restored{if original.manifest.after.get(path)==Some(&restored.source){if let Some(after)=evidence.after_identities.get(path){push_history_edge(&mut result,&mut edge_cost,&restored.identity,&restored.source,after,&restored.source)?}}}
    Ok(result)
   })();match read{Ok(found)=>edges.extend(found),Err(error)if error.starts_with("Source-history connections exceed")=>return Err(error),Err(error)=>notices.push(format!("Unconfirmed resource-history connection {id}: {error}"))}
  }cursor=page.next;if cursor.is_none(){break}
 }if cursor.is_some(){notices.push("Additional resource-history connections exceed this bounded reading; no missing authority was inferred.".into())}Ok((edges,notices))
}
struct Retained {id:String,namespace_current:bool,capture:Capture,before:SavedSource,after:SavedSource,evidence:Option<Evidence>,prepared:bool,fingerprint:String}
fn read_entry(root:&Path,id:&str)->Result<Retained,String>{
 let recovery=super::resource_journal::read_recovery(&storage(root)?,id)?;
 let capture:Capture=serde_json::from_str(&recovery.manifest.context).map_err(|_|"Not an ordinary-source save record.")?;
 if capture.kind!=KIND||capture.workspace!=recovery.manifest.workspace||recovery.manifest.request_id.as_deref()!=Some(id)||recovery.manifest.before.len()!=1||!recovery.manifest.before.contains_key(&capture.journal_path)||recovery.unrecognized_files.iter().any(|name|name!="source-save.json"){return Err("Source history scope or retained files are invalid.".into())}
 let evidence=if recovery.unrecognized_files.iter().any(|name|name=="source-save.json"){
  let bytes=super::resource_journal::read_regular(&storage(root)?.join(".hick-cache/resource-changes").join(id).join("source-save.json"),64*1024)?;
  let value:Evidence=serde_json::from_slice(&bytes).map_err(|_|"Source history outcome is corrupt.")?;
  if value.outcome=="published"{let intended=&recovery.manifest.after[&capture.journal_path];let observed=observed_source(intended,&value);if value.observed_mode.is_none()||(!matches!(recovery.manifest.before[&capture.journal_path],SavedSource::Missing)&&&observed!=intended){return Err("Observed source publication permissions differ from its prepared source.".into())}}
  let expected=match value.outcome.as_str(){"published"=>"applied","failed"=>"refused","unknown"=>"uncertain",_=>return Err("Unknown source history outcome.".into())};
  if value.version!=1||value.manifest_sha256!=recovery.manifest_sha256||recovery.receipts.len()!=1||recovery.receipts[0].state!=expected||recovery.receipts[0].path!=capture.journal_path||(value.outcome=="published"&&value.after_identity.is_none()){return Err("Source history outcome does not confirm this exact manifest.".into())}Some(value)
 }else{None};
 let fingerprint=super::workspace_fs::view::hash(&serde_json::to_vec(&(recovery.manifest_sha256,&evidence)).map_err(|e|e.to_string())?);
 Ok(Retained{id:id.into(),namespace_current:false,before:recovery.manifest.before[&capture.journal_path].clone(),after:evidence.as_ref().filter(|e|e.outcome=="published").map(|e|observed_source(&recovery.manifest.after[&capture.journal_path],e)).unwrap_or_else(||recovery.manifest.after[&capture.journal_path].clone()),prepared:recovery.receipts.is_empty(),capture,evidence,fingerprint})
}
#[derive(Serialize)]
pub struct Entry {id:String,at:u64,outcome:String,fingerprint:String,before:SavedSource,after:SavedSource,diff:String,available:bool,retry_allowed:bool,reason:Option<String>}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadRequest {pub path:String}
#[derive(Serialize)]
pub struct Reading {version:u32,history_protocol:u32,workspace:String,namespace:String,path:String,current:SavedSource,current_identity:String,entries:Vec<Entry>,notices:Vec<String>}
fn retained(root:&Path,_path:&str,workspace:&str)->Result<(Vec<Retained>,Vec<String>),String>{
 let mut cursor=None;let mut entries=Vec::new();let mut notices=Vec::new();let mut bytes=0;
 for _ in 0..20 {
  let page=super::resource_journal::list_recoveries(&storage(root)?,cursor.as_deref())?;
  for id in page.ids {
   match read_entry(root,&id){
    Ok(mut entry)=>{entry.namespace_current=entry.capture.workspace==workspace;bytes+=match &entry.before{SavedSource::File{text,..}=>text.len(),_=>0}+text(&entry.after)?.len();if bytes>32*1024*1024{return Err("Local history exceeds this bounded reading. Narrow retained records before reading it.".into())}entries.push(entry)},
    Err(error) if error=="Not an ordinary-source save record."=>{},Err(error)=>notices.push(format!("Unreadable retained record {id}: {error}")),
   }
  }
  cursor=page.next;if cursor.is_none(){break}
 }
 if cursor.is_some(){notices.push("More retained resource records exist beyond this bounded history reading; no absent history was inferred.".into())}
 let historical=entries.iter().filter(|entry|!entry.namespace_current).count();if historical>0{notices.push(format!("{historical} retained saves belong to an earlier or foreign physical workspace identity. Their text remains historical and readable; they grant no restore or retry authority."))}
 entries.sort_by_key(|entry|entry.capture.at);Ok((entries,notices))
}
fn lineage(entries:&[Retained],current:&SavedSource,identity:&str,edges:&[HistoryEdge])->BTreeSet<String>{
 let mut identities=BTreeSet::from([identity.to_string()]);let mut states=BTreeMap::from([(identity.to_string(),current.clone())]);let mut accepted=BTreeSet::new();
 for _ in 0..=entries.len()+edges.len(){let mut changed=false;
  for edge in edges{if states.get(&edge.after_identity)==Some(&edge.after)&&!identities.contains(&edge.before_identity){identities.insert(edge.before_identity.clone());states.insert(edge.before_identity.clone(),edge.before.clone());changed=true}}
  for entry in entries.iter().rev(){if !entry.namespace_current||accepted.contains(&entry.id){continue}if let Some(evidence)=&entry.evidence{if evidence.outcome=="published"&&evidence.identity_proven{if let Some(after)=&evidence.after_identity{if identities.contains(after)&&states.get(after)==Some(&entry.after){accepted.insert(entry.id.clone());identities.insert(entry.capture.before_identity.clone());states.insert(entry.capture.before_identity.clone(),entry.before.clone());changed=true}}}}}
  if !changed{break}
 }accepted
}
fn source_diff(before:&str,after:&str)->String{
 if before==after{return "No source byte changes. Permission metadata is retained separately.\n".into()}
 let old:Vec<_>=before.split_inclusive('\n').collect();let new:Vec<_>=after.split_inclusive('\n').collect();let mut prefix=0;while prefix<old.len().min(new.len())&&old[prefix]==new[prefix]{prefix+=1}
 let mut suffix=0;while suffix<old.len().min(new.len())-prefix&&old[old.len()-1-suffix]==new[new.len()-1-suffix]{suffix+=1}
 let start=prefix.saturating_sub(3);let tail=suffix.min(3);let mut output=format!("--- before save\n+++ intended saved after\n@@ -{},{} +{},{} @@\n",if old.is_empty(){0}else{start+1},old.len()-suffix+tail-start,if new.is_empty(){0}else{start+1},new.len()-suffix+tail-start);
 for(mark,lines)in [(' ',&old[start..prefix]),('-',&old[prefix..old.len()-suffix]),('+',&new[prefix..new.len()-suffix]),(' ',&old[old.len()-suffix..old.len()-suffix+tail])]{for line in lines{
  if output.len()+line.len()+40>64*1024{output.push_str("[Diff truncated at64KiB; complete before/after text remains readable.]\n");return output}
  output.push(mark);output.push_str(line);if !line.ends_with('\n'){output.push_str("\n\\ No newline at end of file\n")}
 }}output
}
fn reading(state:&LocalState,path:&str)->Result<Reading,String>{
 let root=state.index.root();state.resource_root.check(root)?;let workspace=workspace(root)?;let full=authorized_address(state,path)?;let current=observe(root,path)?;let identity=stamp(&full)?;
 let address=source_address(root,&full)?;let (entries,mut notices)=retained(root,&address,&workspace)?;let(edges,mut connection_notices)=resource_history_edges(root,&workspace)?;notices.append(&mut connection_notices);let available=lineage(&entries,&current,&identity,&edges);
 state.resource_root.check(root)?;
 if stamp(&full)?!=identity||observe(root,path)?!=current{return Err("Source changed while reading local history. Read it again.".into())}
 let unavailable=writable_source(state,&full,path).err();
 let listed=entries.into_iter().filter(|entry|entry.capture.path==address||available.contains(&entry.id)).rev().map(|entry|{let admitted=available.contains(&entry.id)&&unavailable.is_none();let retry_allowed=entry.namespace_current&&unavailable.is_none()&&entry.before==current&&entry.capture.before_identity==identity&&entry.evidence.as_ref().is_some_and(|value|value.outcome=="failed");let diff=source_diff(match &entry.before{SavedSource::File{text,..}=>text,_=>""},text(&entry.after).unwrap_or(""));
  Entry{id:entry.id,at:entry.capture.at,outcome:entry.evidence.map(|e|e.outcome).unwrap_or_else(||(if entry.prepared{"prepared"}else{"unknown"}).into()),fingerprint:entry.fingerprint,before:entry.before,after:entry.after,diff,available:admitted,retry_allowed,reason:(!entry.namespace_current).then(||"Historical workspace identity is unconfirmed; text remains readable, but cannot replace this source or release its save attempt.".into()).or_else(||unavailable.clone()).or_else(||(!admitted).then(||"This record has no confirmed connection to the current physical source. Its text remains readable; prepared intentions do not confirm publication.".into()))}
 }).collect();
 Ok(Reading{version:1,history_protocol:1,namespace:root.canonicalize().map_err(|e|e.to_string())?.to_str().ok_or("Workspace address is not UTF-8.")?.into(),workspace,path:path.into(),current,current_identity:identity,entries:listed,notices})
}
pub async fn list(State(state):State<LocalState>,Query(request):Query<ReadRequest>)->ApiResult<Json<Reading>>{
 tokio::task::spawn_blocking(move||reading(&state,&request.path)).await.map_err(|e|ApiError::internal(e.to_string()))?.map(Json).map_err(ApiError::conflict)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestoreRequest {pub version:u32,pub path:String,pub id:String,pub fingerprint:String,pub workspace:String,pub namespace:String,pub current_identity:String,pub current_hash:String,pub draft_hash:String,pub side:String}
pub async fn restore_draft(State(state):State<LocalState>,Json(request):Json<RestoreRequest>)->ApiResult<Json<Value>>{
 tokio::task::spawn_blocking(move||->Result<Value,String>{
  if request.version!=1||!matches!(request.side.as_str(),"before"|"after")||request.draft_hash.len()!=64{return Err("Choose one exact historical source side and captured draft.".into())}
  let current=reading(&state,&request.path)?;
  if current.namespace!=request.namespace||current.workspace!=request.workspace||current.current_identity!=request.current_identity||super::workspace_fs::view::hash(text(&current.current)?.as_bytes())!=request.current_hash{return Err("Saved source identity or revision changed after history review.".into())}
  if let SavedSource::File{readonly:true,..}=current.current{return Err("The current source is read-only; its history remains readable.".into())}
  let claims=super::api::generated_outputs(state.index.root(),&state.index);
  let full=authorized_address(&state,&request.path)?;
  if super::api::generated_by(&claims,&request.path).is_some()||claims.keys().any(|path|same_file::is_same_file(&full,state.index.root().join(path)).unwrap_or(false)){return Err("Generated source history is readable but cannot replace an authored draft.".into())}
  let entry=current.entries.into_iter().find(|entry|entry.id==request.id&&entry.fingerprint==request.fingerprint&&entry.available).ok_or("History evidence changed or this source identity is retired.")?;
  let source=text(if request.side=="before"{&entry.before}else{&entry.after})?.to_string();
  Ok(json!({"version":1,"history_protocol":1,"workspace":request.workspace,"namespace":request.namespace,"path":request.path,"id":request.id,"fingerprint":request.fingerprint,"current_identity":request.current_identity,"current_hash":request.current_hash,"draft_hash":request.draft_hash,"side":request.side,"source":source,"consequences":"unknown","notice":"Restored text is a source draft. Compiler and generated-output consequences are unknown until separately checked; nothing was published."}))
 }).await.map_err(|e|ApiError::internal(e.to_string()))?.map(Json).map_err(ApiError::conflict)
}
#[cfg(test)]
mod tests {
 use super::*;
 fn save(root:&Path,id:&str,source:&str)->Result<(),String>{let path=root.join("a.txt");write_observed(root,&path,source,Some(id),Some(&super::super::workspace_fs::view::hash(source.as_bytes())),||super::super::store::write_atomic_observed(&path,source.as_bytes()).map_err(|e|e.to_string()))}
 #[test]fn resource_move_connects_exact_logical_source_and_rejects_recreated_names(){
  let root=tempfile::tempdir().unwrap();let a=root.path().join("a.txt");let b=root.path().join("b.txt");std::fs::write(&a,"old").unwrap();let save_id="11111111111111111111111111111111";save(root.path(),save_id,"new").unwrap();
  let saved=observe(root.path(),"a.txt").unwrap();let source=SourceState::File{identity:"original-buffer".into(),text:"new".into(),writable:true,removable:true,version:None};let missing=SourceState::Missing{creatable:true};
  let plan=ResourcePlan{version:1,context:"move".into(),title:"Move".into(),annotations:vec![],patches:vec![ResourcePatch{path:"a.txt".into(),before:source.clone(),after:missing.clone()},ResourcePatch{path:"b.txt".into(),before:missing,after:source}]};
  let manifest=Manifest{version:2,workspace:"runtime".into(),context:"move".into(),title:"Move".into(),request_id:None,restore_of:None,reviewed_plan:plan,before:BTreeMap::from([("a.txt".into(),saved.clone()),("b.txt".into(),SavedSource::Missing)]),after:BTreeMap::from([("a.txt".into(),SavedSource::Missing),("b.txt".into(),saved.clone())])};
  let batch=Batch::begin(root.path(),&manifest).unwrap();let before=BTreeMap::from([("a.txt".into(),stamp(&a).unwrap())]);let capture=prepare_resource_lineage(root.path(),&batch.id,&manifest,&before).unwrap();
  let after=super::super::api::create_source_observed(&b,"new").unwrap();std::fs::remove_file(&a).unwrap();
  assert!(resource_history_edges(root.path(),&workspace(root.path()).unwrap()).unwrap().0.is_empty());
  for(sequence,path)in ["a.txt","b.txt"].into_iter().enumerate(){batch.receipt(&Receipt{sequence,path:path.into(),state:"applied".into(),error:None}).unwrap()}
  finish_resource_lineage(root.path(),&manifest,capture,&BTreeMap::from([("b.txt".into(),after)])).unwrap();
  let original=super::super::resource_journal::read_recovery(root.path(),&batch.id).unwrap();validate_resource_after(root.path(),&original).unwrap();
  let workspace=workspace(root.path()).unwrap();let(entries,_)=retained(root.path(),"b.txt",&workspace).unwrap();let(edges,notices)=resource_history_edges(root.path(),&workspace).unwrap();assert!(notices.is_empty(),"{notices:?}");assert!(lineage(&entries,&saved,&stamp(&b).unwrap(),&edges).contains(save_id));
  std::fs::remove_file(&b).unwrap();std::fs::write(&b,"new").unwrap();assert!(lineage(&entries,&saved,&stamp(&b).unwrap(),&edges).is_empty());assert!(validate_resource_after(root.path(),&original).is_err());
 }
 #[test]fn guarded_delete_restoration_recovers_only_the_original_retained_lineage(){
  let root=tempfile::tempdir().unwrap();let path=root.path().join("a.txt");std::fs::write(&path,"old").unwrap();let save_id="11111111111111111111111111111111";save(root.path(),save_id,"new").unwrap();let saved=observe(root.path(),"a.txt").unwrap();
  let source=SourceState::File{identity:"original-buffer".into(),text:"new".into(),writable:true,removable:true,version:None};let missing=SourceState::Missing{creatable:true};
  let make=|context:&str,before:SourceState,after:SourceState,saved_before:SavedSource,saved_after:SavedSource,restore_of:Option<super::super::resource_journal::RestorationOrigin>|Manifest{version:if restore_of.is_some(){3}else{2},workspace:"runtime".into(),context:context.into(),title:context.into(),request_id:None,restore_of,reviewed_plan:ResourcePlan{version:1,context:context.into(),title:context.into(),annotations:vec![],patches:vec![ResourcePatch{path:"a.txt".into(),before,after}]},before:BTreeMap::from([("a.txt".into(),saved_before)]),after:BTreeMap::from([("a.txt".into(),saved_after)])};
  let deleted=make("delete",source.clone(),missing.clone(),saved.clone(),SavedSource::Missing,None);let batch=Batch::begin(root.path(),&deleted).unwrap();let capture=prepare_resource_lineage(root.path(),&batch.id,&deleted,&BTreeMap::from([("a.txt".into(),stamp(&path).unwrap())])).unwrap();std::fs::remove_file(&path).unwrap();batch.receipt(&Receipt{sequence:0,path:"a.txt".into(),state:"applied".into(),error:None}).unwrap();finish_resource_lineage(root.path(),&deleted,capture,&BTreeMap::new()).unwrap();
  let original=super::super::resource_journal::read_recovery(root.path(),&batch.id).unwrap();let mut restored_source=source;if let SourceState::File{identity,..}=&mut restored_source{*identity="restored-identity".into()}
  let restored=make("restore",missing,restored_source,SavedSource::Missing,saved.clone(),Some(super::super::resource_journal::RestorationOrigin{recovery_id:batch.id,fingerprint:original.fingerprint}));let batch=Batch::begin(root.path(),&restored).unwrap();let capture=prepare_resource_lineage(root.path(),&batch.id,&restored,&BTreeMap::new()).unwrap();let after=super::super::api::create_source_observed(&path,"new").unwrap();batch.receipt(&Receipt{sequence:0,path:"a.txt".into(),state:"applied".into(),error:None}).unwrap();finish_resource_lineage(root.path(),&restored,capture,&BTreeMap::from([("a.txt".into(),after)])).unwrap();
  let workspace=workspace(root.path()).unwrap();let(entries,_)=retained(root.path(),"a.txt",&workspace).unwrap();let(edges,notices)=resource_history_edges(root.path(),&workspace).unwrap();assert!(notices.is_empty(),"{notices:?}");assert!(lineage(&entries,&saved,&stamp(&path).unwrap(),&edges).contains(save_id));
  std::fs::remove_file(&path).unwrap();std::fs::write(&path,"new").unwrap();assert!(lineage(&entries,&saved,&stamp(&path).unwrap(),&edges).is_empty());
 }
 #[test]fn confirmed_saves_form_a_chain_and_exact_retries_never_reexecute(){
  let root=tempfile::tempdir().unwrap();let path=root.path().join("a.txt");std::fs::write(&path,"old").unwrap();
  let first="11111111111111111111111111111111";let second="22222222222222222222222222222222";
  save(root.path(),first,"new").unwrap();
  write(root.path(),&path,"new",Some(first),Some(&super::super::workspace_fs::view::hash(b"new")),||panic!("a retry must not execute")).unwrap();
  assert!(write(root.path(),&path,"new",Some(first),Some(&super::super::workspace_fs::view::hash(b"changed base or transport")),||panic!("changed canonical request must not execute")).is_err());
  let first_runtime=RootIdentity::bind(root.path());let second_runtime=RootIdentity::bind(root.path());assert_ne!(first_runtime.check(root.path()).unwrap(),second_runtime.check(root.path()).unwrap());
  assert!(replay(root.path(),&path,"new",Some(first),Some(&super::super::workspace_fs::view::hash(b"new"))).unwrap());
  assert!(write(root.path(),&path,"other",Some(first),Some(&super::super::workspace_fs::view::hash(b"new")),||panic!("changed retry must not execute")).is_err());
  save(root.path(),second,"newer").unwrap();let workspace=workspace(root.path()).unwrap();
  let(entries,_)=retained(root.path(),"a.txt",&workspace).unwrap();let(_,current)=super::super::resource_inputs::saved_target(root.path(),"a.txt").unwrap();assert_eq!(lineage(&entries,&current,&stamp(&path).unwrap(),&[]).len(),2);
  std::fs::remove_file(&path).unwrap();std::fs::write(&path,"newer").unwrap();assert!(lineage(&entries,&current,&stamp(&path).unwrap(),&[]).is_empty());
  assert!(write(root.path(),&path,"newer",Some(second),Some(&super::super::workspace_fs::view::hash(b"newer")),||panic!("recreated source must not execute")).is_err());
 }
 #[test]fn earlier_workspace_identity_keeps_retained_text_without_restore_or_retry_authority(){
  let root=tempfile::tempdir().unwrap();let path=root.path().join("a.txt");std::fs::write(&path,"old").unwrap();let current_id="11111111111111111111111111111111";save(root.path(),current_id,"new").unwrap();
  let old_id="88888888888888888888888888888888";let store=storage(root.path()).unwrap();let original=super::super::resource_journal::read_recovery(&store,current_id).unwrap();let old=read_entry(root.path(),current_id).unwrap();let mut manifest=original.manifest;let mut capture=old.capture;capture.workspace=super::super::workspace_fs::view::hash(b"legacy path-only workspace identity");manifest.workspace=capture.workspace.clone();manifest.context=serde_json::to_string(&capture).unwrap();manifest.reviewed_plan.context=manifest.context.clone();manifest.request_id=Some(old_id.into());
  let batch=Batch::begin(&store,&manifest).unwrap();batch.receipt(&Receipt{sequence:0,path:capture.journal_path.clone(),state:"applied".into(),error:None}).unwrap();let mut evidence=old.evidence.unwrap();evidence.manifest_sha256=super::super::resource_journal::read_recovery(&store,old_id).unwrap().manifest_sha256;batch.append_source_evidence(&serde_json::to_vec(&evidence).unwrap()).unwrap();
  let(entries,notices)=retained(root.path(),"a.txt",&workspace(root.path()).unwrap()).unwrap();let legacy=entries.iter().find(|entry|entry.id==old_id).unwrap();assert!(!legacy.namespace_current);assert_eq!(text(&legacy.before).unwrap(),"old");assert_eq!(text(&legacy.after).unwrap(),"new");assert!(notices.iter().any(|notice|notice.contains("historical and readable")));assert!(!lineage(&entries,&observe(root.path(),"a.txt").unwrap(),&stamp(&path).unwrap(),&[]).contains(old_id));assert!(replay(root.path(),&path,"new",Some(old_id),Some(&capture.request_fingerprint)).is_err());
 }
 #[test]fn failed_and_unknown_writers_are_readable_without_published_authority(){
  let root=tempfile::tempdir().unwrap();let path=root.path().join("a.txt");std::fs::write(&path,"old").unwrap();
  let failed="33333333333333333333333333333333";assert!(write(root.path(),&path,"new",Some(failed),Some(&super::super::workspace_fs::view::hash(b"new")),||Err("refused before effect".into())).is_err());assert_eq!(read_entry(root.path(),failed).unwrap().evidence.unwrap().outcome,"failed");
  let unknown="44444444444444444444444444444444";assert!(write(root.path(),&path,"new",Some(unknown),Some(&super::super::workspace_fs::view::hash(b"new")),||{std::fs::write(&path,"partial").unwrap();Err("publication stopped".into())}).is_err());assert_eq!(read_entry(root.path(),unknown).unwrap().evidence.unwrap().outcome,"unknown");assert_eq!(std::fs::read_to_string(path).unwrap(),"partial");
 }
 #[test]fn altered_manifest_and_outcome_evidence_refuse_reading(){
  let root=tempfile::tempdir().unwrap();std::fs::write(root.path().join("a.txt"),"old").unwrap();let id="55555555555555555555555555555555";save(root.path(),id,"new").unwrap();
  let path=storage(root.path()).unwrap().join(".hick-cache/resource-changes").join(id).join("manifest.json");let mut bytes=std::fs::read(&path).unwrap();bytes.push(b' ');std::fs::write(path,bytes).unwrap();assert!(read_entry(root.path(),id).is_err());
 }
 #[test]fn authorized_creation_records_missing_before_and_observed_mode_without_clobber(){
  let root=tempfile::tempdir().unwrap();let path=root.path().join("new.md");let id="77777777777777777777777777777777";let intent=super::super::workspace_fs::view::hash(b"create full source");
  write_observed(root.path(),&path,"new source",Some(id),Some(&intent),||super::super::api::create_source_observed(&path,"new source").map_err(|e|e.detail().to_string())).unwrap();
  let retained=read_entry(root.path(),id).unwrap();assert!(matches!(retained.before,SavedSource::Missing));assert_eq!(text(&retained.after).unwrap(),"new source");assert!(retained.evidence.unwrap().identity_proven);
  assert!(replay(root.path(),&path,"new source",Some(id),Some(&intent)).unwrap());
  assert!(super::super::api::create_source_observed(&path,"other").is_err());assert_eq!(std::fs::read_to_string(path).unwrap(),"new source");assert!(!root.path().join(".hick-cache").exists());
 }
 #[cfg(unix)]
 #[test]fn declared_root_alias_accepts_only_its_verified_canonical_source_names(){
  let owner=tempfile::tempdir().unwrap();let root=owner.path().join("project");std::fs::create_dir(&root).unwrap();let alias=owner.path().join("alias");std::os::unix::fs::symlink(&root,&alias).unwrap();
  let source=root.join("a.txt");std::fs::write(&source,"old").unwrap();assert_eq!(relative_name(&alias,&source).unwrap(),"a.txt");assert_eq!(workspace(&alias).unwrap(),workspace(&root).unwrap());
  std::os::unix::fs::symlink(&source,root.join("file-alias.txt")).unwrap();
  assert!(write(&alias,&root.join("file-alias.txt"),"new",None,None,||panic!("file aliases do not acquire authority")).is_err());
 }
}
