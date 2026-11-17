<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep proposal readings without reviving application authority

For maintainers retaining exact agent reasoning across an engine restart.

A live proposal is an engine capability with original source observations. Its archive is only a historical reading: retain exact sealed material, original scope and any explicitly unverified host report, then attach the current reading scope when reopened. Never deserialize an archive into `Review`, re-run its plan, or populate live admission state. Private prediction requests never enter this ordinary store.

## Seal one bounded private catalogue under held physical identities

Use the existing noncreating user-state path resolver. Create private storage only through held ordinary directories and keep the source root and bucket birth identities in the keyed catalogue seal. A substituted bucket, symlink, damaged record or missing key refuses; no same-named source is borrowed. Atomic catalogue replacement preserves the previous complete readings on a failed write. Keep at most32 readings and128MiB of signed catalogue data (old plus staged replacement is bounded transient storage). Persistence failures leave the in-process review usable with an explicit durability failure.

<ui:copy id="agent-proposal-archive">
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
</ui:copy>

## Verify restart and hostile storage without manufacturing current authority

The tests reopen the store through a fresh handle, check the original material and explicitly unverified host receipt, and prove no archive load inserts its token into the live registry. They also exercise the retention bound, keyed-seal known vector, tampered material, symlinks, hardlinks, missing keys and substituted roots or stores. Invalid archives leave source untouched and provide no substitute admission.

<ui:copy id="agent-proposal-archive-witness">
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
</ui:copy>

## Keep ordinary process isolation around every archive operation

A blocking archive worker holds the existing ordinary-operation lease, so private prediction startup must drain it before publishing private context. Recheck the ordinary policy and source root before returning any reading. Unsupported engine platforms and failed storage return explicit unavailability instead of a pretend durable success.

