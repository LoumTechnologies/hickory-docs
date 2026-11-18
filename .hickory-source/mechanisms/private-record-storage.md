<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Hold private records without granting source authority

For maintainers sharing durable private storage between historical proposal readings and workspace-entry policy. This layer retains bounded bytes and proves physical storage identity; callers still validate their own payload, namespace and authority. No record is promoted into live admission here.

## Separate record policy from seal bytes

Internal constant specifications select one namespace and fixed filenames, never request-controlled paths. Legacy proposal HMAC bytes remain unchanged when the domain is absent; new policies use a domain so a valid record cannot be borrowed by another purpose.

<ui:copy id="private-record-specification">
//! Held private records; policy and live authority remain in their owning services.
use std::{collections::BTreeMap,fs::File,path::{Path,PathBuf},io::{Read,Write},sync::{Arc,LazyLock,Mutex}};
use super::resource_inputs;
#[derive(Clone,Copy)]
pub(crate) struct Spec {pub namespace:&'static str,pub key:&'static str,pub lock:&'static str,pub record:&'static str,pub limit:usize,pub physical_bucket:bool}
pub(crate) fn seal(key:&[u8],domain:Option<&[u8]>,bytes:&[u8])->Result<String,String>{use hmac::{Hmac,Mac};let mut mac=Hmac::<sha2::Sha256>::new_from_slice(key).map_err(|_|"Private record seal unavailable")?;if let Some(domain)=domain{mac.update(&(domain.len() as u64).to_be_bytes());mac.update(domain);}mac.update(bytes);Ok(hex::encode(mac.finalize().into_bytes()))}
pub(crate) fn verify(key:&[u8],domain:Option<&[u8]>,bytes:&[u8],signature:&str)->Result<(),String>{use hmac::{Hmac,Mac};let mut mac=Hmac::<sha2::Sha256>::new_from_slice(key).map_err(|_|"Private record seal unavailable")?;if let Some(domain)=domain{mac.update(&(domain.len() as u64).to_be_bytes());mac.update(domain);}mac.update(bytes);mac.verify_slice(&hex::decode(signature).map_err(|_|"Private record seal is malformed")?).map_err(|_|"Private record seal does not match its original material.".into())}
</ui:copy>

## Keep held identity through reading and replacement

Open every component without following symlinks, retain the original bucket handle, and reject replacement, aliases or public file permissions. Address-bound buckets preserve the proposal archive format. Physical buckets find a workspace after a folder rename, but the separate full root fingerprint still changes: callers must refuse the old namespace rather than treating retained bytes as new authority.

Opening a shared snapshot cannot initialize missing storage or replace record metadata. Exclusive replacement compares the original bytes and synchronizes the complete temporary file before its atomic publication. A separate nonblocking publication lease allows callers to hold source-write policy stable without keeping a metadata writer lock across unrelated reads.

