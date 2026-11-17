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
mod archive {
 use super::*;
 use std::{path::{Path,PathBuf},fs::File,io::{Read,Write}};
 const BUDGET:usize=128*1024*1024;
 const MATERIAL:usize=32*1024*1024;
 #[derive(Clone,Serialize,Deserialize)]
 #[serde(deny_unknown_fields)]
 struct Reading {token:String,fingerprint:String,material:String,workspace:String,namespace:String,title:String,origin:String,origin_authority:String,request_id:String,host_application:Option<Value>,retained_at:u64}
 #[derive(Serialize,Deserialize)]
 #[serde(deny_unknown_fields)]
 struct Catalogue {version:u32,physical_root:String,physical_store:String,records:Vec<Reading>}
 #[derive(Serialize,Deserialize)]
 #[serde(deny_unknown_fields)]
 struct Sealed {payload:Catalogue,seal:String}
 fn digest(bytes:&[u8])->String{super::super::workspace_fs::view::hash(bytes)}
 fn seal(key:&[u8],bytes:&[u8])->Result<String,String>{use hmac::{Hmac,Mac};let mut mac=Hmac::<sha2::Sha256>::new_from_slice(key).map_err(|_|"Proposal archive seal unavailable")?;mac.update(bytes);Ok(hex::encode(mac.finalize().into_bytes()))}
 fn verify(key:&[u8],bytes:&[u8],signature:&str)->Result<(),String>{use hmac::{Hmac,Mac};let mut mac=Hmac::<sha2::Sha256>::new_from_slice(key).map_err(|_|"Proposal archive seal unavailable")?;mac.update(bytes);let signature=hex::decode(signature).map_err(|_|"Proposal archive seal is malformed")?;mac.verify_slice(&signature).map_err(|_|"Proposal archive seal does not match. Retained readings were not admitted as live proposals.".into())}
 fn validate_reading(row:&Reading)->Result<(),String>{
  if !super::super::resource_saves::hex(&row.token,32)||row.token.bytes().any(|b|b.is_ascii_uppercase())||row.material.len()>MATERIAL||digest(row.material.as_bytes())!=row.fingerprint{return Err("Proposal archive material or identity is damaged.".into())}
  let material:Value=serde_json::from_str(&row.material).map_err(|_|"Proposal archive material is malformed")?;
  if material["version"]!=1||material["workspace"]!=row.workspace||material["namespace"]!=row.namespace||material["origin_authority"]!=row.origin_authority||material["intention"]["title"]!=row.title||material["intention"]["origin"]!=row.origin||material["intention"]["request_id"]!=row.request_id{return Err("Proposal archive summary does not match its exact original material.".into())}
  if let Some(report)=&row.host_application{if report["token"]!=row.token||report["fingerprint"]!=row.fingerprint||report["workspace"]!=row.workspace||report["namespace"]!=row.namespace||report["saved"]!=false||report["checks_executed"]!=false||report["evidence_authority"]!="host report; not independently observed by the engine"{return Err("Archived host report has no matching original review identity.".into())}}
  Ok(())
 }
 fn historical(row:&Reading,workspace:&str,namespace:&str)->Value{json!({"version":1,"proposal_protocol":1,"token":row.token,"fingerprint":row.fingerprint,"workspace":row.workspace,"namespace":row.namespace,"material":row.material,"summary":{"title":row.title,"origin":row.origin,"origin_authority":row.origin_authority},"historical":true,"request_id":row.request_id,"reading_scope":{"workspace":workspace,"namespace":namespace},"source_effects":false,"checks_executed":false,"host_application":row.host_application,"durability":{"retained":true,"authority":"sealed historical reading only; no live admission or application authority"}})}
 #[cfg(unix)]
 mod unix {
  use super::*;
  use std::{ffi::CString,os::{fd::{AsRawFd,FromRawFd},unix::fs::{MetadataExt,OpenOptionsExt}}};
  static HELD:LazyLock<Mutex<BTreeMap<String,Arc<File>>>>=LazyLock::new(||Mutex::new(BTreeMap::new()));
  fn c(value:&str)->Result<CString,String>{CString::new(value).map_err(|_|"Invalid proposal store component".into())}
  fn same(file:&File,path:&Path)->Result<(),String>{
   let metadata=std::fs::symlink_metadata(path).map_err(|e|e.to_string())?;
   if metadata.file_type().is_symlink()||same_file::Handle::from_file(file.try_clone().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?!=same_file::Handle::from_path(path).map_err(|e|e.to_string())?{return Err("Proposal archive storage was substituted. Preserve live reviews and restore its original private store.".into())}Ok(())
  }
  fn directory(parent:&File,name:&str,create:bool)->Result<File,String>{
   let name=c(name)?;if create&&unsafe{libc::mkdirat(parent.as_raw_fd(),name.as_ptr(),0o700)}!=0{let error=std::io::Error::last_os_error();if error.kind()!=std::io::ErrorKind::AlreadyExists{return Err(format!("Cannot create private proposal archive: {error}"));}}
   let fd=unsafe{libc::openat(parent.as_raw_fd(),name.as_ptr(),libc::O_RDONLY|libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC)};
   if fd<0{return Err(format!("Proposal archive requires ordinary held directories: {}",std::io::Error::last_os_error()));}Ok(unsafe{File::from_raw_fd(fd)})
  }
  fn data_base(desired:&Path)->Result<(File,PathBuf),String>{
   let desired=if desired.is_absolute(){desired.to_path_buf()}else{std::env::current_dir().map_err(|e|e.to_string())?.join(desired)};
   if desired.components().any(|part|matches!(part,std::path::Component::ParentDir)){return Err("Proposal state directory cannot contain parent traversal.".into())}
   let mut ancestor=desired.clone();let mut missing=Vec::new();
   loop{match std::fs::symlink_metadata(&ancestor){Ok(meta)=>{if !meta.is_dir()||meta.file_type().is_symlink(){return Err("Proposal state root must be an ordinary directory.".into())}break},Err(error)if error.kind()==std::io::ErrorKind::NotFound=>{missing.push(ancestor.file_name().ok_or("Proposal state directory has no existing ancestor")?.to_os_string());if !ancestor.pop(){return Err("Proposal state directory has no existing ancestor".into())}},Err(error)=>return Err(error.to_string())}}
   let canonical=ancestor.canonicalize().map_err(|e|e.to_string())?;
   let mut options=std::fs::OpenOptions::new();options.read(true).custom_flags(libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC);
   let mut held=options.open(&canonical).map_err(|e|e.to_string())?;same(&held,&ancestor)?;
   for name in missing.into_iter().rev(){held=directory(&held,name.to_str().ok_or("Proposal store component is not exact UTF-8")?,true)?;}
   same(&held,&desired)?;let path=desired.canonicalize().map_err(|e|e.to_string())?;Ok((held,path))
  }
  pub(super) struct Store {root:PathBuf,root_id:String,base:File,address:PathBuf,base_path:PathBuf,parent:File,parent_path:PathBuf,bucket:Arc<File>,path:PathBuf,store_id:String,key:Vec<u8>,key_file:File,_lock:File}
  impl Store {
   pub(super) fn under(base:&Path,root:&Path)->Result<Self,String>{
    let root_id=resource_inputs::RootIdentity::fingerprint(root)?;let root=root.canonicalize().map_err(|e|e.to_string())?;
    let address=if base.is_absolute(){base.to_path_buf()}else{std::env::current_dir().map_err(|e|e.to_string())?.join(base)};
    let (base,base_path)=data_base(base)?;let parent=directory(&base,"agent-proposal-readings-v1",true)?;let parent_path=base_path.join("agent-proposal-readings-v1");
    let bucket=Arc::new(directory(&parent,&root_id,true)?);let path=parent_path.join(&root_id);
    if parent.metadata().map_err(|e|e.to_string())?.mode()&0o077!=0||bucket.metadata().map_err(|e|e.to_string())?.mode()&0o077!=0{return Err("Proposal archive directories must remain private to this user.".into())}
    same(&parent,&parent_path)?;same(&bucket,&path)?;
    let held_key=format!("{}:{}",address.display(),root_id);let mut stores=HELD.lock().map_err(|_|"Proposal store identity unavailable")?;
    if !stores.contains_key(&held_key)&&stores.len()>=64{return Err("Proposal store identity capacity reached; keep the current live review and retry in its original engine.".into())}
    if let Some(previous)=stores.get(&held_key){if same_file::Handle::from_file(previous.try_clone().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?!=same_file::Handle::from_file(bucket.try_clone().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?{return Err("Original proposal store was replaced; historical recovery requires its original storage.".into())}}else{stores.insert(held_key,bucket.clone());}drop(stores);
    let lock=Self::open_file(&bucket,"archive.lock",libc::O_RDWR|libc::O_CREAT,0o600)?;
    fs2::FileExt::try_lock_exclusive(&lock).map_err(|_|"Another proposal reading is being retained. Keep the live review and retry its reading.".to_string())?;
    let store_id=resource_inputs::RootIdentity::fingerprint(&path)?;
    let key=match Self::read_file(&bucket,"archive.key",32)?{Some(bytes)if bytes.len()==32=>bytes,Some(_)=>return Err("Proposal archive key is damaged; no records were promoted to live authority.".into()),None=>{
     if Self::read_file(&bucket,"readings.json",BUDGET+1024)?.is_some(){return Err("Proposal archive key is missing. Existing readings cannot be verified.".into())}
     let mut key=vec![0;32];getrandom::fill(&mut key).map_err(|e|e.to_string())?;let mut file=Self::open_file(&bucket,"archive.key",libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL,0o600)?;file.write_all(&key).and_then(|_|file.sync_all()).map_err(|e|e.to_string())?;bucket.sync_all().map_err(|e|e.to_string())?;key
    }};
    let key_file=Self::open_file(&bucket,"archive.key",libc::O_RDONLY,0)?;
    let store=Self{root,root_id,base,address,base_path,parent,parent_path,bucket,path,store_id,key,key_file,_lock:lock};store.check()?;Ok(store)
   }
   fn open_file(directory:&File,name:&str,flags:i32,mode:u32)->Result<File,String>{
    let name=c(name)?;let fd=unsafe{libc::openat(directory.as_raw_fd(),name.as_ptr(),flags|libc::O_NOFOLLOW|libc::O_NONBLOCK|libc::O_CLOEXEC,mode)};
    if fd<0{return Err(std::io::Error::last_os_error().to_string())}let file=unsafe{File::from_raw_fd(fd)};let metadata=file.metadata().map_err(|e|e.to_string())?;
    if !metadata.is_file()||metadata.nlink()!=1||metadata.mode()&0o077!=0{return Err("Proposal archive files must be private ordinary files with one identity.".into())}Ok(file)
   }
   fn read_file(directory:&File,name:&str,limit:usize)->Result<Option<Vec<u8>>,String>{
    let name_c=c(name)?;let fd=unsafe{libc::openat(directory.as_raw_fd(),name_c.as_ptr(),libc::O_RDONLY|libc::O_NOFOLLOW|libc::O_NONBLOCK|libc::O_CLOEXEC)};
    if fd<0{let error=std::io::Error::last_os_error();return if error.kind()==std::io::ErrorKind::NotFound{Ok(None)}else{Err(format!("Cannot read private proposal archive: {error}"))}}
    let file=unsafe{File::from_raw_fd(fd)};let metadata=file.metadata().map_err(|e|e.to_string())?;
    if !metadata.is_file()||metadata.nlink()!=1||metadata.mode()&0o077!=0||metadata.len()>limit as u64{return Err("Proposal archive file is not a bounded private ordinary file.".into())}
    let mut bytes=Vec::new();file.take(limit as u64+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;if bytes.len()>limit{return Err("Proposal archive exceeds its reading budget.".into())}Ok(Some(bytes))
   }
   fn check(&self)->Result<(),String>{
    if self.parent.metadata().map_err(|e|e.to_string())?.mode()&0o077!=0||self.bucket.metadata().map_err(|e|e.to_string())?.mode()&0o077!=0{return Err("Proposal archive directory privacy changed while reading or retaining it.".into())}
    if resource_inputs::RootIdentity::fingerprint(&self.root)?!=self.root_id{return Err("Original physical proposal workspace changed; no replacement source was adopted.".into())}
    same(&self.base,&self.address)?;same(&self.base,&self.base_path)?;same(&self.parent,&self.parent_path)?;same(&self.bucket,&self.path)?;same(&self.key_file,&self.path.join("archive.key"))?;same(&self._lock,&self.path.join("archive.lock"))?;
    if Self::read_file(&self.bucket,"archive.key",32)?.as_ref()!=Some(&self.key){return Err("Proposal archive key changed while retaining a reading.".into())}
    if resource_inputs::RootIdentity::fingerprint(&self.path)?!=self.store_id{return Err("Proposal archive physical identity changed.".into())}Ok(())
   }
   fn load(&self)->Result<(Catalogue,Option<Vec<u8>>),String>{
    self.check()?;let bytes=Self::read_file(&self.bucket,"readings.json",BUDGET+1024)?;
    let payload=if let Some(bytes)=&bytes{
     let value:Sealed=serde_json::from_slice(bytes).map_err(|_|"Proposal archive catalogue is damaged")?;let signed=serde_json::to_vec(&value.payload).map_err(|e|e.to_string())?;verify(&self.key,&signed,&value.seal)?;
     if signed.len()>BUDGET||value.payload.version!=1||value.payload.physical_root!=self.root_id||value.payload.physical_store!=self.store_id||value.payload.records.len()>32{return Err("Proposal archive root, store or retention bounds do not match.".into())}
     let mut tokens=BTreeSet::new();for row in &value.payload.records{validate_reading(row)?;if !tokens.insert(row.token.clone()){return Err("Proposal archive repeats an intention identity.".into())}}value.payload
    }else{Catalogue{version:1,physical_root:self.root_id.clone(),physical_store:self.store_id.clone(),records:vec![]}};
    self.check()?;Ok((payload,bytes))
   }
   fn put(&self,row:Reading)->Result<(),String>{
    validate_reading(&row)?;let(mut payload,original)=self.load()?;payload.records.retain(|old|old.token!=row.token);payload.records.push(row);
    while payload.records.len()>32{payload.records.remove(0);}
    let signed=loop{let bytes=serde_json::to_vec(&payload).map_err(|e|e.to_string())?;if bytes.len()<=BUDGET{break bytes}if payload.records.len()<=1{return Err("Complete proposal exceeds its private archive budget.".into())}payload.records.remove(0);};
    let signature=seal(&self.key,&signed)?;let bytes=serde_json::to_vec(&Sealed{payload,seal:signature}).map_err(|e|e.to_string())?;
    let temporary=format!(".{}.tmp",nonce()?);let mut file=Self::open_file(&self.bucket,&temporary,libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL,0o600)?;
    let result=(||{file.write_all(&bytes).and_then(|_|file.sync_all()).map_err(|e|e.to_string())?;self.check()?;if Self::read_file(&self.bucket,"readings.json",BUDGET+1024)?!=original{return Err("Proposal archive changed while retaining this reading.".into())}
     let from=c(&temporary)?;let to=c("readings.json")?;if unsafe{libc::renameat(self.bucket.as_raw_fd(),from.as_ptr(),self.bucket.as_raw_fd(),to.as_ptr())}!=0{return Err(std::io::Error::last_os_error().to_string())}self.bucket.sync_all().map_err(|e|e.to_string())?;self.check()
    })();if result.is_err(){if let Ok(name)=c(&temporary){unsafe{libc::unlinkat(self.bucket.as_raw_fd(),name.as_ptr(),0)};}}result
   }
   pub(super) fn retain(&self,review:&Review,application:Option<&Value>)->Result<(),String>{self.put(Reading{token:review.token.clone(),fingerprint:review.fingerprint.clone(),material:review.material.clone(),workspace:review.workspace.clone(),namespace:review.namespace.clone(),title:review.intention.title.clone(),origin:review.intention.origin.clone(),origin_authority:review.origin_authority.clone(),request_id:review.intention.request_id.clone(),host_application:application.cloned(),retained_at:std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e|e.to_string())?.as_millis().min(u64::MAX as u128)as u64})}
   pub(super) fn readings(&self,workspace:&str,namespace:&str)->Result<Vec<Value>,String>{let(payload,_)=self.load()?;Ok(payload.records.iter().rev().map(|row|historical(row,workspace,namespace)).collect())}
  }
#[cfg(test)]
mod tests {
 use super::*;
 fn review()->Review{
  let intention=super::super::super::tests::intention();let workspace="original-live-runtime".to_string();let namespace="/original/source/address".to_string();let origin_authority="caller claim, not an observed verdict".to_string();
  let material=serde_json::to_string(&json!({"version":1,"intention":intention,"workspace":workspace,"namespace":namespace,"origin_authority":origin_authority})).unwrap();
  Review{token:nonce().unwrap(),fingerprint:digest(material.as_bytes()),material,origin_authority,intention,workspace,namespace,plan:resource_changes::ResourcePlan{version:1,context:"archive-witness".into(),title:"historical reading".into(),patches:vec![],annotations:vec![]}}
 }
 #[test]
 fn standard_keyed_seal_matches_rfc4231_and_refuses_modified_material(){
  assert_eq!(seal(&[0x0b;20],b"Hi There").unwrap(),"b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7");let key=[7;32];let signature=seal(&key,b"original").unwrap();assert!(verify(&key,b"modified",&signature).is_err());
 }
 #[test]
 fn reopen_preserves_exact_original_reading_and_host_claim_without_live_review_authority(){
  let temp=tempfile::tempdir().unwrap();let root=temp.path().join("workspace");std::fs::create_dir(&root).unwrap();let base=temp.path().join("private");let row=review();
  let report=json!({"token":row.token,"fingerprint":row.fingerprint,"workspace":row.workspace,"namespace":row.namespace,"status":"host-reported draft application","evidence_authority":"host report; not independently observed by the engine","saved":false,"checks_executed":false});
  let store=Store::under(&base,&root).unwrap();store.retain(&row,Some(&report)).unwrap();assert_eq!(store.bucket.metadata().unwrap().mode()&0o777,0o700);assert_eq!(std::fs::metadata(store.path.join("readings.json")).unwrap().mode()&0o777,0o600);drop(store);
  let store=Store::under(&base,&root).unwrap();let readings=store.readings("new-runtime","/current/address").unwrap();assert_eq!(readings.len(),1);let reading=&readings[0];assert_eq!(reading["material"],row.material);assert_eq!(reading["workspace"],row.workspace);assert_eq!(reading["namespace"],row.namespace);assert_eq!(reading["fingerprint"],row.fingerprint);assert_eq!(reading["reading_scope"],json!({"workspace":"new-runtime","namespace":"/current/address"}));assert_eq!(reading["historical"],true);assert_eq!(reading["host_application"],report);assert!(!REVIEWS.lock().unwrap().records.contains_key(&row.token));assert!(std::fs::read_dir(root).unwrap().next().is_none());std::fs::set_permissions(&store.path,<std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o755)).unwrap();assert!(store.readings("new-runtime","/current/address").is_err());
 }
 #[test]
 fn archive_retention_is_bounded_and_damaged_material_never_replaces_verified_content(){
  let temp=tempfile::tempdir().unwrap();let root=temp.path().join("workspace");std::fs::create_dir(&root).unwrap();let store=Store::under(&temp.path().join("private"),&root).unwrap();let first=review();store.retain(&first,None).unwrap();
  for _ in 0..32{store.retain(&review(),None).unwrap();}let rows=store.readings("new","namespace").unwrap();assert_eq!(rows.len(),32);assert!(!rows.iter().any(|row|row["token"]==first.token));let omitted=rows.last().unwrap()["token"].as_str().unwrap().to_owned();let(catalogue,notice)=super::super::super::merge_proposal_catalogue(vec![json!({"token":"current-live","historical":false})],Ok(rows));assert_eq!(catalogue.len(),32);assert!(notice.is_some());assert!(!catalogue.iter().any(|row|row["token"]==omitted));assert!(store.readings("new","namespace").unwrap().iter().any(|row|row["token"]==omitted));
  let mut damaged=review();damaged.material.push(' ');let bytes=std::fs::read(store.path.join("readings.json")).unwrap();assert!(store.retain(&damaged,None).is_err());assert_eq!(std::fs::read(store.path.join("readings.json")).unwrap(),bytes);
  let mut catalog:Value=serde_json::from_slice(&bytes).unwrap();catalog["payload"]["records"][0]["material"]=json!("tampered");std::fs::write(store.path.join("readings.json"),serde_json::to_vec(&catalog).unwrap()).unwrap();assert!(store.readings("new","namespace").is_err());
 }
 #[test]
 fn symlink_hardlink_missing_key_and_substituted_store_refuse(){
  use std::os::unix::fs::symlink;
  let temp=tempfile::tempdir().unwrap();let root=temp.path().join("workspace");std::fs::create_dir(&root).unwrap();let base=temp.path().join("private");let store=Store::under(&base,&root).unwrap();store.retain(&review(),None).unwrap();let path=store.path.clone();drop(store);
  let record=path.join("readings.json");let original=path.join("original.json");std::fs::rename(&record,&original).unwrap();symlink(&original,&record).unwrap();assert!(Store::under(&base,&root).unwrap().readings("new","namespace").is_err());std::fs::remove_file(&record).unwrap();std::fs::rename(&original,&record).unwrap();
  let linked=path.join("key-alias");std::fs::hard_link(path.join("archive.key"),&linked).unwrap();assert!(Store::under(&base,&root).is_err());std::fs::remove_file(linked).unwrap();
  let key=std::fs::read(path.join("archive.key")).unwrap();std::fs::remove_file(path.join("archive.key")).unwrap();assert!(Store::under(&base,&root).is_err());{let mut file=Store::open_file(&File::open(&path).unwrap(),"archive.key",libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL,0o600).unwrap();file.write_all(&key).unwrap();}
  let retired=path.with_file_name("retired-original-store");std::fs::rename(&path,&retired).unwrap();std::fs::create_dir(&path).unwrap();std::fs::set_permissions(&path,<std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o700)).unwrap();std::fs::copy(retired.join("archive.key"),path.join("archive.key")).unwrap();std::fs::copy(retired.join("readings.json"),path.join("readings.json")).unwrap();assert!(Store::under(&base,&root).is_err());
 }
 #[test]
 fn substituted_root_and_state_symlink_cannot_borrow_archived_identity(){
  use std::os::unix::fs::symlink;
  let temp=tempfile::tempdir().unwrap();let root=temp.path().join("workspace");std::fs::create_dir(&root).unwrap();let base=temp.path().join("private");let store=Store::under(&base,&root).unwrap();store.retain(&review(),None).unwrap();std::fs::rename(&root,temp.path().join("retired-root")).unwrap();std::fs::create_dir(&root).unwrap();assert!(store.readings("new","namespace").is_err());drop(store);assert!(Store::under(&base,&root).unwrap().readings("new","namespace").unwrap().is_empty());
  let linked=temp.path().join("linked-state");symlink(&base,&linked).unwrap();assert!(Store::under(&linked,&root).is_err());
 }
}
  pub(super) fn retain(state:&LocalState,review:&Review,application:Option<&Value>)->Result<(),String>{let _ordinary=crate::engine::private_policy::ordinary();ordinary(state).map_err(|e|e.message().to_string())?;state.resource_root.check(state.index.root())?;let store=Store::under(&hickory_workspace::data_root().map_err(|e|e.to_string())?,state.index.root())?;ordinary(state).map_err(|e|e.message().to_string())?;store.retain(review,application)?;ordinary(state).map_err(|e|e.message().to_string())?;state.resource_root.check(state.index.root())?;Ok(())}
  pub(super) fn readings(state:&LocalState,workspace:&str,namespace:&str)->Result<Vec<Value>,String>{let _ordinary=crate::engine::private_policy::ordinary();ordinary(state).map_err(|e|e.message().to_string())?;state.resource_root.check(state.index.root())?;let store=Store::under(&hickory_workspace::data_root().map_err(|e|e.to_string())?,state.index.root())?;let rows=store.readings(workspace,namespace)?;ordinary(state).map_err(|e|e.message().to_string())?;state.resource_root.check(state.index.root())?;Ok(rows)}
 }
 pub(super) fn retain(state:&LocalState,review:&Review,application:Option<&Value>)->Result<(),String>{#[cfg(unix)]{unix::retain(state,review,application)}#[cfg(not(unix))]{let _=(state,review,application);Err("Private held-directory proposal persistence is unavailable on this engine platform; current live review remains available.".into())}}
 pub(super) fn readings(state:&LocalState,workspace:&str,namespace:&str)->Result<Vec<Value>,String>{#[cfg(unix)]{unix::readings(state,workspace,namespace)}#[cfg(not(unix))]{let _=(state,workspace,namespace);Err("Private historical proposal readings are unavailable on this engine platform.".into())}}
}
async fn archive_review(state:&LocalState,review:Arc<Review>,application:Option<Value>)->Value{
 let worker=state.clone();let result=tokio::task::spawn_blocking(move||archive::retain(&worker,&review,application.as_ref())).await;
 match result{Ok(Ok(()))=>json!({"retained":true,"authority":"private sealed historical reading; live admission remains process-local"}),Ok(Err(error))=>json!({"retained":false,"notice":format!("Current live review remains available; historical retention failed: {error}")}),Err(error)=>json!({"retained":false,"notice":format!("Current live review remains available; historical retention worker failed: {error}")})}
}
async fn archive_readings(state:&LocalState,workspace:&str,namespace:&str)->Result<Vec<Value>,String>{
 let worker=state.clone();let workspace=workspace.to_owned();let namespace=namespace.to_owned();tokio::task::spawn_blocking(move||archive::readings(&worker,&workspace,&namespace)).await.map_err(|error|error.to_string())?
}

fn merge_proposal_catalogue(mut reviews:Vec<Value>,archived:Result<Vec<Value>,String>)->(Vec<Value>,Option<String>){
 let mut omitted=reviews.len().saturating_sub(32);reviews.truncate(32);
 let mut tokens:BTreeSet<String>=reviews.iter().filter_map(|row|row["token"].as_str().map(str::to_owned)).collect();
 let failure=match archived{Ok(rows)=>{for row in rows{if let Some(token)=row["token"].as_str(){if tokens.insert(token.into()){
  if reviews.len()==32{omitted+=1;continue;}
  reviews.push(json!({"token":row["token"],"fingerprint":row["fingerprint"],"title":row["summary"]["title"],"origin":row["summary"]["origin"],"origin_authority":row["summary"]["origin_authority"],"request_id":row["request_id"],"historical":true,"applied":false,"saved":false,"host_application":row["host_application"],"durability":row["durability"]}));
 }}}None},Err(error)=>Some(format!("Historical proposal readings unavailable: {error}. Current live reviews remain available."))};
 let notice=if omitted>0{let message=format!("Showing current live reviews first within the 32-reading catalogue; {omitted} historical readings were omitted. Retained historical tokens remain readable.");Some(failure.map_or(message.clone(),|failure|format!("{failure} {message}")))}else{failure};
 (reviews,notice)
}
#[cfg(test)]
mod archive_catalogue_tests {
 use super::*;
 #[test]
 fn current_live_reviews_remain_usable_when_archive_snapshot_adds_another_full_catalogue(){
  let live=(0..32).map(|n|json!({"token":format!("live-{n}"),"historical":false})).collect::<Vec<_>>();let old=(0..32).map(|n|json!({"token":format!("old-{n}"),"summary":{"title":"Earlier historical claim"}})).collect();
  let(rows,notice)=merge_proposal_catalogue(live.clone(),Ok(old));assert_eq!(rows,live);assert!(notice.unwrap().contains("32 historical readings were omitted"));
  let(rows,notice)=merge_proposal_catalogue(live.clone(),Err("damaged private catalogue".into()));assert_eq!(rows,live);assert!(notice.unwrap().contains("unavailable"));
 }
 #[test]
 fn catalogue_deduplicates_live_rows_and_bounds_remaining_historical_capacity(){
  let live=vec![json!({"token":"same","historical":false}),json!({"token":"live","historical":false})];let mut old=vec![json!({"token":"same","summary":{"title":"Duplicate older row"}})];old.extend((0..32).map(|n|json!({"token":format!("old-{n}"),"summary":{"title":"Historical claim"}})));
  let(rows,notice)=merge_proposal_catalogue(live.clone(),Ok(old));assert_eq!(rows.len(),32);assert_eq!(&rows[..2],live.as_slice());assert!(rows[2..].iter().all(|row|row["historical"]==true));assert!(notice.unwrap().contains("2 historical readings were omitted"));
 }
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
 let material=serde_json::to_string(&Material{version:1,intention,origin_authority,workspace:&inputs.workspace,namespace:&namespace,inputs:&inputs,plan:&plan,consequences:&consequences,before:&before,after:&after,saved_guards:&saved_guards}).map_err(|error|ApiError::internal(error.to_string()))?;
 let fingerprint=super::workspace_fs::view::hash(material.as_bytes());
 Ok(Review{token:String::new(),fingerprint,material,origin_authority:origin_authority.into(),intention:intention.clone(),workspace:inputs.workspace.clone(),namespace,plan})
}
pub async fn prepare(State(state):State<LocalState>,Json(intention):Json<Intention>)->ApiResult<Json<Value>>{prepare_with_origin(state,intention,"caller-provided intention; not an agent attestation").await}
async fn prepare_with_origin(state:LocalState,intention:Intention,origin_authority:&str)->ApiResult<Json<Value>>{
 let mut review=analyze(&state,&intention,origin_authority).await?;review.token=nonce().map_err(ApiError::internal)?;
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
 let fresh=analyze(&state,&held.intention,&held.origin_authority).await?;if fresh.fingerprint!=held.fingerprint{return Err(ApiError::conflict("Proposal source, ownership, generated consequences or evidence changed. No partial Apply was admitted."))}
 Ok(Json(json!({"version":1,"proposal_protocol":1,"token":body.token,"fingerprint":body.fingerprint,"workspace":fresh.workspace,"namespace":fresh.namespace,"admitted":true,"source_effects":false,"checks_executed":false,"authority":"current engine observation; host editor lease still required"})))
}
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
 prepare_with_origin(state,intention,if live{"engine-managed agent tool; live editor capture at preparation; claims remain unverified"}else{"engine-managed agent tool; non-live submitted editor snapshot; claims remain unverified"}).await
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
#[cfg(test)]
mod tests {
 use super::*;
 pub(super) fn intention()->Intention{serde_json::from_value(json!({"version":1,"request_id":"0123456789abcdef0123456789abcdef","origin":"owned provider fixture","title":"Change note","explanation":"Keep the change and its reasoning together.","claims":["Expected behavior; not an observed verdict"],"check_ids":[],"capture":{"context":"context","edit":{"changes":{}},"drafts":{},"ownership_sources":{},"project_entry":null},"reading":{"version":1,"entry":"project.md","sources":{},"missing_sources":[],"focus":null}})).unwrap()}
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
#[cfg(test)] pub(super) async fn assert_private_refused(state:&LocalState){
 let value=tests::intention();let token="0123456789abcdef0123456789abcdef".to_string();
 let actions=[prepare(State(state.clone()),Json(value.clone())).await,validate(State(state.clone()),Json(Admission{version:1,token:token.clone(),fingerprint:"unknown".into()})).await,list(State(state.clone())).await,read(State(state.clone()),axum::extract::Path(token.clone())).await,report(State(state.clone()),Json(HostReport{version:1,token,fingerprint:"unknown".into(),workspace:"unknown".into(),namespace:"unknown".into(),request_id:"0123456789abcdef0123456789abcdef".into(),host:"private refusal witness".into(),after_hashes:BTreeMap::new()})).await,from_agent(state.clone(),&super::acp::edits::Edits::default(),"private".into(),&Value::Null).await];
 for action in actions{match action{Ok(_)=>panic!("Ordinary proposal entry admitted private prediction state"),Err(error)=>assert!(error.message().contains("Private prediction"),"{}",error.message())}}
}
