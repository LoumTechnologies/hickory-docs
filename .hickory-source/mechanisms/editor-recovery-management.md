<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Select and manage bounded private recovery sessions

For maintainers making every retained checkpoint reachable without granting source-save authority.

List every page of up to32 private sessions and expose all3 revision identities. Forgetting is an explicit whole-session operation, distinct from recovering copies: it never edits a current source or editor. Bind removal to the observed entire record (including physical ctime identity), current engine/root/window and held private directory. Retry an exact lost acknowledgment without deleting a replacement. Removal atomically moves the observed address to a private unpredictable hold name, verifies the actual moved file identity and complete bytes, then deletes only that held record. A raced replacement is restored without overwriting a newer original address; failures retain bytes explicitly. Source remains immediately readable when optional recovery is unavailable.

## Hold private storage before observing or changing a record

The workspace library resolves a project path without creating folders. Walk owned private components through no-follow directory handles; do not let a symlinked window ancestor redirect a checkpoint or removal. Reuse the shared physical record identity helper to reject equal-byte replacement and edit/Undo. A bounded replay cache returns the original private-removal receipt without repeating unlink.

<ui:copy id="editor-recovery-management-core">
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordGuard {version:u32,runtime:String,physical:String,namespace:String,window_slot:String,client_id:String,directory:String,identity:String,hash:String}
fn recovery_page_full(processed:usize,bytes:u64,cost:u64)->bool{processed>=8||(processed>0&&bytes.saturating_add(cost)>RECORD_LIMIT as u64)}
struct RecordObservation {bytes:Vec<u8>,identity:String,hash:String,handle:same_file::Handle}
fn observe_record(path:&Path)->Result<RecordObservation,String>{
 let before=std::fs::symlink_metadata(path).map_err(|e|e.to_string())?;
 if before.file_type().is_symlink()||!before.is_file()||before.len()>RECORD_LIMIT as u64{return Err("Recovery record is not a bounded ordinary file.".into())}
 let mut options=std::fs::OpenOptions::new();options.read(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK|libc::O_CLOEXEC);}
 let file=options.open(path).map_err(|e|e.to_string())?;let identity=super::source_history::identity_for_metadata(&file.metadata().map_err(|e|e.to_string())?)?;
 let held=same_file::Handle::from_file(file.try_clone().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
 let mut bytes=Vec::new();file.take(RECORD_LIMIT as u64+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
 let after=std::fs::symlink_metadata(path).map_err(|e|e.to_string())?;
 if bytes.len()>RECORD_LIMIT||after.file_type().is_symlink()||same_file::Handle::from_path(path).map_err(|e|e.to_string())?!=held||super::source_history::identity_for_metadata(&after)?!=identity{return Err("Recovery record changed while being observed.".into())}
 Ok(RecordObservation{hash:hash(&bytes),bytes,identity,handle:held})
}
fn read_observed_record(path:&Path)->Result<(Record,RecordObservation),String>{let observed=observe_record(path)?;let record=decode_record(&observed.bytes)?;Ok((record,observed))}
fn revision_rows(record:&Record)->Vec<Value>{record.history.iter().map(|saved|json!({"revision":saved.revision,"request_id":saved.request_id,"snapshot_hash":saved.hash,"host":saved.snapshot.host,"buffers":saved.snapshot.buffers.len()})).collect()}
#[cfg(unix)]
struct RecoveryDirectory {path:PathBuf,file:std::fs::File,identity:String}
#[cfg(unix)]
impl RecoveryDirectory {
 fn open(path:&Path)->Result<Self,String>{
  use std::os::unix::fs::OpenOptionsExt;
  if std::fs::symlink_metadata(path).map_err(|e|e.to_string())?.file_type().is_symlink(){return Err("Private recovery directories cannot be symbolic links.".into())}
  let file=std::fs::OpenOptions::new().read(true).custom_flags(libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC).open(path).map_err(|e|e.to_string())?;
  let value=Self{path:path.to_path_buf(),file,identity:super::resource_inputs::RootIdentity::fingerprint(path)?};value.check()?;Ok(value)
 }
 fn child(&self,name:&str)->Result<Self,String>{
  use std::os::fd::{AsRawFd,FromRawFd};self.check()?;if name.is_empty()||Path::new(name).components().count()!=1||Path::new(name).components().any(|part|!matches!(part,std::path::Component::Normal(_))){return Err("Invalid private recovery component.".into())}
  let value=std::ffi::CString::new(name).map_err(|e|e.to_string())?;
  if unsafe{libc::mkdirat(self.file.as_raw_fd(),value.as_ptr(),0o700)}!=0{let error=std::io::Error::last_os_error();if error.kind()!=std::io::ErrorKind::AlreadyExists{return Err(error.to_string())}}
  let fd=unsafe{libc::openat(self.file.as_raw_fd(),value.as_ptr(),libc::O_RDONLY|libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC)};if fd<0{return Err("Private recovery ancestor is not an ordinary held directory.".into())}
  let path=self.path.join(name);let result=Self{path:path.clone(),file:unsafe{std::fs::File::from_raw_fd(fd)},identity:super::resource_inputs::RootIdentity::fingerprint(&path)?};self.check()?;result.check()?;Ok(result)
 }
 fn check(&self)->Result<(),String>{
  if std::fs::symlink_metadata(&self.path).map_err(|e|e.to_string())?.file_type().is_symlink()||same_file::Handle::from_file(self.file.try_clone().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?!=same_file::Handle::from_path(&self.path).map_err(|e|e.to_string())?||super::resource_inputs::RootIdentity::fingerprint(&self.path)?!=self.identity{return Err("Private recovery storage was replaced. No substituted record was adopted.".into())}Ok(())
 }
 fn lock(&self)->Result<std::fs::File,String>{
  use std::os::fd::{AsRawFd,FromRawFd};self.check()?;let name=std::ffi::CString::new("checkpoint.lock").unwrap();let fd=unsafe{libc::openat(self.file.as_raw_fd(),name.as_ptr(),libc::O_RDWR|libc::O_CREAT|libc::O_NOFOLLOW|libc::O_NONBLOCK|libc::O_CLOEXEC,0o600)};
  if fd<0{return Err(std::io::Error::last_os_error().to_string())}let lock=unsafe{std::fs::File::from_raw_fd(fd)};if !lock.metadata().map_err(|e|e.to_string())?.is_file(){return Err("Recovery lock is not an ordinary file.".into())}fs2::FileExt::try_lock_exclusive(&lock).map_err(|_|"Another checkpoint or private session removal is in progress; retry the exact request.".to_string())?;Ok(lock)
 }
 fn validate_lock(&self,lock:&std::fs::File)->Result<(),String>{self.check()?;let path=self.path.join("checkpoint.lock");if std::fs::symlink_metadata(&path).map_err(|e|e.to_string())?.file_type().is_symlink()||same_file::Handle::from_file(lock.try_clone().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?!=same_file::Handle::from_path(path).map_err(|e|e.to_string())?{return Err("Recovery lock changed; no private session operation was admitted.".into())}Ok(())}
 fn publish(&self,name:&str,bytes:&[u8])->Result<(),String>{
  use std::{io::Write,os::fd::{AsRawFd,FromRawFd}};self.check()?;let mut id=[0;16];getrandom::fill(&mut id).map_err(|e|e.to_string())?;let temporary=std::ffi::CString::new(format!(".{}.tmp",hex::encode(id))).unwrap();let name=std::ffi::CString::new(name).map_err(|e|e.to_string())?;
  let fd=unsafe{libc::openat(self.file.as_raw_fd(),temporary.as_ptr(),libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL|libc::O_NOFOLLOW|libc::O_CLOEXEC,0o600)};if fd<0{return Err(std::io::Error::last_os_error().to_string())}let mut file=unsafe{std::fs::File::from_raw_fd(fd)};
  let result=(||{file.write_all(bytes).and_then(|_|file.sync_all()).map_err(|e|e.to_string())?;self.check()?;if unsafe{libc::renameat(self.file.as_raw_fd(),temporary.as_ptr(),self.file.as_raw_fd(),name.as_ptr())}!=0{return Err(std::io::Error::last_os_error().to_string())}self.file.sync_all().map_err(|e|e.to_string())?;self.check()})();if result.is_err(){unsafe{libc::unlinkat(self.file.as_raw_fd(),temporary.as_ptr(),0)};}result
 }
 fn remove_observed(&self,name:&str,expected:&RecordObservation)->Result<(),String>{
  use std::os::fd::AsRawFd;self.check()?;let mut random=[0;16];getrandom::fill(&mut random).map_err(|e|e.to_string())?;let temporary=format!(".forget-{}.hold",hex::encode(random));let source=std::ffi::CString::new(name).map_err(|e|e.to_string())?;let destination=std::ffi::CString::new(temporary.as_str()).unwrap();
  if unsafe{libc::renameat(self.file.as_raw_fd(),source.as_ptr(),self.file.as_raw_fd(),destination.as_ptr())}!=0{return Err(std::io::Error::last_os_error().to_string())}
  let observed=observe_record(&self.path.join(&temporary));
  let matches=observed.as_ref().is_ok_and(|actual|actual.handle==expected.handle&&actual.hash==expected.hash);
  if !matches||self.check().is_err(){
   // Restore without overwriting a replacement published at the original address.
   let restored=unsafe{libc::linkat(self.file.as_raw_fd(),destination.as_ptr(),self.file.as_raw_fd(),source.as_ptr(),0)}==0;
   if restored{unsafe{libc::unlinkat(self.file.as_raw_fd(),destination.as_ptr(),0)};}
   let _=self.file.sync_all();return Err(if restored{"Private session changed during removal. Its record was restored and no revisions were deleted.".into()}else{format!("Private session changed during removal. Its bytes remain privately retained as {temporary}; the replacement was not deleted.")});
  }
  if unsafe{libc::unlinkat(self.file.as_raw_fd(),destination.as_ptr(),0)}!=0{return Err("Private session was detached but its retained bytes could not be removed; inspect private recovery before retrying.".into())}self.file.sync_all().map_err(|e|e.to_string())?;self.check()
 }
}
#[cfg(unix)]
fn recovery_base(path:&Path)->Result<RecoveryDirectory,String>{
 let path=if path.is_absolute(){path.to_path_buf()}else{std::env::current_dir().map_err(|e|e.to_string())?.join(path)};if path.components().any(|part|matches!(part,std::path::Component::ParentDir)){return Err("Private state path contains parent traversal.".into())}
 let mut ancestor=path.clone();let mut missing=Vec::new();loop{match std::fs::symlink_metadata(&ancestor){Ok(meta)=>{if !meta.is_dir()||meta.file_type().is_symlink(){return Err("Private state root is not an ordinary directory.".into())}break},Err(error)if error.kind()==std::io::ErrorKind::NotFound=>{missing.push(ancestor.file_name().ok_or("Private state path has no ancestor")?.to_os_string());ancestor.pop();},Err(error)=>return Err(error.to_string())}}
 let mut held=RecoveryDirectory::open(&ancestor)?;for part in missing.into_iter().rev(){held=held.child(part.to_str().ok_or("Private state component is not exact UTF-8")?)?;}held.check()?;Ok(held)
}
#[cfg(unix)]
fn recovery_project(root:&Path)->Result<RecoveryDirectory,String>{let base=hickory_workspace::data_root().map_err(|e|e.to_string())?;let path=hickory_workspace::WorkspaceStore::project_directory(root).map_err(|e|e.to_string())?;let mut held=recovery_base(&base)?;for part in path.strip_prefix(&base).map_err(|_|"Private project address escaped user state")?.components(){let std::path::Component::Normal(part)=part else{return Err("Private project address has invalid components".into())};held=held.child(part.to_str().ok_or("Private project component is not UTF-8")?)?;}Ok(held)}
#[cfg(unix)]
fn recovery_directory(root:&Path,selected:Option<&str>)->Result<RecoveryDirectory,String>{let mut held=recovery_project(root)?;if let Some(selected)=selected.filter(|slot|*slot!="0"){if !slot(selected){return Err("Private recovery window identity is invalid".into())}held=held.child("windows")?.child(selected)?;}held.child("editor-recovery-v1")}
#[cfg(not(unix))]
fn directory(_root:&Path,_slot:Option<&str>)->Result<PathBuf,String>{Err("Held private recovery storage is unavailable on this engine platform. Readable editor text is retained.".into())}
#[cfg(unix)]
fn directory(root:&Path,selected:Option<&str>)->Result<PathBuf,String>{Ok(recovery_directory(root,selected)?.path)}
#[cfg(not(unix))]
fn capture_at(_directory:&Path,_workspace:&str,_namespace:&str,_request:&Checkpoint,_current:impl Fn()->Result<(),String>)->Result<Value,String>{Err("Held private recovery checkpoint publication is unavailable on this engine platform.".into())}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForgetRequest {version:u32,workspace:String,window_slot:String,client_id:String,request_id:String,record_guard:String}
#[derive(Default)]
struct ForgetReceipts {rows:std::collections::VecDeque<(String,String,Value)>}
static FORGOTTEN:std::sync::LazyLock<std::sync::Mutex<ForgetReceipts>>=std::sync::LazyLock::new(||std::sync::Mutex::new(ForgetReceipts::default()));
fn record_guard(directory:&Path,id:&str,observed:&RecordObservation,scope:&(String,String,String),selected:&str)->Result<String,String>{serde_json::to_string(&RecordGuard{version:1,runtime:scope.0.clone(),physical:scope.1.clone(),namespace:scope.2.clone(),window_slot:selected.into(),client_id:id.into(),directory:super::resource_inputs::RootIdentity::fingerprint(directory)?,identity:observed.identity.clone(),hash:observed.hash.clone()}).map_err(|e|e.to_string())}
#[cfg(unix)]
fn forget_at(directory:&RecoveryDirectory,scope:&(String,String,String),request:&ForgetRequest,current:impl Fn()->Result<(),String>)->Result<Value,String>{
 if request.version!=1||!token(&request.workspace)||!token(&request.client_id)||!token(&request.request_id)||!slot(&request.window_slot)||request.record_guard.len()>16384{return Err("Private removal requires current opaque workspace/client/request identities.".into())}
 if request.workspace!=scope.0{return Err("Recovery engine changed; this removal cannot be rebound.".into())}
 let guard:RecordGuard=serde_json::from_str(&request.record_guard).map_err(|_|"Private record guard is malformed")?;
 if guard.version!=1||guard.runtime!=scope.0||guard.physical!=scope.1||guard.namespace!=scope.2||guard.client_id!=request.client_id||guard.window_slot!=request.window_slot||guard.directory!=directory.identity{return Err("Private record/root/window scope changed. Refresh before forgetting a session.".into())}
 let intent=hash(&serde_json::to_vec(request).map_err(|e|e.to_string())?);let key=format!("{}:{}",scope.0,request.request_id);current()?;directory.check()?;
 let mut receipts=FORGOTTEN.lock().map_err(|_|"Private removal replay cache unavailable")?;
 if let Some((_,previous,value))=receipts.rows.iter().find(|(id,_,_)|*id==key){if previous!=&intent{return Err("Private removal request was reused with a different session.".into())}let mut value=value.clone();value["replayed"]=json!(true);return Ok(value)}
 let lock=directory.lock()?;let name=format!("{}.json",request.client_id);let observed=observe_record(&directory.path.join(&name))?;
 if observed.identity!=guard.identity||observed.hash!=guard.hash{return Err("The private session changed or was replaced. No retained revision was forgotten.".into())}
 current()?;directory.validate_lock(&lock)?;directory.remove_observed(&name,&observed)?;
 let receipt=json!({"version":1,"recovery_protocol":1,"engine_workspace_id":scope.0,"workspace_id":scope.1,"namespace":scope.2,"window_slot":request.window_slot,"client_id":request.client_id,"request_id":request.request_id,"guard_hash":hash(request.record_guard.as_bytes()),"forgotten":true,"replayed":false,"source_effects":false,"source_saved":false});
 receipts.rows.push_back((key,intent,receipt.clone()));while receipts.rows.len()>64{receipts.rows.pop_front();}Ok(receipt)
}
pub async fn forget(State(state):State<LocalState>,Json(request):Json<ForgetRequest>)->ApiResult<Json<Value>>{
 let _ordinary=crate::engine::private_policy::ordinary();let original=scope(&state).map_err(ApiError::conflict)?;
 if request.workspace!=original.0||request.version!=1||!slot(&request.window_slot)||!token(&request.client_id)||!token(&request.request_id){return Err(ApiError::conflict("Private removal scope or request changed before storage admission."))}
 tokio::task::spawn_blocking(move||->Result<Json<Value>,String>{
  #[cfg(unix)]{let held=recovery_directory(state.index.root(),Some(&request.window_slot))?;forget_at(&held,&original,&request,||{if scope(&state)?!=original{return Err("Workspace changed before private session removal.".into())}Ok(())}).map(Json)}
  #[cfg(not(unix))]{let _=(state,request,original);Err("Guarded private session removal is unavailable on this engine platform.".into())}
 }).await.map_err(|e|ApiError::internal(e.to_string()))?.map_err(ApiError::conflict)
}
</ui:copy>

## Exercise capacity, retained revisions and refusal paths before touching current editors

These focused cases fill all32 private session slots, remove exactly one explicit session and admit a new checkpoint. An exact replay must leave a newly created replacement intact. Independent cases retire stale revision, root, window and physical-file guards, inject a replacement after initial guard validation, and discard bounded corrupt private bytes without adopting them as a checkpoint. No test confers source-save authority.

<ui:copy id="editor-recovery-management-tests">
#[cfg(all(test,unix))]
mod management_tests {
 use super::*;
 fn request(client:u8,revision:u32,text:&str)->Checkpoint{Checkpoint{version:1,workspace:"e".repeat(32),window_slot:"0".into(),client_id:format!("{client:032x}"),request_id:nonce(),base_revision:revision,snapshot:Snapshot{host:"browser".into(),buffers:vec![Buffer{id:"b".repeat(32),address:Address::Unnamed{name:"retained draft".into()},text:text.into(),baseline:None,view:View{selection:Selection{anchor:0,head:0},scroll_x:0.0,scroll_y:0.0}}],layout:json!({})}}}
 fn nonce()->String{let mut value=[0;16];getrandom::fill(&mut value).unwrap();hex::encode(value)}
 fn scope()->(String,String,String){("e".repeat(32),"c".repeat(64),"/workspace".into())}
 fn removal(directory:&RecoveryDirectory,id:&str)->ForgetRequest{let original=scope();let observed=observe_record(&directory.path.join(format!("{id}.json"))).unwrap();ForgetRequest{version:1,workspace:original.0.clone(),window_slot:"0".into(),client_id:id.into(),request_id:nonce(),record_guard:record_guard(&directory.path,id,&observed,&original,"0").unwrap()}}
 #[test]
 fn capacity_is_released_explicitly_and_exact_replay_never_deletes_a_recreated_session(){
  let temp=tempfile::tempdir().unwrap();let held=RecoveryDirectory::open(temp.path()).unwrap();let original=scope();for client in 0..32{capture_at(temp.path(),&original.1,&original.2,&request(client,0,"old"),||Ok(())).unwrap();}
  assert!(capture_at(temp.path(),&original.1,&original.2,&request(32,0,"current readable draft"),||Ok(())).unwrap_err().contains("capacity"));let id=format!("{:032x}",0);let action=removal(&held,&id);let first=forget_at(&held,&original,&action,||Ok(())).unwrap();assert_eq!(first["forgotten"],true);assert_eq!(first["source_saved"],false);
  capture_at(temp.path(),&original.1,&original.2,&request(32,0,"current readable draft"),||Ok(())).unwrap();let replacement=request(0,0,"replacement retained text");let another=removal(&held,&format!("{:032x}",1));forget_at(&held,&original,&another,||Ok(())).unwrap();capture_at(temp.path(),&original.1,&original.2,&replacement,||Ok(())).unwrap();
  let repeated=forget_at(&held,&original,&action,||Ok(())).unwrap();assert_eq!(repeated["replayed"],true);let mut changed=removal(&held,&id);changed.request_id=action.request_id.clone();assert!(forget_at(&held,&original,&changed,||Ok(())).is_err());assert_eq!(read_record(&temp.path().join(format!("{id}.json"))).unwrap().history[0].snapshot.buffers[0].text,"replacement retained text");
 }
 #[test]
 fn all_retained_revision_identities_are_exposed_and_changed_record_guards_refuse(){
  assert!(recovery_page_full(1,60*1024*1024,8*1024*1024),"unreadable entries consume the same page budget");assert!(recovery_page_full(8,0,1));assert!(!recovery_page_full(0,0,RECORD_LIMIT as u64+1));
  let temp=tempfile::tempdir().unwrap();let held=RecoveryDirectory::open(temp.path()).unwrap();let original=scope();for revision in 0..5{capture_at(temp.path(),&original.1,&original.2,&request(1,revision,&format!("revision {revision}")),||Ok(())).unwrap();}
  let id=format!("{:032x}",1);let (record,_)=read_observed_record(&temp.path().join(format!("{id}.json"))).unwrap();let rows=revision_rows(&record);assert_eq!(rows.iter().map(|row|row["revision"].as_u64().unwrap()).collect::<Vec<_>>(),vec![3,4,5]);assert_ne!(rows[0]["snapshot_hash"],rows[2]["snapshot_hash"]);
  let action=removal(&held,&id);capture_at(temp.path(),&original.1,&original.2,&request(1,5,"new checkpoint"),||Ok(())).unwrap();assert!(forget_at(&held,&original,&action,||Ok(())).unwrap_err().contains("changed"));assert_eq!(read_record(&temp.path().join(format!("{id}.json"))).unwrap().history.last().unwrap().revision,6);
 }
 #[test]
 fn physical_aba_root_window_and_stale_scope_refuse_without_removing_any_record(){
  let temp=tempfile::tempdir().unwrap();let held=RecoveryDirectory::open(temp.path()).unwrap();let original=scope();let saved=request(1,0,"same bytes");capture_at(temp.path(),&original.1,&original.2,&saved,||Ok(())).unwrap();let action=removal(&held,&saved.client_id);let path=temp.path().join(format!("{}.json",saved.client_id));let bytes=std::fs::read(&path).unwrap();let replacement=temp.path().join("replacement");std::fs::write(&replacement,&bytes).unwrap();std::fs::rename(replacement,&path).unwrap();assert!(forget_at(&held,&original,&action,||Ok(())).is_err());
  let action=removal(&held,&saved.client_id);let mut changed=action.clone();changed.window_slot="1".into();assert!(forget_at(&held,&original,&changed,||Ok(())).is_err());let mut wrong=original.clone();wrong.1="d".repeat(64);assert!(forget_at(&held,&wrong,&action,||Ok(())).is_err());assert!(forget_at(&held,&original,&action,||Err("retired root lease".into())).is_err());assert_eq!(std::fs::read(&path).unwrap(),bytes);
 }
 #[test]
 fn replacement_after_guard_check_is_restored_instead_of_unlinked(){
  let temp=tempfile::tempdir().unwrap();let held=RecoveryDirectory::open(temp.path()).unwrap();let original=scope();let saved=request(1,0,"protected private bytes");capture_at(temp.path(),&original.1,&original.2,&saved,||Ok(())).unwrap();let action=removal(&held,&saved.client_id);let path=temp.path().join(format!("{}.json",saved.client_id));let bytes=std::fs::read(&path).unwrap();let calls=std::cell::Cell::new(0);
  let result=forget_at(&held,&original,&action,||{calls.set(calls.get()+1);if calls.get()==2{let replacement=temp.path().join("replacement");std::fs::write(&replacement,&bytes).map_err(|e|e.to_string())?;std::fs::rename(replacement,&path).map_err(|e|e.to_string())?;}Ok(())});assert!(result.unwrap_err().contains("restored"));assert_eq!(std::fs::read(&path).unwrap(),bytes);assert!(!std::fs::read_dir(temp.path()).unwrap().filter_map(Result::ok).any(|entry|entry.file_name().to_string_lossy().starts_with(".forget-")));
 }
 #[test]
 fn corrupt_bounded_private_record_can_be_explicitly_forgotten_without_adopting_its_contents(){
  let temp=tempfile::tempdir().unwrap();let held=RecoveryDirectory::open(temp.path()).unwrap();let original=scope();let id="a".repeat(32);let path=temp.path().join(format!("{id}.json"));std::fs::write(&path,b"not a checkpoint").unwrap();assert!(read_record(&path).is_err());let action=removal(&held,&id);forget_at(&held,&original,&action,||Ok(())).unwrap();assert!(!path.exists());
 }
 #[test]
 fn held_private_walk_and_remove_refuse_symlinked_or_substituted_storage(){
  use std::os::unix::fs::symlink;let temp=tempfile::tempdir().unwrap();let held=recovery_base(&temp.path().join("private")).unwrap();let outside=temp.path().join("outside");std::fs::create_dir(&outside).unwrap();symlink(&outside,held.path.join("windows")).unwrap();assert!(held.child("windows").is_err());assert!(std::fs::read_dir(&outside).unwrap().next().is_none());
  let child=held.child("records").unwrap();let original=scope();let saved=request(1,0,"retained");capture_at(&child.path,&original.1,&original.2,&saved,||Ok(())).unwrap();let action=removal(&child,&saved.client_id);let old=held.path.join("retired-records");std::fs::rename(&child.path,&old).unwrap();std::fs::create_dir(&child.path).unwrap();assert!(forget_at(&child,&original,&action,||Ok(())).is_err());assert!(old.join(format!("{}.json",saved.client_id)).exists());
 }
}
</ui:copy>

</ui:doc>