<ui:copy id="private-record-held-storage">
#[cfg(unix)]
mod unix {
 use super::*;
 use std::{ffi::CString,os::{fd::{AsRawFd,FromRawFd},unix::fs::{MetadataExt,OpenOptionsExt}}};
fn physical_key(root:&Path)->Result<String,String>{let file=std::fs::OpenOptions::new().read(true).custom_flags(libc::O_RDONLY|libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC).open(root).map_err(|error|error.to_string())?;let metadata=file.metadata().map_err(|error|error.to_string())?;let birth=metadata.created().map_err(|_|"Persistent physical workspace identity is unavailable")?.duration_since(std::time::UNIX_EPOCH).map_err(|error|error.to_string())?.as_nanos();Ok(super::super::workspace_fs::view::hash(&serde_json::to_vec(&(metadata.dev(),metadata.ino(),birth)).map_err(|error|error.to_string())?))}
  static HELD_PUBLICATION:LazyLock<Mutex<BTreeMap<PathBuf,Arc<File>>>>=LazyLock::new(||Mutex::new(BTreeMap::new()));
  static HELD:LazyLock<Mutex<BTreeMap<(PathBuf,&'static str,String),Arc<File>>>>=LazyLock::new(||Mutex::new(BTreeMap::new()));
  fn c(value:&str)->Result<CString,String>{CString::new(value).map_err(|_|"Invalid private store component".into())}
  fn same(file:&File,path:&Path)->Result<(),String>{
   let metadata=std::fs::symlink_metadata(path).map_err(|e|e.to_string())?;
   if metadata.file_type().is_symlink()||same_file::Handle::from_file(file.try_clone().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?!=same_file::Handle::from_path(path).map_err(|e|e.to_string())?{return Err("Private record storage was substituted. Preserve live reviews and restore its original private store.".into())}Ok(())
  }
  fn directory(parent:&File,name:&str,create:bool)->Result<File,String>{
   let name=c(name)?;if create&&unsafe{libc::mkdirat(parent.as_raw_fd(),name.as_ptr(),0o700)}!=0{let error=std::io::Error::last_os_error();if error.kind()!=std::io::ErrorKind::AlreadyExists{return Err(format!("Cannot create private record: {error}"));}}
   let fd=unsafe{libc::openat(parent.as_raw_fd(),name.as_ptr(),libc::O_RDONLY|libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC)};
   if fd<0{return Err(format!("Private record requires ordinary held directories: {}",std::io::Error::last_os_error()));}Ok(unsafe{File::from_raw_fd(fd)})
  }
  fn data_base(desired:&Path,create:bool)->Result<(File,PathBuf),String>{
   let desired=if desired.is_absolute(){desired.to_path_buf()}else{std::env::current_dir().map_err(|e|e.to_string())?.join(desired)};
   if desired.components().any(|part|matches!(part,std::path::Component::ParentDir)){return Err("Private state directory cannot contain parent traversal.".into())}
   let mut ancestor=desired.clone();let mut missing=Vec::new();
   loop{match std::fs::symlink_metadata(&ancestor){Ok(meta)=>{if !meta.is_dir()||meta.file_type().is_symlink(){return Err("Private state root must be an ordinary directory.".into())}break},Err(error)if error.kind()==std::io::ErrorKind::NotFound=>{missing.push(ancestor.file_name().ok_or("Private state directory has no existing ancestor")?.to_os_string());if !ancestor.pop(){return Err("Private state directory has no existing ancestor".into())}},Err(error)=>return Err(error.to_string())}}
   if !create&&!missing.is_empty(){return Err("Private record storage has not been initialized.".into())}
   let canonical=ancestor.canonicalize().map_err(|e|e.to_string())?;
   let mut options=std::fs::OpenOptions::new();options.read(true).custom_flags(libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC);
   let mut held=options.open(&canonical).map_err(|e|e.to_string())?;same(&held,&ancestor)?;
   for name in missing.into_iter().rev(){held=directory(&held,name.to_str().ok_or("Private store component is not exact UTF-8")?,true)?;}
   same(&held,&desired)?;let path=desired.canonicalize().map_err(|e|e.to_string())?;Ok((held,path))
  }
// Insert beside data_base: absence is a typed observation, never an error string.
fn optional_base_exists(desired:&Path)->Result<bool,String>{
 let desired=if desired.is_absolute(){desired.to_path_buf()}else{std::env::current_dir().map_err(|error|error.to_string())?.join(desired)};
 if desired.components().any(|part|matches!(part,std::path::Component::ParentDir)){return Err("Private state directory cannot contain parent traversal.".into())}
 let mut ancestor=desired.clone();let mut missing=false;
 loop{match std::fs::symlink_metadata(&ancestor){Ok(metadata)=>{if !metadata.is_dir()||metadata.file_type().is_symlink(){return Err("Private state ancestor is not an ordinary directory.".into())}break},Err(error)if error.kind()==std::io::ErrorKind::NotFound=>{missing=true;if !ancestor.pop(){return Err("Private state directory has no existing ordinary ancestor.".into())}},Err(error)=>return Err(error.to_string())}}
 let canonical=ancestor.canonicalize().map_err(|error|error.to_string())?;let held=std::fs::OpenOptions::new().read(true).custom_flags(libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC).open(&canonical).map_err(|error|error.to_string())?;same(&held,&ancestor)?;Ok(!missing)
}
fn optional_directory(parent:&File,name:&str)->Result<Option<File>,String>{let name=c(name)?;let descriptor=unsafe{libc::openat(parent.as_raw_fd(),name.as_ptr(),libc::O_RDONLY|libc::O_DIRECTORY|libc::O_NOFOLLOW|libc::O_CLOEXEC)};if descriptor<0{let error=std::io::Error::last_os_error();if error.kind()==std::io::ErrorKind::NotFound{return Ok(None)}return Err(error.to_string())}Ok(Some(unsafe{File::from_raw_fd(descriptor)}))}
  pub(crate) struct HeldRecord {root:PathBuf,pub(crate) root_id:String,base:File,address:PathBuf,base_path:PathBuf,parent:File,parent_path:PathBuf,pub(crate) bucket:Arc<File>,pub(crate) path:PathBuf,pub(crate) store_id:String,pub(crate) key:Vec<u8>,pub(crate) created_key:bool,exclusive:bool,spec:Spec,key_file:File,_lock:File}
  impl HeldRecord {
   pub(crate) fn under(base:&Path,root:&Path,spec:Spec)->Result<Self,String>{Self::open(base,root,spec,true)}
   pub(crate) fn snapshot(base:&Path,root:&Path,spec:Spec)->Result<Self,String>{Self::open(base,root,spec,false)}
   pub(crate) fn try_snapshot(base:&Path,root:&Path,spec:Spec)->Result<Option<Self>,String>{
    for value in [spec.namespace,spec.key,spec.lock,spec.record]{if value.is_empty()||Path::new(value).components().count()!=1||Path::new(value).components().any(|part|!matches!(part,std::path::Component::Normal(_))){return Err("Private record specification is not a single component".into())}}
    let root_id=resource_inputs::RootIdentity::fingerprint(root)?;let address=if base.is_absolute(){base.to_path_buf()}else{std::env::current_dir().map_err(|error|error.to_string())?.join(base)};
    let bucket_id=if spec.physical_bucket{physical_key(root)?}else{root_id};let key=(address,spec.namespace,bucket_id.clone());
    let known=HELD.lock().map_err(|_|"Original private store identity is unavailable")?.contains_key(&key);
    if !optional_base_exists(base)?{return if known{Err("Original private storage disappeared; absence cannot initialize a replacement.".into())}else{Ok(None)}}
    let(base_handle,_)=data_base(base,false)?;
    let Some(parent)=optional_directory(&base_handle,spec.namespace)? else{return if known{Err("Original private namespace disappeared.".into())}else{Ok(None)}};
    if parent.metadata().map_err(|error|error.to_string())?.mode()&0o077!=0{return Err("Private namespace privacy is unavailable; absence cannot authorize a fresh scope.".into())}
    let Some(bucket)=optional_directory(&parent,&bucket_id)? else{return if known{Err("Original private store disappeared.".into())}else{Ok(None)}};
    drop(bucket);drop(parent);drop(base_handle);Self::open(base,root,spec,false).map(Some)
   }

   fn open(base:&Path,root:&Path,spec:Spec,exclusive:bool)->Result<Self,String>{
    for value in [spec.namespace,spec.key,spec.lock,spec.record]{if value.is_empty()||Path::new(value).components().count()!=1||Path::new(value).components().any(|part|!matches!(part,std::path::Component::Normal(_))){return Err("Private record specification is not a single component".into())}}
    let root_id=resource_inputs::RootIdentity::fingerprint(root)?;let root=root.canonicalize().map_err(|e|e.to_string())?;
    let address=if base.is_absolute(){base.to_path_buf()}else{std::env::current_dir().map_err(|e|e.to_string())?.join(base)};
    let (base,base_path)=data_base(base,exclusive)?;let parent=directory(&base,spec.namespace,exclusive)?;let parent_path=base_path.join(spec.namespace);
    let bucket_id=if spec.physical_bucket{physical_key(&root)?}else{root_id.clone()};let bucket=Arc::new(directory(&parent,&bucket_id,exclusive)?);let path=parent_path.join(&bucket_id);
    if parent.metadata().map_err(|e|e.to_string())?.mode()&0o077!=0||bucket.metadata().map_err(|e|e.to_string())?.mode()&0o077!=0{return Err("Private record directories must remain private to this user.".into())}
    same(&parent,&parent_path)?;same(&bucket,&path)?;
    let held_key=(address.clone(),spec.namespace,bucket_id.clone());let mut stores=HELD.lock().map_err(|_|"Private store identity unavailable")?;
    if !stores.contains_key(&held_key)&&stores.keys().filter(|(_,namespace,_)|*namespace==spec.namespace).count()>=64{return Err("Private store identity capacity reached; keep the current live review and retry in its original engine.".into())}
    if let Some(previous)=stores.get(&held_key){if same_file::Handle::from_file(previous.try_clone().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?!=same_file::Handle::from_file(bucket.try_clone().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?{return Err("Original private store was replaced; historical recovery requires its original storage.".into())}}else{stores.insert(held_key,bucket.clone());}drop(stores);
    let lock=Self::open_file(&bucket,spec.lock,if exclusive{libc::O_RDWR|libc::O_CREAT}else{libc::O_RDONLY},0o600)?;
    if exclusive{fs2::FileExt::try_lock_exclusive(&lock)}else{fs2::FileExt::try_lock_shared(&lock)}.map_err(|_|"Private record metadata is being updated; retain the current reading and retry.".to_string())?;
    let store_id=resource_inputs::RootIdentity::fingerprint(&path)?;
    let (key,created_key)=match Self::read_file(&bucket,spec.key,32)?{Some(bytes)if bytes.len()==32=>(bytes,false),Some(_)=>return Err("Private record key is damaged; no records were promoted to live authority.".into()),None=>{
     if !exclusive{return Err("Private record key is unavailable; no state was initialized by this reading.".into())}
     if Self::read_file(&bucket,spec.record,spec.limit)?.is_some(){return Err("Private record key is missing. Existing readings cannot be verified.".into())}
     let mut key=vec![0;32];getrandom::fill(&mut key).map_err(|e|e.to_string())?;let mut file=Self::open_file(&bucket,spec.key,libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL,0o600)?;file.write_all(&key).and_then(|_|file.sync_all()).map_err(|e|e.to_string())?;bucket.sync_all().map_err(|e|e.to_string())?;(key,true)
    }};
    let key_file=Self::open_file(&bucket,spec.key,libc::O_RDONLY,0)?;
    let store=Self{root,root_id,base,address,base_path,parent,parent_path,bucket,path,store_id,key,created_key,exclusive,spec,key_file,_lock:lock};store.check()?;Ok(store)
   }
   pub(crate) fn open_file(directory:&File,name:&str,flags:i32,mode:u32)->Result<File,String>{
    let name=c(name)?;let fd=unsafe{libc::openat(directory.as_raw_fd(),name.as_ptr(),flags|libc::O_NOFOLLOW|libc::O_NONBLOCK|libc::O_CLOEXEC,mode)};
    if fd<0{return Err(std::io::Error::last_os_error().to_string())}let file=unsafe{File::from_raw_fd(fd)};let metadata=file.metadata().map_err(|e|e.to_string())?;
    if !metadata.is_file()||metadata.nlink()!=1||metadata.mode()&0o077!=0{return Err("Private record files must be private ordinary files with one identity.".into())}Ok(file)
   }
   pub(crate) fn read_file(directory:&File,name:&str,limit:usize)->Result<Option<Vec<u8>>,String>{
    let name_c=c(name)?;let fd=unsafe{libc::openat(directory.as_raw_fd(),name_c.as_ptr(),libc::O_RDONLY|libc::O_NOFOLLOW|libc::O_NONBLOCK|libc::O_CLOEXEC)};
    if fd<0{let error=std::io::Error::last_os_error();return if error.kind()==std::io::ErrorKind::NotFound{Ok(None)}else{Err(format!("Cannot read private record: {error}"))}}
    let file=unsafe{File::from_raw_fd(fd)};let metadata=file.metadata().map_err(|e|e.to_string())?;
    if !metadata.is_file()||metadata.nlink()!=1||metadata.mode()&0o077!=0||metadata.len()>limit as u64{return Err("Private record file is not a bounded private ordinary file.".into())}
    let mut bytes=Vec::new();file.take(limit as u64+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;if bytes.len()>limit{return Err("Private record exceeds its reading budget.".into())}Ok(Some(bytes))
   }
   pub(crate) fn check(&self)->Result<(),String>{
    if self.parent.metadata().map_err(|e|e.to_string())?.mode()&0o077!=0||self.bucket.metadata().map_err(|e|e.to_string())?.mode()&0o077!=0{return Err("Private record directory privacy changed while reading or retaining it.".into())}
    if resource_inputs::RootIdentity::fingerprint(&self.root)?!=self.root_id{return Err("Original physical private workspace changed; no replacement source was adopted.".into())}
    same(&self.base,&self.address)?;same(&self.base,&self.base_path)?;same(&self.parent,&self.parent_path)?;same(&self.bucket,&self.path)?;same(&self.key_file,&self.path.join(self.spec.key))?;same(&self._lock,&self.path.join(self.spec.lock))?;
    if Self::read_file(&self.bucket,self.spec.key,32)?.as_ref()!=Some(&self.key){return Err("Private record key changed while retaining a reading.".into())}
    if resource_inputs::RootIdentity::fingerprint(&self.path)?!=self.store_id{return Err("Private record physical identity changed.".into())}Ok(())
   }
   pub(crate) fn read(&self)->Result<Option<Vec<u8>>,String>{self.check()?;let value=Self::read_file(&self.bucket,self.spec.record,self.spec.limit)?;self.check()?;Ok(value)}
   pub(crate) fn publication_lock(&self,exclusive:bool)->Result<File,String>{
    self.check()?;let file=Self::open_file(&self.bucket,"publication.lock",libc::O_RDWR|libc::O_CREAT,0o600)?;
    // Keep an independently opened, unlocked description. Duplicating the
    // locked descriptor here would accidentally retain its flock forever.
    let mut retained=HELD_PUBLICATION.lock().map_err(|_|"Original publication identity is unavailable")?;
    if let Some(original)=retained.get(&self.path){same(original,&self.path.join("publication.lock"))?;}else{if retained.len()>=256{return Err("Original publication identity capacity reached; preserve drafts and use the original engine.".into())}let original=Self::open_file(&self.bucket,"publication.lock",libc::O_RDONLY,0)?;same(&original,&self.path.join("publication.lock"))?;retained.insert(self.path.clone(),Arc::new(original));}drop(retained);
    if exclusive{fs2::FileExt::try_lock_exclusive(&file)}else{fs2::FileExt::try_lock_shared(&file)}.map_err(|_|"Another workspace entry publication is active; preserve source and retry.".to_string())?;self.check_publication_lock(&file)?;Ok(file)
   }
   pub(crate) fn check_publication_lock(&self,file:&File)->Result<(),String>{self.check()?;let metadata=file.metadata().map_err(|error|error.to_string())?;if !metadata.is_file()||metadata.nlink()!=1||metadata.mode()&0o077!=0{return Err("Publication lock must retain its private ordinary file identity.".into())}same(file,&self.path.join("publication.lock"))}
   pub(crate) fn replace(&self,expected:&Option<Vec<u8>>,bytes:&[u8])->Result<(),String>{
    if !self.exclusive{return Err("A read-only private snapshot cannot publish metadata.".into())}if bytes.len()>self.spec.limit{return Err("Private record exceeds its bounded storage limit.".into())}self.check()?;
    let mut random=[0;16];getrandom::fill(&mut random).map_err(|e|e.to_string())?;let temporary=format!(".{}.tmp",hex::encode(random));let mut file=Self::open_file(&self.bucket,&temporary,libc::O_WRONLY|libc::O_CREAT|libc::O_EXCL,0o600)?;
    let result=(||{file.write_all(bytes).and_then(|_|file.sync_all()).map_err(|e|e.to_string())?;self.check()?;if Self::read_file(&self.bucket,self.spec.record,self.spec.limit)?!=*expected{return Err("Private record changed before its compare-and-replace publication.".into())}let from=c(&temporary)?;let to=c(self.spec.record)?;if unsafe{libc::renameat(self.bucket.as_raw_fd(),from.as_ptr(),self.bucket.as_raw_fd(),to.as_ptr())}!=0{return Err(std::io::Error::last_os_error().to_string())}self.bucket.sync_all().map_err(|e|e.to_string())?;self.check()})();if result.is_err(){if let Ok(name)=c(&temporary){unsafe{libc::unlinkat(self.bucket.as_raw_fd(),name.as_ptr(),0)};}}result
   }
  }
}
#[cfg(unix)]pub(crate) use unix::HeldRecord;
</ui:copy>

Typed optional snapshots distinguish a genuinely absent namespace/bucket from unreadable, aliased, damaged or previously held missing storage. They do not create a key or turn a missing role record into initial authority. Publication locks also retain an independently opened identity handle: replacing the filename cannot create a second lock domain while an earlier lease lives. The retained description remains unlocked so dropping the actual lease releases its advisory lock.

## Check compatibility and lease boundaries

The focused witnesses prove the published legacy HMAC vector, purpose separation, concurrent read leases, denied metadata writes, stale replacement refusal and physical-bucket continuity without namespace-guard continuity. Existing proposal persistence witnesses remain required integration evidence; these model checks alone do not establish historical UI behavior.

<ui:copy id="private-record-witness">
#[cfg(all(test,unix))]
mod tests {
 use super::*;
 fn spec(physical:bool)->Spec{Spec{namespace:"private-record-owned-test",key:"state.key",lock:"state.lock",record:"state.json",limit:4096,physical_bucket:physical}}
 #[test]
 fn legacy_seal_bytes_remain_compatible_but_entry_domains_cannot_borrow_them(){let key=[0x0b;20];assert_eq!(seal(&key,None,b"Hi There").unwrap(),"b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7");let first=seal(&key,Some(b"workspace-entry.v1"),b"material").unwrap();verify(&key,Some(b"workspace-entry.v1"),b"material",&first).unwrap();assert!(verify(&key,Some(b"agent-proposal.v1"),b"material",&first).is_err());assert!(verify(&key,None,b"material",&first).is_err());}
 #[test]
 fn shared_snapshots_and_publication_leases_never_grant_metadata_writes(){let temporary=tempfile::tempdir().unwrap();let root=temporary.path().join("workspace");std::fs::create_dir(&root).unwrap();let base=temporary.path().join("private");let store=HeldRecord::under(&base,&root,spec(true)).unwrap();assert!(store.created_key);store.replace(&None,b"original").unwrap();drop(store);let first=HeldRecord::snapshot(&base,&root,spec(true)).unwrap();let second=HeldRecord::snapshot(&base,&root,spec(true)).unwrap();assert!(!first.created_key);assert_eq!(first.read().unwrap().unwrap(),b"original");assert!(first.replace(&Some(b"original".to_vec()),b"forbidden").is_err());assert!(HeldRecord::under(&base,&root,spec(true)).is_err());let a=first.publication_lock(false).unwrap();let b=second.publication_lock(false).unwrap();assert!(second.publication_lock(true).is_err());first.check_publication_lock(&a).unwrap();drop(a);drop(b);drop(first);drop(second);let store=HeldRecord::under(&base,&root,spec(true)).unwrap();assert!(store.replace(&Some(b"wrong".to_vec()),b"new").is_err());assert_eq!(store.read().unwrap().unwrap(),b"original");store.replace(&Some(b"original".to_vec()),b"new").unwrap();}
 #[test]
 fn physical_bucket_finds_retained_bytes_after_root_rename_without_reusing_namespace_guard(){let temporary=tempfile::tempdir().unwrap();let root=temporary.path().join("workspace");std::fs::create_dir(&root).unwrap();let base=temporary.path().join("private");let store=HeldRecord::under(&base,&root,spec(true)).unwrap();let bucket=store.path.clone();let old_guard=store.root_id.clone();store.replace(&None,b"retained binding").unwrap();let renamed=temporary.path().join("renamed");std::fs::rename(&root,&renamed).unwrap();assert!(store.check().is_err());drop(store);let current=HeldRecord::under(&base,&renamed,spec(true)).unwrap();assert_eq!(current.path,bucket);assert_ne!(current.root_id,old_guard);assert_eq!(current.read().unwrap().unwrap(),b"retained binding");}
 #[test]
 fn optional_snapshot_distinguishes_absence_from_alias_damage_and_original_loss(){
  let temporary=tempfile::tempdir().unwrap();let root=temporary.path().join("workspace");std::fs::create_dir(&root).unwrap();let base=temporary.path().join("state");assert!(HeldRecord::try_snapshot(&base,&root,spec(true)).unwrap().is_none());assert!(!base.exists());std::fs::create_dir(&base).unwrap();std::fs::write(base.join("private-record-owned-test"),"not a namespace directory").unwrap();assert!(HeldRecord::try_snapshot(&base,&root,spec(true)).is_err());std::fs::remove_file(base.join("private-record-owned-test")).unwrap();let namespace=base.join("private-record-owned-test");std::fs::create_dir(&namespace).unwrap();std::fs::set_permissions(&namespace,<std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o755)).unwrap();assert!(HeldRecord::try_snapshot(&base,&root,spec(true)).is_err());std::fs::remove_dir(namespace).unwrap();let original=HeldRecord::under(&base,&root,spec(true)).unwrap();original.replace(&None,b"retained").unwrap();let path=original.path.clone();drop(original);assert_eq!(HeldRecord::try_snapshot(&base,&root,spec(true)).unwrap().unwrap().read().unwrap().unwrap(),b"retained");std::fs::remove_file(path.join("state.key")).unwrap();assert!(HeldRecord::try_snapshot(&base,&root,spec(true)).is_err());std::fs::remove_dir_all(path).unwrap();assert!(HeldRecord::try_snapshot(&base,&root,spec(true)).is_err());
 }
 #[test]
 fn publication_lock_substitution_refuses_without_retaining_its_advisory_lock_forever(){
  let temporary=tempfile::tempdir().unwrap();let root=temporary.path().join("workspace");std::fs::create_dir(&root).unwrap();let base=temporary.path().join("state");let store=HeldRecord::under(&base,&root,spec(true)).unwrap();let reader=store.publication_lock(false).unwrap();let path=store.path.join("publication.lock");let retired=store.path.join("retired.lock");std::fs::rename(&path,&retired).unwrap();drop(HeldRecord::open_file(&store.bucket,"publication.lock",libc::O_RDWR|libc::O_CREAT|libc::O_EXCL,0o600).unwrap());assert!(store.publication_lock(true).is_err());assert!(store.check_publication_lock(&reader).is_err());std::fs::remove_file(&path).unwrap();std::fs::rename(retired,&path).unwrap();store.check_publication_lock(&reader).unwrap();drop(reader);drop(store);let store=HeldRecord::under(&base,&root,spec(true)).unwrap();store.publication_lock(true).unwrap();
 }
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/private_record.rs"><ui:paste select="#private-record-specification"/><ui:paste select="#private-record-held-storage"/><ui:paste select="#private-record-witness"/></ui:file>
</ui:doc>
