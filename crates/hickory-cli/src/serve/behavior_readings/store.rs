use super::*;
use std::io::Write;
fn path(root: &Path, id: &str, create: bool) -> Result<std::path::PathBuf, String> {
    if id.len() != 32
        || !id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("Verification ID must be 32 lowercase hexadecimal characters.".into());
    }
    let mut dir = root.canonicalize().map_err(|e| e.to_string())?;
    for part in [".hick-cache", "behavior-verification"] {
        dir.push(part);
        if create && !dir.exists() {
            std::fs::create_dir(&dir).map_err(|e| e.to_string())?
        }
        match std::fs::symlink_metadata(&dir) {
            Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => {}
            Ok(_) => {
                return Err("Verification storage cannot be a symlink or non-directory.".into());
            }
            Err(e) if !create && e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(dir.join(format!("{id}.json")))
}
fn read_retained(root: &Path, id: &str) -> Result<Option<Value>, String> {
    let path = path(root, id, false)?;
    let meta = match std::fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    if meta.file_type().is_symlink() || !meta.is_file() || meta.len() > 2 * 1024 * 1024 {
        return Err("Verification receipt is unsafe or too large.".into());
    }
    let value: Value = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if value["request_id"]!=id{return Err("Verification receipt request identity changed.".into());}
    Ok(Some(value))
}
pub(super) fn existing(root:&Path,id:&str)->Result<Option<Value>,String>{let value=read_retained(root,id)?;if let Some(value)=&value{if value["workspace_identity_kind"]!="physical-root-v1"||value["workspace_id"]!=super::super::resource_inputs::RootIdentity::fingerprint(root)?{return Err("Historical or foreign verification origin is unconfirmed; no request was replayed.".into());}}Ok(value)}
pub(super) fn reserve(root: &Path, id: &str, value: &Value) -> Result<(), String> {
    let path = path(root, id, true)?;
    let count = std::fs::read_dir(path.parent().unwrap())
        .map_err(|e| e.to_string())?
        .take(4097)
        .count();
    if count >= 4096 {
        return Err("Verification receipt store is full. Retained receipts were not evicted; no check was started.".into());
    }
    let bytes=serde_json::to_vec(value).map_err(|error|error.to_string())?;
    if bytes.len()>1024*1024{return Err("Verification input receipt exceeds the retained-capacity budget. No check was started.".into())}
    let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e|format!("Verification request is already consumed or storage is unavailable: {e}. Inspect its receipt; do not retry with a new ID."))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}
pub(super) fn save(root: &Path, id: &str, value: &Value) -> Result<(), String> {
    let path = path(root, id, true)?;
    let bytes=serde_json::to_vec(value).map_err(|error|error.to_string())?;
    if bytes.len()>2*1024*1024{return Err("Verification observation exceeds its retained budget. The previous consumed receipt remains authoritative; no process was retried.".into())}
    let mut file =
        tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}
