//! Private editor checkpoints, separate from source saves and provider evidence.
use super::{LocalState,api::{ApiError,ApiResult}};
use axum::{Json,extract::{State,Path as RoutePath,Query}};
use serde::{Deserialize,Serialize};
use serde_json::{Value,json};
use std::{collections::BTreeSet,path::{Path,PathBuf},io::Read};
const RECORD_LIMIT:usize=64*1024*1024;
const TEXT_LIMIT:usize=32*1024*1024;
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Address { Source{path:String}, Unnamed{name:String}, Generated{path:String,owner:String}, Detached{name:String,original_path:Option<String>} }
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {pub text:String,pub save_guard:Option<String>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {pub anchor:u32,pub head:u32}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View {pub selection:Selection,pub scroll_x:f64,pub scroll_y:f64}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Buffer {pub id:String,pub address:Address,pub text:String,pub baseline:Option<Baseline>,pub view:View}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {pub host:String,pub buffers:Vec<Buffer>,pub layout:Value}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {pub version:u32,pub workspace:String,pub window_slot:String,pub client_id:String,pub request_id:String,pub base_revision:u32,pub snapshot:Snapshot}
#[derive(Clone,Debug,Serialize,Deserialize)]
struct Saved {revision:u32,request_id:String,hash:String,snapshot:Snapshot}
#[derive(Clone,Debug,Serialize,Deserialize)]
struct Receipt {request_id:String,intent:String,revision:u32,hash:String}
#[derive(Clone,Debug,Serialize,Deserialize)]
struct Record {version:u32,client_id:String,#[serde(default)]window_slot:Option<String>,workspace:String,namespace:String,history:Vec<Saved>,receipts:Vec<Receipt>}
fn hash(bytes:&[u8])->String {super::workspace_fs::view::hash(bytes)}
fn slot(value:&str)->bool{!value.is_empty()&&value.len()<=20&&value.bytes().all(|byte|byte.is_ascii_digit())&&(value=="0"||!value.starts_with('0'))&&value.parse::<u64>().is_ok()}
fn token(value:&str)->bool{value.len()==32&&value.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))}
fn scalar_boundary(text:&str,position:u32)->bool{let mut offset=0u32;for ch in text.chars(){if offset==position{return true}offset+=ch.len_utf16() as u32;if offset>position{return false}}offset==position}
fn validate(snapshot:&Snapshot)->Result<(),String>{
 if !["browser","macos","ios","android"].contains(&snapshot.host.as_str()){return Err("Unknown editor checkpoint host.".into())}
 if snapshot.buffers.len()>256{return Err("Editor recovery retains at most 256 buffers per checkpoint; readable text is unaffected.".into())}
 let mut ids=BTreeSet::new();let mut bytes=0usize;
 for buffer in &snapshot.buffers{
  if !token(&buffer.id)||!ids.insert(&buffer.id){return Err("Recovery needs distinct opaque buffer identities.".into())}
  bytes=bytes.checked_add(buffer.text.len()).ok_or("Recovery text size overflow")?;
  if let Some(base)=&buffer.baseline{bytes=bytes.checked_add(base.text.len()).ok_or("Recovery baseline size overflow")?;if base.save_guard.as_ref().is_some_and(|guard|guard.len()>16384){return Err("Recovery save guard is too large.".into())}}
  if bytes>TEXT_LIMIT{return Err("Editor recovery text and baselines exceed the 32 MiB checkpoint budget. Keep drafts open and save explicitly.".into())}
  let label=match &buffer.address{Address::Source{path}|Address::Generated{path,..}=>path,Address::Unnamed{name}|Address::Detached{name,..}=>name};
  if serde_json::to_vec(&buffer.address).map_err(|e|e.to_string())?.len()>32768{return Err("Recovery address metadata exceeds its budget.".into())}
  if label.is_empty()||label.len()>16384||label.contains('\0'){return Err("Recovery source address is invalid.".into())}
  if !scalar_boundary(&buffer.text,buffer.view.selection.anchor)||!scalar_boundary(&buffer.text,buffer.view.selection.head){return Err("Recovery selection splits a Unicode scalar or exceeds the captured text.".into())}
  if [buffer.view.scroll_x,buffer.view.scroll_y].iter().any(|v|!v.is_finite()||*v<0.0||*v>1e9){return Err("Recovery scroll coordinates are invalid.".into())}
 }
 let layout=serde_json::to_vec(&snapshot.layout).map_err(|e|e.to_string())?;
 if layout.len()>512*1024{return Err("Recovery layout exceeds its 512 KiB metadata budget.".into())}
 Ok(())
}
fn directory(root:&Path,slot:Option<&str>)->Result<PathBuf,String>{
 let store=hickory_workspace::WorkspaceStore::for_project(root).and_then(|store|match slot{Some(slot)=>store.window(slot),None=>Ok(store)}).map_err(|e|e.to_string())?;
 let dir=store.dir().join("editor-recovery-v1");super::resource_journal::regular_directory(&dir)?;Ok(dir)
}
fn read_record(path:&Path)->Result<Record,String>{
 let metadata=std::fs::symlink_metadata(path).map_err(|e|e.to_string())?;
 if !metadata.is_file()||metadata.file_type().is_symlink()||metadata.len()>RECORD_LIMIT as u64{return Err("Recovery record is not a bounded ordinary file.".into())}
 let mut options=std::fs::OpenOptions::new();options.read(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK);}
 let file=options.open(path).map_err(|e|e.to_string())?;let held=same_file::Handle::from_file(file.try_clone().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
 let mut bytes=Vec::new();file.take(RECORD_LIMIT as u64+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
 if bytes.len()>RECORD_LIMIT||same_file::Handle::from_path(path).map_err(|e|e.to_string())?!=held{return Err("Recovery record changed while being read.".into())}
 let record:Record=serde_json::from_slice(&bytes).map_err(|e|format!("Recovery record is damaged: {e}"))?;
 if record.version!=1||!token(&record.client_id)||record.history.is_empty()||record.history.len()>3||record.receipts.len()>32{return Err("Recovery record shape is invalid.".into())}
 let mut previous=0;for saved in &record.history{validate(&saved.snapshot)?;if saved.revision<=previous||saved.hash!=hash(&serde_json::to_vec(&saved.snapshot).map_err(|e|e.to_string())?){return Err("Recovery checkpoint identity or content is damaged.".into())}previous=saved.revision}
 Ok(record)
}
fn receipt(record:&Record,saved:&Receipt,replayed:bool)->Value{json!({"version":1,"recovery_protocol":1,"client_id":record.client_id,"window_slot":record.window_slot,"workspace_id":record.workspace,"namespace":record.namespace,"request_id":saved.request_id,"revision":saved.revision,"snapshot_hash":saved.hash,"replayed":replayed,"source_effects":false,"source_saved":false,"notice":"Recorded a private editor checkpoint. Source files were not saved."})}
fn capture_at(directory:&Path,workspace:&str,namespace:&str,request:&Checkpoint,current:impl Fn()->Result<(),String>)->Result<Value,String>{
 if request.version!=1||!token(&request.client_id)||!token(&request.request_id)||!token(&request.workspace)||!slot(&request.window_slot){return Err("Recovery requires protocol 1 and opaque client/request identities.".into())}validate(&request.snapshot)?;current()?;
 super::resource_journal::regular_directory(directory)?;
 let lock_path=directory.join("checkpoint.lock");
 let mut options=std::fs::OpenOptions::new();options.create(true).read(true).write(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600).custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK);}
 let lock=options.open(&lock_path).map_err(|e|e.to_string())?;
 if !lock.metadata().map_err(|e|e.to_string())?.is_file(){return Err("Recovery checkpoint lock is not an ordinary file.".into())}
 fs2::FileExt::try_lock_exclusive(&lock).map_err(|_|"Another checkpoint is being recorded. Retain the draft and retry this exact request.".to_string())?;
 let held=same_file::Handle::from_file(lock.try_clone().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
 let path=directory.join(format!("{}.json",request.client_id));
 if !path.exists(){let count=std::fs::read_dir(directory).map_err(|e|e.to_string())?.take(4097).filter_map(Result::ok).filter(|entry|entry.path().extension().is_some_and(|value|value=="json")).take(32).count();if count>=32{return Err("Recovery client capacity reached. Retain buffers and inspect the existing private sessions before creating another.".into())}}
 let mut record=match std::fs::symlink_metadata(&path){Ok(_)=>read_record(&path)?,Err(e)if e.kind()==std::io::ErrorKind::NotFound=>Record{version:1,client_id:request.client_id.clone(),window_slot:Some(request.window_slot.clone()),workspace:workspace.into(),namespace:namespace.into(),history:vec![],receipts:vec![]},Err(e)=>return Err(e.to_string())};
 if record.client_id!=request.client_id||record.window_slot.as_deref()!=Some(request.window_slot.as_str())||record.workspace!=workspace||record.namespace!=namespace{return Err("The retained client belongs to an earlier workspace or window identity. Read it as historical and start a new recovery client.".into())}
 let intent=hash(&serde_json::to_vec(request).map_err(|e|e.to_string())?);
 if let Some(previous)=record.receipts.iter().find(|row|row.request_id==request.request_id){if previous.intent!=intent{return Err("Recovery request identity was reused with a different checkpoint.".into())}return Ok(receipt(&record,previous,true))}
 let previous=record.history.last().map_or(0,|snapshot|snapshot.revision);
 if request.base_revision!=previous{return Err("A newer recovery checkpoint already exists. Do not overwrite it with a delayed editor snapshot.".into())}
 let revision=previous.checked_add(1).ok_or("Recovery revision exhausted; retain drafts and use a new client identity")?;
 let snapshot_hash=hash(&serde_json::to_vec(&request.snapshot).map_err(|e|e.to_string())?);
 record.history.push(Saved{revision,request_id:request.request_id.clone(),hash:snapshot_hash.clone(),snapshot:request.snapshot.clone()});
 if record.history.len()>3{record.history.remove(0);}
 let saved=Receipt{request_id:request.request_id.clone(),intent,revision,hash:snapshot_hash};record.receipts.push(saved.clone());if record.receipts.len()>32{record.receipts.remove(0);}
 let bytes=loop{let bytes=serde_json::to_vec(&record).map_err(|e|e.to_string())?;if bytes.len()<=RECORD_LIMIT{break bytes}if record.history.len()<=1{return Err("Recovery record exceeds its 64 MiB storage budget. Source text remains in the editor.".into())}record.history.remove(0);};
 current()?;if same_file::Handle::from_path(&lock_path).map_err(|e|e.to_string())?!=held{return Err("Recovery lock changed. No checkpoint was published.".into())}
 super::store::write_atomic(&path,&bytes).map_err(|e|e.to_string())?;
 Ok(receipt(&record,&saved,false))
}
fn scope(state:&LocalState)->Result<(String,String,String),String>{
 if state.private_prediction.is_some()||crate::engine::private_policy::active().map_err(|error|error.to_string())?{return Err("Private prediction cannot access ordinary editor recovery; use its isolated state.".into())}
 let runtime=state.resource_root.check(state.index.root())?.to_owned();
 let physical=super::resource_inputs::RootIdentity::fingerprint(state.index.root())?;
 let namespace=state.index.root().to_str().ok_or("Recovery workspace address is not UTF-8")?.to_owned();
 Ok((runtime,physical,namespace))
}
pub async fn checkpoint(State(state):State<LocalState>,Json(request):Json<Checkpoint>)->ApiResult<Json<Value>>{
 let (runtime,physical,namespace)=scope(&state).map_err(ApiError::conflict)?;
 if request.window_slot!=state.window_slot.as_deref().unwrap_or("0"){return Err(ApiError::conflict("Recovery window changed. Do not write a delayed snapshot into another window's private session."))}
 if request.workspace!=runtime{return Err(ApiError::conflict("Recovery workspace changed. Preserve the buffers and capture a new checkpoint for this engine."))}
 tokio::task::spawn_blocking(move||->Result<Json<Value>,String>{
  if scope(&state)!=Ok((runtime.clone(),physical.clone(),namespace.clone())){return Err("Recovery scope changed before storage admission.".into())}
  let dir=directory(state.index.root(),state.window_slot.as_deref())?;
  capture_at(&dir,&physical,&namespace,&request,||{
   let current=scope(&state)?;if current!=(runtime.clone(),physical.clone(),namespace.clone()){return Err("Workspace changed before private checkpoint publication.".into())}Ok(())
  }).map(Json)
 }).await.map_err(|e|ApiError::internal(e.to_string()))?.map_err(ApiError::conflict)
}
#[derive(Deserialize)]pub struct ListQuery{pub after:Option<String>,pub window_slot:Option<String>}
pub async fn list(State(state):State<LocalState>,Query(query):Query<ListQuery>)->ApiResult<Json<Value>>{
 let(runtime,physical,namespace)=scope(&state).map_err(ApiError::conflict)?;
 if query.after.as_ref().is_some_and(|id|!token(id)){return Err(ApiError::bad_request("Recovery page cursor is invalid"))}
 let selected_slot=query.window_slot.clone().unwrap_or_else(||state.window_slot.clone().unwrap_or_else(||"0".into()));if !slot(&selected_slot){return Err(ApiError::bad_request("Recovery window slot is invalid"))}
 tokio::task::spawn_blocking(move||->Result<Json<Value>,String>{
  if scope(&state)!=Ok((runtime.clone(),physical.clone(),namespace.clone())){return Err("Recovery scope changed before private storage reading.".into())}
  let dir=directory(state.index.root(),Some(&selected_slot))?;
  let mut names=Vec::new();
  for entry in std::fs::read_dir(&dir).map_err(|e|e.to_string())?.take(4097){let entry=entry.map_err(|e|e.to_string())?;let name=entry.file_name();let Some(name)=name.to_str()else{continue};if let Some(id)=name.strip_suffix(".json"){if token(id)&&query.after.as_deref().is_none_or(|after|id>after){names.push(id.to_owned());}}}
  names.sort();let mut rows=Vec::new();let mut notices=Vec::new();let mut next=None;let mut bytes=0u64;
  for (index,id) in names.iter().enumerate(){
   let path=dir.join(format!("{id}.json"));let cost=std::fs::symlink_metadata(&path).map(|meta|meta.len()).unwrap_or(0);
   if index>=8||(!rows.is_empty()&&bytes.saturating_add(cost)>RECORD_LIMIT as u64){break}
   next=Some(id.clone());bytes=bytes.saturating_add(cost);
   match read_record(&path){Ok(record)if record.client_id==*id=>{let saved=record.history.last().unwrap();rows.push(json!({"client_id":id,"window_slot":selected_slot,"origin_window_slot":record.window_slot,"workspace_id":record.workspace,"namespace":record.namespace,"current_workspace":record.workspace==physical&&record.namespace==namespace,"revision":saved.revision,"request_id":saved.request_id,"snapshot_hash":saved.hash,"host":saved.snapshot.host,"buffers":saved.snapshot.buffers.len(),"source_effects":false}));},Ok(_)=>notices.push(format!("Recovery record {id} does not match its address; it was not adopted.")),Err(error)=>notices.push(format!("Recovery record {id} is unreadable: {error}"))}
  }
  let more=next.as_ref().is_some_and(|last|names.last().is_some_and(|id|id>last));
  if scope(&state)!=Ok((runtime.clone(),physical.clone(),namespace.clone())){return Err("Workspace changed while listing private editor recovery.".into())}
  let base=hickory_workspace::WorkspaceStore::for_project(state.index.root()).map_err(|error|error.to_string())?;let mut slots=BTreeSet::from(["0".to_string(),selected_slot.clone()]);let windows=base.dir().join("windows");let mut window_notice=None;
  if std::fs::symlink_metadata(&windows).is_ok_and(|meta|meta.is_dir()&&!meta.file_type().is_symlink()){for (index,entry)in std::fs::read_dir(&windows).map_err(|error|error.to_string())?.enumerate(){if index>=256{window_notice=Some("Recovery window listing is bounded; additional private windows remain retained.");break}let entry=entry.map_err(|error|error.to_string())?;let name=entry.file_name();if let Some(value)=name.to_str(){if slot(value)&&entry.file_type().map_err(|error|error.to_string())?.is_dir(){slots.insert(value.to_owned());}}}}
  if let Some(notice)=window_notice{notices.push(notice.into())}
  if scope(&state)!=Ok((runtime.clone(),physical.clone(),namespace.clone())){return Err("Recovery scope changed while enumerating retained windows.".into())}
  Ok(Json(json!({"version":1,"recovery_protocol":1,"engine_workspace_id":runtime,"workspace_id":physical,"namespace":namespace,"window_slot":selected_slot,"current_window_slot":state.window_slot.as_deref().unwrap_or("0"),"window_slots":slots,"sessions":rows,"notices":notices,"next":if more{next}else{None},"source_effects":false,"notice":"Private snapshots are recoverable drafts, not source saves or test evidence."})))
 }).await.map_err(|e|ApiError::internal(e.to_string()))?.map_err(ApiError::conflict)
}
#[derive(Deserialize)]pub struct ReadQuery{pub revision:Option<u32>,pub window_slot:Option<String>}
pub async fn read(State(state):State<LocalState>,RoutePath(id):RoutePath<String>,Query(query):Query<ReadQuery>)->ApiResult<Json<Value>>{
 if !token(&id){return Err(ApiError::bad_request("Recovery client identity is invalid"))}
 let(runtime,physical,namespace)=scope(&state).map_err(ApiError::conflict)?;
 let selected_slot=query.window_slot.clone().unwrap_or_else(||state.window_slot.clone().unwrap_or_else(||"0".into()));if !slot(&selected_slot){return Err(ApiError::bad_request("Recovery window slot is invalid"))}
 tokio::task::spawn_blocking(move||->Result<Json<Value>,String>{
  if scope(&state)!=Ok((runtime.clone(),physical.clone(),namespace.clone())){return Err("Recovery scope changed before private storage reading.".into())}
  let dir=directory(state.index.root(),Some(&selected_slot))?;let record=read_record(&dir.join(format!("{id}.json")))?;
  if record.client_id!=id{return Err("Recovery record does not match its requested client identity.".into())}
  let saved=match query.revision{Some(revision)=>record.history.iter().find(|saved|saved.revision==revision).ok_or("Requested recovery revision is no longer retained")?,None=>record.history.last().unwrap()};
  let current=record.workspace==physical&&record.namespace==namespace;
  let mut authority=Vec::new();
  for buffer in &saved.snapshot.buffers{
   let observation=match (&buffer.address,&buffer.baseline){
    (Address::Source{path},Some(base))if current&&!Path::new(path).is_absolute()=>{
     match super::resource_inputs::saved_target(state.index.root(),path){
      Ok((full,_))=>match base.save_guard.as_deref(){Some(guard)=>super::source_history::validate_observed_guard(state.index.root(),&full,path,guard).and_then(|_|{
       let source=super::source_history::read_observed(state.index.root(),&full,path)?;if source.source!=base.text{return Err("Recorded baseline differs from the physically guarded source.".into())}Ok(())
      }),None=>Err("No originally loaded save guard was retained.".into())},
      Err(error)=>Err(error),
     }
    },
    (Address::Generated{..},_)=>Err("Generated text remains read-only; recover its authored owner before changing outputs.".into()),
    _=>Err("This retained text has no current physically proven named baseline. Recover it as a draft and choose a guarded destination explicitly.".into()),
   };
   authority.push(json!({"buffer_id":buffer.id,"baseline_current":observation.is_ok(),"reason":observation.err(),"write_permission":false}));
  }
  if scope(&state)!=(Ok((runtime.clone(),physical.clone(),namespace.clone()))){return Err("Workspace changed while the recovery preview was being read.".into())}
  Ok(Json(json!({"version":1,"recovery_protocol":1,"engine_workspace_id":runtime,"client_id":record.client_id,"window_slot":selected_slot,"origin_window_slot":record.window_slot,"workspace_id":record.workspace,"namespace":record.namespace,"current_workspace":current,"revision":saved.revision,"request_id":saved.request_id,"snapshot_hash":saved.hash,"material":serde_json::to_string(&saved.snapshot).map_err(|e|e.to_string())?,"retained_revisions":record.history.iter().map(|saved|saved.revision).collect::<Vec<_>>(),"authority":authority,"source_effects":false,"source_saved":false,"notice":"Preview only. Restoring text must use the current host editor lease; saving remains separately guarded."})))
 }).await.map_err(|e|ApiError::internal(e.to_string()))?.map_err(ApiError::conflict)
}
#[cfg(test)]mod tests{
 use super::*;
 fn request(id:u8,revision:u32,text:&str)->Checkpoint{Checkpoint{version:1,workspace:"e".repeat(32),window_slot:"0".into(),client_id:"a".repeat(32),request_id:format!("{id:032x}"),base_revision:revision,snapshot:Snapshot{host:"browser".into(),buffers:vec![Buffer{id:"b".repeat(32),address:Address::Source{path:"café #.py".into()},text:text.into(),baseline:Some(Baseline{text:"saved".into(),save_guard:None}),view:View{selection:Selection{anchor:0,head:0},scroll_x:0.0,scroll_y:17.0}}],layout:json!({"tabs":[{"kind":"source","buffer_id":"b".repeat(32)}]})}}}
 #[test]fn exact_lost_reply_retry_and_delayed_checkpoints_never_replace_newer_drafts(){
  let dir=tempfile::tempdir().unwrap();let first=request(1,0,"draft one");let response=capture_at(dir.path(),&"c".repeat(64),"/workspace",&first,||Ok(())).unwrap();assert_eq!(response["revision"],1);assert_eq!(response["source_saved"],false);
  let bytes=std::fs::read(dir.path().join(format!("{}.json",first.client_id))).unwrap();
  assert_eq!(capture_at(dir.path(),&"c".repeat(64),"/workspace",&first,||Ok(())).unwrap()["replayed"],true);assert_eq!(std::fs::read(dir.path().join(format!("{}.json",first.client_id))).unwrap(),bytes);
  let second=request(2,1,"draft two");capture_at(dir.path(),&"c".repeat(64),"/workspace",&second,||Ok(())).unwrap();
  assert!(capture_at(dir.path(),&"c".repeat(64),"/workspace",&request(3,0,"delayed stale"),||Ok(())).is_err());
  let mut changed=first.clone();changed.snapshot.buffers[0].text="changed retry".into();assert!(capture_at(dir.path(),&"c".repeat(64),"/workspace",&changed,||Ok(())).is_err());
  let restored=read_record(&dir.path().join(format!("{}.json",first.client_id))).unwrap();assert_eq!(restored.history.last().unwrap().snapshot.buffers[0].text,"draft two");assert_eq!(restored.history.len(),2);
 }
 #[test]fn restart_preserves_exact_unicode_unnamed_and_generated_text_without_path_materialization(){
  let parent=tempfile::tempdir().unwrap();let directory=parent.path().join("private");std::fs::create_dir(&directory).unwrap();let mut snapshot=request(1,0,"dirty 😀");
  for (id,address) in [("d",Address::Source{path:"cafe\u{301} #.py".into()}),("e",Address::Unnamed{name:"note".into()}),("f",Address::Generated{path:"out.py".into(),owner:"construction.md".into()})]{let mut buffer=snapshot.snapshot.buffers[0].clone();buffer.id=id.repeat(32);buffer.address=address;buffer.text=format!("{id} retained");snapshot.snapshot.buffers.push(buffer);}
  capture_at(&directory,&"c".repeat(64),"/workspace",&snapshot,||Ok(())).unwrap();let restored=read_record(&directory.join(format!("{}.json",snapshot.client_id))).unwrap();assert_eq!(restored.history[0].snapshot.buffers.len(),4);assert_eq!(serde_json::to_value(&restored.history[0].snapshot).unwrap(),serde_json::to_value(&snapshot.snapshot).unwrap());assert_eq!(std::fs::read_dir(parent.path()).unwrap().count(),1);
 }
 #[test]fn foreign_workspace_and_revoked_capture_preserve_recorded_private_and_source_bytes(){
  let root=tempfile::tempdir().unwrap();let source=root.path().join("source.txt");std::fs::write(&source,"saved").unwrap();let dir=root.path().join("private");std::fs::create_dir(&dir).unwrap();let first=request(1,0,"unsaved");capture_at(&dir,&"c".repeat(64),"/workspace",&first,||Ok(())).unwrap();let path=dir.join(format!("{}.json",first.client_id));let original=std::fs::read(&path).unwrap();
  assert!(capture_at(&dir,&"d".repeat(64),"/workspace",&request(2,1,"foreign"),||Ok(())).is_err());
  let checks=std::cell::Cell::new(0);assert!(capture_at(&dir,&"c".repeat(64),"/workspace",&request(2,1,"revoked"),||{checks.set(checks.get()+1);if checks.get()>1{Err("workspace replaced".into())}else{Ok(())}}).is_err());
  assert_eq!(std::fs::read(path).unwrap(),original);assert_eq!(std::fs::read_to_string(source).unwrap(),"saved");
 }
 #[test]fn duplicate_handles_split_surrogates_and_nonfinite_geometry_refuse_before_storage(){
  let dir=tempfile::tempdir().unwrap();let mut snapshot=request(1,0,"😀");snapshot.snapshot.buffers[0].view.selection.head=1;assert!(capture_at(dir.path(),&"c".repeat(64),"/workspace",&snapshot,||Ok(())).is_err());snapshot.snapshot.buffers[0].view.selection.head=2;snapshot.snapshot.buffers[0].view.scroll_y=f64::NAN;assert!(validate(&snapshot.snapshot).is_err());snapshot.snapshot.buffers[0].view.scroll_y=0.0;snapshot.snapshot.buffers.push(snapshot.snapshot.buffers[0].clone());assert!(validate(&snapshot.snapshot).is_err());assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(),0);
 }
 #[cfg(unix)]#[test]fn symlinked_records_or_foreign_checkpoint_locks_never_get_overwritten(){
  let dir=tempfile::tempdir().unwrap();let foreign=dir.path().join("foreign");std::fs::write(&foreign,"outside record").unwrap();let snapshot=request(1,0,"draft");let path=dir.path().join(format!("{}.json",snapshot.client_id));std::os::unix::fs::symlink(&foreign,&path).unwrap();assert!(capture_at(dir.path(),&"c".repeat(64),"/workspace",&snapshot,||Ok(())).is_err());assert_eq!(std::fs::read_to_string(&foreign).unwrap(),"outside record");std::fs::remove_file(path).unwrap();
  let lock=std::fs::OpenOptions::new().read(true).write(true).open(dir.path().join("checkpoint.lock")).unwrap();fs2::FileExt::lock_exclusive(&lock).unwrap();assert!(capture_at(dir.path(),&"c".repeat(64),"/workspace",&snapshot,||Ok(())).is_err());
 }
 #[test]fn a_retained_client_cannot_cross_window_scope_or_reuse_unbound_legacy_record(){
  let dir=tempfile::tempdir().unwrap();let first=request(1,0,"window zero");capture_at(dir.path(),&"c".repeat(64),"/workspace",&first,||Ok(())).unwrap();let path=dir.path().join(format!("{}.json",first.client_id));let bytes=std::fs::read(&path).unwrap();let mut other=request(2,1,"other window");other.window_slot="1".into();assert!(capture_at(dir.path(),&"c".repeat(64),"/workspace",&other,||Ok(())).is_err());assert_eq!(std::fs::read(&path).unwrap(),bytes);
  let mut legacy:Value=serde_json::from_slice(&bytes).unwrap();legacy.as_object_mut().unwrap().remove("window_slot");std::fs::write(&path,serde_json::to_vec(&legacy).unwrap()).unwrap();assert_eq!(read_record(&path).unwrap().history[0].snapshot.buffers[0].text,"window zero");assert!(capture_at(dir.path(),&"c".repeat(64),"/workspace",&request(3,1,"not an authorized writer reuse"),||Ok(())).is_err());
 }
 #[test]fn retained_history_is_bounded_and_corrupt_records_do_not_parse_as_recovery_authority(){
  let dir=tempfile::tempdir().unwrap();for revision in 0..5{capture_at(dir.path(),&"c".repeat(64),"/workspace",&request(revision as u8+1,revision,&format!("draft {revision}")),||Ok(())).unwrap();}let path=dir.path().join(format!("{}.json","a".repeat(32)));let restored=read_record(&path).unwrap();assert_eq!(restored.history.len(),3);assert_eq!(restored.history[0].revision,3);
  let mut damaged:Value=serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();damaged["history"][0]["snapshot"]["buffers"][0]["text"]=json!("tampered");std::fs::write(&path,serde_json::to_vec(&damaged).unwrap()).unwrap();assert!(read_record(&path).unwrap_err().contains("damaged"));
 }
}
#[cfg(test)]pub(crate) async fn assert_private_refused(state:&LocalState){
 let request:Checkpoint=serde_json::from_value(json!({"version":1,"workspace":"e".repeat(32),"window_slot":"0","client_id":"a".repeat(32),"request_id":"b".repeat(32),"base_revision":0,"snapshot":{"host":"browser","buffers":[],"layout":{}}})).unwrap();
 assert!(checkpoint(State(state.clone()),Json(request)).await.is_err());
 assert!(list(State(state.clone()),Query(ListQuery{after:None,window_slot:None})).await.is_err());
 assert!(read(State(state.clone()),RoutePath("a".repeat(32)),Query(ReadQuery{revision:None,window_slot:None})).await.is_err());
}