<ui:copy id="agent-proposal-archive-api">
  pub(super) fn retain(state:&LocalState,review:&Review,application:Option<&Value>)->Result<(),String>{let _ordinary=crate::engine::private_policy::ordinary();ordinary(state).map_err(|e|e.message().to_string())?;state.resource_root.check(state.index.root())?;let store=Store::under(&hickory_workspace::data_root().map_err(|e|e.to_string())?,state.index.root())?;ordinary(state).map_err(|e|e.message().to_string())?;store.retain(review,application)?;ordinary(state).map_err(|e|e.message().to_string())?;state.resource_root.check(state.index.root())?;Ok(())}
  pub(super) fn readings(state:&LocalState,workspace:&str,namespace:&str)->Result<Vec<Value>,String>{let _ordinary=crate::engine::private_policy::ordinary();ordinary(state).map_err(|e|e.message().to_string())?;state.resource_root.check(state.index.root())?;let store=Store::under(&hickory_workspace::data_root().map_err(|e|e.to_string())?,state.index.root())?;let rows=store.readings(workspace,namespace)?;ordinary(state).map_err(|e|e.message().to_string())?;state.resource_root.check(state.index.root())?;Ok(rows)}
 }
 pub(super) fn retain(state:&LocalState,review:&Review,application:Option<&Value>)->Result<(),String>{#[cfg(unix)]{unix::retain(state,review,application)}#[cfg(not(unix))]{let _=(state,review,application);Err("Private held-directory proposal persistence is unavailable on this engine platform; current live review remains available.".into())}}
 pub(super) fn readings(state:&LocalState,workspace:&str,namespace:&str)->Result<Vec<Value>,String>{#[cfg(unix)]{unix::readings(state,workspace,namespace)}#[cfg(not(unix))]{let _=(state,workspace,namespace);Err("Private historical proposal readings are unavailable on this engine platform.".into())}}
}
</ui:copy>

## Attach durability to the existing live routes without blocking unrelated source reading

Archive I/O runs on blocking workers. A failed archive write does not pretend a source change failed or discard the live review: its envelope reports the retention failure. A damaged historical catalogue yields no historical rows and an explicit list notice. Limit the merged catalogue to32 rows, preferring current live reviews; asynchronous archive snapshots cannot make the host reject all usable live rows. Omitted historical tokens remain independently readable and a retention notice names that bounded omission. Reading a valid old envelope does not create a new live review or allow a new host report.

<ui:copy id="agent-proposal-archive-routes">
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
</ui:copy>

## Follow the usable historical list and reading routes after a real restart

The owned engine witness prepares a live review and records a host claim without changing source, stops the process, then reopens the same physical workspace and private state. Its original material remains readable with a current reading scope, while both admission and new reports refuse the archived token. A new live proposal remains independent. Corrupting the catalogue yields an explicit historical refusal and truthful live-only retention rather than false durable success.

<ui:copy id="agent-proposal-persistence-protocol">
#!/usr/bin/env python3
"""Owned real-engine historical proposal reading, restart refusal and storage-failure isolation."""
import hashlib,importlib.util,json,os,tempfile,time,urllib.error,urllib.request,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
FILES={'project.md':'\x3ch:doc xmlns:h="http://www.hickorydocs.com/1.0">\x3ch:project sources="construction.md"/>\x3c/h:doc>\n','construction.md':'\x3ch:doc xmlns:h="http://www.hickorydocs.com/1.0">\x3ch:file path="generated.py">generated = 1\n\x3c/h:file>\x3c/h:doc>\n','generated.py':'generated = 1\n','note.py':'value = 1\n'}
def call(origin,method,route,body=None):
 request=urllib.request.Request(origin+route,data=None if body is None else json.dumps(body).encode(),method=method,headers={'Content-Type':'application/json'})
 with urllib.request.urlopen(request,timeout=90) as response:return json.load(response)
def refused(action):
 try:action()
 except urllib.error.HTTPError as error:assert error.code in (400,403,404,409,422),(error.code,error.read());return
 raise AssertionError('Historical or tampered proposal was admitted')
def intention():
 return {'version':1,'request_id':uuid.uuid4().hex,'origin':'owned restart fixture','title':'Retain an exact historical intention','explanation':'The expected transition is an agent claim. Restarted readings carry no admission authority.','claims':['This claim was not verified by an executed check.'],'check_ids':[],'capture':{'context':'owned-restart','project_entry':'project.md','drafts':{},'ownership_sources':{},'unaddressed_sources':{},'saved_guards':{},'edit':{'changes':{'hick:///note.py':[{'range':{'start':{'line':0,'character':0},'end':{'line':1,'character':0}},'newText':'value = 41\n'}]}}},'reading':{'version':1,'entry':'project.md','sources':{},'missing_sources':[],'focus':'note.py'}}
def stop_engine(fixture,process,state):
 # Closing a window proxy does not itself end the shared engine process.
 fixture.stop(process);endpoint=Path(state)/'engine/endpoint.json';deadline=time.monotonic()+30
 while endpoint.exists() and time.monotonic()<deadline:time.sleep(.1)
 assert not endpoint.exists(),'Owned engine did not finish its lease shutdown; a true engine restart was not established'
def main():
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-proposal-history-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();state=folder/'private-state'
  files=FILES
  for path,text in files.items():(root/path).write_text(text)
  def start():return fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(state)})
  engine=start()
  try:
   origin=fixture.origin(engine);review=call(origin,'POST','/api/agent-proposals/prepare',intention());assert review['historical'] is False and review['durability']['retained'] is True,review.get('durability');assert hashlib.sha256(review['material'].encode()).hexdigest()==review['fingerprint']
   admission={'version':1,'token':review['token'],'fingerprint':review['fingerprint']};assert call(origin,'POST','/api/agent-proposals/validate',admission)['admitted'] is True
   report={'version':1,'token':review['token'],'fingerprint':review['fingerprint'],'workspace':review['workspace'],'namespace':str(root),'request_id':uuid.uuid4().hex,'host':'owned fixture host claim, not an observed draft','after_hashes':{'note.py':hashlib.sha256(b'value = 41\n').hexdigest()}}
   claimed=call(origin,'POST','/api/agent-proposals/report',report);assert claimed['evidence_authority']=='host report; not independently observed by the engine';assert claimed['saved'] is False and claimed['checks_executed'] is False
   assert call(origin,'GET','/api/agent-proposals/'+review['token'])['host_application']==claimed
  finally:stop_engine(fixture,engine,state)
  engine=start()
  try:
   origin=fixture.origin(engine);listing=call(origin,'GET','/api/agent-proposals');row=next(row for row in listing['reviews'] if row['token']==review['token']);assert row['historical'] is True and listing['workspace']!=review['workspace'];assert listing['retention_notice'] is None
   historical=call(origin,'GET','/api/agent-proposals/'+review['token']);assert historical['historical'] is True;assert historical['material']==review['material'];assert historical['fingerprint']==review['fingerprint'];assert historical['workspace']==review['workspace'] and historical['namespace']==review['namespace'];assert historical['reading_scope']=={'workspace':listing['workspace'],'namespace':listing['namespace']};assert historical['host_application']==claimed;assert historical['source_effects'] is False and historical['checks_executed'] is False
   refused(lambda:call(origin,'POST','/api/agent-proposals/validate',admission));refused(lambda:call(origin,'POST','/api/agent-proposals/report',report));refused(lambda:call(origin,'POST','/api/agent-proposals/report',{**report,'workspace':listing['workspace'],'request_id':uuid.uuid4().hex}))
   fresh=call(origin,'POST','/api/agent-proposals/prepare',intention());assert fresh['historical'] is False and fresh['token']!=review['token'];assert call(origin,'POST','/api/agent-proposals/validate',{'version':1,'token':fresh['token'],'fingerprint':fresh['fingerprint']})['admitted'] is True
  finally:stop_engine(fixture,engine,state)
  catalogues=list(state.rglob('readings.json'));assert len(catalogues)==1;catalogue=json.loads(catalogues[0].read_text());catalogue['payload']['records'][0]['material']+='forged';catalogues[0].write_text(json.dumps(catalogue))
  engine=start()
  try:
   origin=fixture.origin(engine);listing=call(origin,'GET','/api/agent-proposals');assert listing['reviews']==[] and 'unavailable' in listing['retention_notice'].lower();refused(lambda:call(origin,'GET','/api/agent-proposals/'+review['token']));refused(lambda:call(origin,'POST','/api/agent-proposals/validate',admission))
   live=call(origin,'POST','/api/agent-proposals/prepare',intention());assert live['historical'] is False and live['durability']['retained'] is False;assert 'failed' in live['durability']['notice'].lower();assert call(origin,'GET','/api/agent-proposals/'+live['token'])==live;assert call(origin,'POST','/api/agent-proposals/validate',{'version':1,'token':live['token'],'fingerprint':live['fingerprint']})['admitted'] is True
  finally:stop_engine(fixture,engine,state)
  for path,text in files.items():assert (root/path).read_text()==text,path
  assert not (root/'.hick-cache').exists()
  print('PASS: real-engine live preparation/report, restart historical list/read with exact original material and current reading scope, no archived admission/report or automatic source effect, fresh live proposal independently admitted, tampered archive refusal with truthful live-only durability failure, original source bytes retained. Host reports remain unverified claims; physical/accessibility input not exercised.')
if __name__=='__main__':main()
</ui:copy>

<ui:file path="scripts/test-agent-proposal-persistence.py"><ui:paste select="#agent-proposal-persistence-protocol"/></ui:file>

</ui:doc>