pub(super) fn list(root: &Path) -> Result<Value, String> {
    let directory = path(root, "00000000000000000000000000000000", false)?
        .parent()
        .unwrap()
        .to_path_buf();
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(json!({"status":"missing","records":[],"truncated":false}));
        }
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    let mut budget = 16 * 1024 * 1024u64;
    let mut truncated = false;
    let mut unknown = 0;
    for entry in entries.take(4097) {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(id) = name.strip_suffix(".json") else {
            continue;
        };
        let meta = std::fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
        if meta.len() > budget || records.len() >= 100 {
            truncated = true;
            break;
        }
        budget -= meta.len();
        match read_retained(root, id) {
            Ok(Some(mut record)) => {let current=record["workspace_identity_kind"]=="physical-root-v1"&&super::super::resource_inputs::RootIdentity::fingerprint(root).ok().is_some_and(|identity|record["workspace_id"]==identity);if current{record["origin"]=json!("current-workspace");}else{record["origin"]=json!("unknown");record["terminal"]=Value::Null;record["freshness"]=json!("origin-unknown");record["notice"]=json!("Retained historical origin is unconfirmed; no live process or replay authority is attached.");}records.push(record)},
            Ok(None) => {}
            Err(_) => unknown += 1,
        }
    }
    records.sort_by_key(|record| std::cmp::Reverse(record["started_ms"].as_u64().unwrap_or(0)));
    Ok(
        json!({"status":"retained","records":records,"truncated":truncated,"unknown_records":unknown}),
    )
}
use std::sync::{Arc,Mutex,OnceLock,Weak};
type RootScope=super::super::resource_inputs::RootIdentity;
struct OwnedProcess {scope:Arc<RootScope>,hub:Weak<hick_term::Terminals>,runtime:String,id:String,intent:String,value:Value,known_terminal:Option<String>,revision:u64,quota:usize,disk_reserved:bool,persisted:bool}
#[derive(Default)]struct OwnedProcesses {rows:std::collections::VecDeque<OwnedProcess>,bytes:usize}
static OWNED:OnceLock<Mutex<OwnedProcesses>>=OnceLock::new();
#[derive(Clone)]pub(super) struct OwnedSnapshot {pub value:Value,pub known_terminal:Option<String>,pub revision:u64,pub runtime:String,pub disk_reserved:bool}
impl OwnedProcess {fn snapshot(&self)->OwnedSnapshot{OwnedSnapshot{value:self.value.clone(),known_terminal:self.known_terminal.clone(),revision:self.revision,runtime:self.runtime.clone(),disk_reserved:self.disk_reserved}}fn belongs(&self,scope:&Arc<RootScope>,hub:&Arc<hick_term::Terminals>)->bool{Arc::ptr_eq(&self.scope,scope)&&self.hub.upgrade().is_some_and(|owned|Arc::ptr_eq(&owned,hub))}}
fn owned_cache()->Result<std::sync::MutexGuard<'static,OwnedProcesses>,String>{OWNED.get_or_init(||Mutex::new(OwnedProcesses::default())).lock().map_err(|_|"Owned verification retention is unavailable; no untracked process is authorized.".into())}
pub(super) fn owned(scope:&Arc<RootScope>,hub:&Arc<hick_term::Terminals>,id:&str)->Result<Option<OwnedSnapshot>,String>{Ok(owned_cache()?.rows.iter().find(|row|row.id==id&&row.belongs(scope,hub)).map(OwnedProcess::snapshot))}
/// None means this exact engine-owned intention was reserved; Some is an existing replay.
pub(super) fn claim_owned(scope:&Arc<RootScope>,hub:&Arc<hick_term::Terminals>,runtime:&str,id:&str,value:&Value)->Result<Option<OwnedSnapshot>,String>{
 if id.len()!=32||!id.bytes().all(|byte|byte.is_ascii_digit()||(b'a'..=b'f').contains(&byte)){return Err("Verification request identity is malformed.".into())}
 let intent=value["request_fingerprint"].as_str().filter(|value|value.len()==64).ok_or("Retained verification has no full intention fingerprint.")?;
 let mut cache=owned_cache()?;if let Some(row)=cache.rows.iter().find(|row|row.id==id&&row.belongs(scope,hub)){if row.intent!=intent{return Err("This request ID belongs to a different verification intention.".into())}return Ok(Some(row.snapshot()))}
 let encoded=serde_json::to_vec(value).map_err(|error|error.to_string())?.len();if encoded>1024*1024{return Err("Verification intention exceeds the bounded retained input budget; no process was started.".into())}let quota=encoded+512*1024;
 while cache.rows.len()>=256||cache.bytes.saturating_add(quota)>32*1024*1024{let index=cache.rows.iter().position(|row|row.hub.upgrade().is_none()||row.persisted&&matches!(row.value["status"].as_str(),Some("process-finished"|"admission-refused"|"request-refused"))).ok_or("Owned verification retention is full. Refresh or stop the existing process requests before starting another; uncertain requests were not evicted.")?;let row=cache.rows.remove(index).unwrap();cache.bytes-=row.quota;}
 let mut value=value.clone();value["observation_revision"]=json!(0);value["observation_engine_workspace_id"]=json!(runtime);
 cache.rows.push_back(OwnedProcess{scope:scope.clone(),hub:Arc::downgrade(hub),runtime:runtime.into(),id:id.into(),intent:intent.into(),value,known_terminal:None,revision:0,quota,disk_reserved:false,persisted:false});cache.bytes+=quota;Ok(None)
}
pub(super) fn disk_reserved(scope:&Arc<RootScope>,hub:&Arc<hick_term::Terminals>,id:&str)->Result<(),String>{let mut cache=owned_cache()?;let row=cache.rows.iter_mut().find(|row|row.id==id&&row.belongs(scope,hub)).ok_or("Owned request disappeared before persistence.")?;row.disk_reserved=true;row.persisted=true;Ok(())}
/// Updates are CAS observations; a held older read cannot replace a newer final outcome.
pub(super) fn update_owned(scope:&Arc<RootScope>,hub:&Arc<hick_term::Terminals>,id:&str,expected:Option<u64>,mut value:Value,known_terminal:Option<String>,persisted:bool)->Result<OwnedSnapshot,String>{
 let mut cache=owned_cache()?;let row=cache.rows.iter_mut().find(|row|row.id==id&&row.belongs(scope,hub)).ok_or("The original engine no longer owns this verification request.")?;
 if expected.is_some_and(|expected|expected!=row.revision){return Ok(row.snapshot())}
 if value["request_fingerprint"]!=row.intent{return Err("Observed process no longer matches its complete captured intention.".into())}
 if row.value["status"]=="process-finished"&&value["status"]!="process-finished"{return Ok(row.snapshot())}
 if let Some(id)=known_terminal{if row.known_terminal.as_ref().is_some_and(|known|known!=&id){return Err("An owned request cannot adopt another process identity.".into())}row.known_terminal=Some(id)}
 row.revision=row.revision.saturating_add(1);value["observation_revision"]=json!(row.revision);value["observation_engine_workspace_id"]=json!(row.runtime);
 if serde_json::to_vec(&value).map_err(|error|error.to_string())?.len()>row.quota{value["output"]=Value::Null;value["output_truncated"]=json!(true);if let Some(id)=&row.known_terminal{value["terminal"]=json!({"id":id});}}
 if serde_json::to_vec(&value).map_err(|error|error.to_string())?.len()>row.quota{return Err("Process metadata exceeds its pre-reserved retention budget. The original owned request remains retained.".into())}
 row.value=value;row.persisted=persisted;Ok(row.snapshot())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unretainable_inputs_refuse_before_consuming_the_request(){let root=tempfile::tempdir().unwrap();let id="11111111111111111111111111111111";assert!(reserve(root.path(),id,&json!({"checked_sources":"x".repeat(1024*1024+1)})).is_err());assert!(!path(root.path(),id,false).unwrap().exists());let value=json!({"workspace_identity_kind":"physical-root-v1","workspace_id":super::super::super::resource_inputs::RootIdentity::fingerprint(root.path()).unwrap(),"request_id":id,"status":"admitting"});reserve(root.path(),id,&value).unwrap();assert!(existing(root.path(),id).unwrap().is_some());}

    #[test]
    fn consumed_requests_do_not_reexecute_and_foreign_receipts_refuse() {
        let root = tempfile::tempdir().unwrap();
        let id = "0123456789abcdef0123456789abcdef";
        let value = json!({"request_id":id,"workspace_identity_kind":"physical-root-v1","workspace_id":super::super::super::resource_inputs::RootIdentity::fingerprint(root.path()).unwrap(),"status":"admitting"});
        reserve(root.path(), id, &value).unwrap();
        assert!(reserve(root.path(), id, &value).is_err());
        assert_eq!(
            existing(root.path(), id).unwrap().unwrap()["status"],
            "admitting"
        );
        let other = tempfile::tempdir().unwrap();
        let mut foreign = value;
        foreign["workspace_id"] = json!(hash(other.path().as_os_str().as_encoded_bytes()));
        save(root.path(), id, &foreign).unwrap();
        assert!(existing(root.path(), id).is_err());
        assert!(existing(root.path(), "../escape").is_err());
    }
    #[test]
    fn legacy_path_only_origins_remain_readable_without_replay_or_terminal_authority(){let root=tempfile::tempdir().unwrap();let id="22222222222222222222222222222222";let old=json!({"version":1,"request_id":id,"workspace_id":hash(root.path().canonicalize().unwrap().as_os_str().as_encoded_bytes()),"label":"Legacy result","status":"process-finished","terminal":{"id":"old-process"}});reserve(root.path(),id,&old).unwrap();assert!(existing(root.path(),id).is_err());let value=list(root.path()).unwrap();let row=&value["records"][0];assert_eq!(row["label"],"Legacy result");assert_eq!(row["origin"],"unknown");assert!(row["terminal"].is_null());}

}
