<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Hold a workspace-entry reading without inventing a default

For maintainers coordinating compiler capture/publication with main-entry changes. The reader holds physical root, coordination-lock and optional role-record identities. It never waits on a writer. Ordinary source write gating and reviewed Move publication must acquire this same coordination provider before the read lease can claim exclusion against them; this slice alone is not completed namespace publication.

## Keep coordination separate from role initialization

An explicit ROOT_ENTRY needs a shared coordination lease without creating a main role. Its fixed private namespace contains only lock-provider state. A present role record must still validate; absence can be accepted only through the typed optional reader. A retained role-loss hint can refuse missing private state but can never supply an entry address, generation or permission.

<ui:copy id="workspace-entry-coordination-provider">
#[cfg(unix)]
mod leases {
 use super::*;
 use std::{fs::File,path::Path};
 use super::super::private_record::{HeldRecord,Spec};
 const COORDINATION:Spec=Spec{namespace:"workspace-source-publication-v1",key:"coordination.key",lock:"coordination.lock",record:"coordination.state",limit:256,physical_bucket:true};
 pub(super) struct Coordination {held:HeldRecord,publication:File}
 impl Coordination {
  fn check(&self)->Result<(),String>{self.held.check_publication_lock(&self.publication)?;if self.held.read()?.is_some(){return Err("Source publication coordination contains unexpected record material.".into())}Ok(())}
 }
 pub(super) fn coordination_under(base:&Path,root:&Path,exclusive:bool)->Result<Coordination,String>{
  let held=match HeldRecord::try_snapshot(base,root,COORDINATION)?{
   Some(held)=>held,
   None=>{let initialized=HeldRecord::under(base,root,COORDINATION)?;if initialized.read()?.is_some(){return Err("Source publication coordination is not an empty lock provider.".into())}let publication=initialized.publication_lock(exclusive)?;drop(initialized);let held=HeldRecord::snapshot(base,root,COORDINATION)?;let lease=Coordination{held,publication};lease.check()?;return Ok(lease)}
  };
  let publication=held.publication_lock(exclusive)?;let lease=Coordination{held,publication};lease.check()?;Ok(lease)
 }
 pub(super) fn coordination(root:&Path,exclusive:bool)->Result<Coordination,String>{let base=hickory_workspace::data_root().map_err(|error|error.to_string())?;coordination_under(&base,root,exclusive)}
 #[derive(Serialize,Deserialize)]#[serde(deny_unknown_fields)]struct RoleLossHint{version:u32,identity:String}
 pub(super) fn role_loss_hint(root:&Path)->Result<bool,String>{
  let directory=root.join(".hick-cache");match std::fs::symlink_metadata(&directory){Err(error)if error.kind()==std::io::ErrorKind::NotFound=>return Ok(false),Err(error)=>return Err(error.to_string()),Ok(metadata)if !metadata.is_dir()||metadata.file_type().is_symlink()=>return Err("Workspace entry recovery metadata is not an ordinary directory.".into()),Ok(_)=>{}}
  let path=directory.join("workspace-entry-role.json");match std::fs::symlink_metadata(&path){Err(error)if error.kind()==std::io::ErrorKind::NotFound=>Ok(false),Err(error)=>Err(error.to_string()),Ok(_)=>{let bytes=super::super::resource_journal::read_regular(&path,4096)?;let hint:RoleLossHint=serde_json::from_slice(&bytes).map_err(|_|"Workspace-entry role hint is damaged; missing state cannot become a fresh default.")?;if hint.version!=1||hint.identity.len()!=32||!hint.identity.bytes().all(|byte|byte.is_ascii_digit()||(b'a'..=b'f').contains(&byte)){return Err("Workspace-entry role hint is unsupported; restore the original private binding.".into())}Ok(true)}}
 }
 fn ordinary_root(root:&Path)->Result<(),String>{
  if crate::engine::private_policy::active().map_err(|error|error.to_string())?||crate::engine::private_policy::exact_private_root(root).map_err(|error|error.to_string())?{return Err("Ordinary workspace entry and generation are unavailable for a retained private-practice scope. Use its captured source provider.".into())}Ok(())
 }
</ui:copy>

## Bind capture to the original tuple

Implicit readers require the exact retained identity, generation and address. Explicit readers retain their chosen address semantics and do not initialize or repair a role. All reads retain the original metadata bytes and physical root; later checks refuse changes instead of renewing the scope. Private practice uses its own captured source provider.

<ui:copy id="workspace-entry-read-lease-admission">
 pub(crate) struct EntryReadLease {coordination:Coordination,bound:Option<super::storage::Bound>,_ordinary:Option<crate::engine::private_policy::Ordinary>}
 impl EntryReadLease {
  /// Membership is meaningful only after check() succeeds on this same held lease.
  pub(crate) fn matches_current(&self,expected:&EntryStamp)->bool{self.bound.as_ref().is_some_and(|bound|bound.payload.pending.is_none()&&bound.payload.current.path.is_some()&&bound.payload.current==*expected)}
  pub(crate) fn check(&self)->Result<(),String>{self.coordination.check()?;if let Some(bound)=&self.bound{bound.unchanged()?;if bound.payload.pending.is_some(){return Err("Workspace entry publication remains unconfirmed; no generation scope was renewed.".into())}}if self._ordinary.is_some(){if crate::engine::private_policy::active().map_err(|error|error.to_string())?{return Err("Ordinary generation paused for private practice.".into())}}Ok(())}
 }
 fn read_under(base:&Path,root:&Path,expected:Option<&EntryStamp>,ordinary:Option<crate::engine::private_policy::Ordinary>)->Result<EntryReadLease,String>{
  // Acquisition is shared and never waits or upgrades. A default binding must
  // be resolved separately before capture; explicit entry reads never do that.
  let coordination=coordination_under(base,root,false)?;
  let bound=super::storage::Bound::optional_under(base,root)?;
  if let Some(expected)=expected{expected.validate()?;if expected.path.is_none()||bound.as_ref().is_none_or(|bound|bound.payload.current!=*expected){return Err("Implicit generation lost its original workspace-entry identity, generation or address. Reopen the workspace explicitly; no path fallback was selected.".into())}}
  let lease=EntryReadLease{coordination,bound,_ordinary:ordinary};lease.check()?;Ok(lease)
 }
 pub(crate) fn acquire_entry_read_lease(root:&Path,expected:Option<&EntryStamp>)->Result<EntryReadLease,String>{let ordinary=crate::engine::private_policy::ordinary();ordinary_root(root)?;let base=hickory_workspace::data_root().map_err(|error|error.to_string())?;read_under(&base,root,expected,Some(ordinary))}
 pub(crate) fn default_entry_binding(root:&Path)->Result<EntryStamp,String>{
  let _ordinary=crate::engine::private_policy::ordinary();ordinary_root(root)?;
  let bound=super::storage::Bound::snapshot_or_initialize(root)?;
  if bound.payload.pending.is_some(){return Err("Workspace entry publication remains unconfirmed. Read its retained recovery before generation.".into())}
  let path=bound.payload.current.path.as_ref().ok_or("No default workspace construction is configured. Supply an explicit ROOT_ENTRY or author the first declared project.md.")?;
  let observed=super::entry_observation(root,path)?;let document=hick_lang::parse(&observed.source).map_err(|error|error.to_string())?;
  if document.find_tags("project").is_empty(){return Err("The retained workspace entry no longer declares its project. Repair it; no project.md fallback was selected.".into())}
  let stamp=bound.payload.current.clone();bound.unchanged()?;ordinary_root(root)?;Ok(stamp)
 }
</ui:copy>

## Make source publication participate in the same exclusion

A compiler reader may consume a tag-free contributor. Until a positive inventory proves independence, every source write inside the workspace takes the exclusive coordination lease. Non-Unix engines preserve their existing ordinary Save path without claiming this interlock: entry-role Move and coordinated watch admission remain explicitly unavailable there. This deliberately conservative intermediate rule refuses immediately while a reader is active; it never waits while holding the global history writer mutex. Test-only coordination storage stays inside the owned fixture state directory and is removed with that fixture; production uses the existing private workspace state directory. Independently registered physical files outside the workspace retain their existing source authority and do not borrow its entry role. The lease is a prerequisite, not permission to edit generated output or bypass saved-source guards.

<ui:copy id="workspace-entry-source-coordination">
pub(crate) struct SourceCoordinationLease {coordination:Option<Coordination>,bound:Option<super::storage::Bound>,_ordinary:Option<crate::engine::private_policy::Ordinary>}
impl SourceCoordinationLease {
 pub(crate) fn check(&self)->Result<(),String>{if let Some(coordination)=&self.coordination{coordination.check()?}if let Some(bound)=&self.bound{bound.unchanged()?;if bound.payload.pending.is_some(){return Err("Workspace-entry publication remains unconfirmed. Source text is retained; inspect its recovery before saving.".into())}}if crate::engine::private_policy::active().map_err(|error|error.to_string())?{return Err("Ordinary source save paused for private practice.".into())}Ok(())}
}
fn source_target(root:&Path,path:&Path)->Result<Option<String>,String>{
 let root=root.canonicalize().map_err(|error|error.to_string())?;
 let metadata=match std::fs::symlink_metadata(path){Ok(metadata)=>Some(metadata),Err(error)if error.kind()==std::io::ErrorKind::NotFound=>None,Err(error)=>return Err(error.to_string())};
 if metadata.as_ref().is_some_and(|metadata|!metadata.is_file()||metadata.file_type().is_symlink()){return Err("File aliases remain readable but have no ordinary publication lease. Open the exact authored source.".into())}
 let canonical=if metadata.is_some(){path.canonicalize().map_err(|error|error.to_string())?}else{path.parent().ok_or("Source destination has no parent")?.canonicalize().map_err(|error|error.to_string())?.join(path.file_name().ok_or("Source destination has no filename")?)};
 if let Ok(relative)=canonical.strip_prefix(&root){return relative.to_str().map(|path|Some(path.replace(std::path::MAIN_SEPARATOR,"/"))).ok_or("Source address is not exact UTF-8.".into())}
 // A canonical outside file with one physical link cannot be a hardlinked
 // workspace source. Existing registered-file and original save guards still
 // remain mandatory in the caller. Ambiguous multi-link outside files refuse.
 if canonical!=path||metadata.as_ref().is_some_and(|metadata|{use std::os::unix::fs::MetadataExt;metadata.nlink()!=1}){return Err("Outside source identity is aliased or has multiple physical names. Preserve the draft and open its exact independent source.".into())}Ok(None)
}
fn source_coordination_under(base:&Path,root:&Path,path:&Path,source:&str,ordinary:Option<crate::engine::private_policy::Ordinary>)->Result<SourceCoordinationLease,String>{
 let Some(relative)=source_target(root,path)? else{return Ok(SourceCoordinationLease{coordination:None,bound:None,_ordinary:ordinary})};
 let bound=super::storage::Bound::optional_under(base,root)?;
 // Until the captured compiler inventory proves this file unrelated, even
 // tag-free contributors may feed the main graph. Never authorize a shared
 // write merely because the edited text has no construction tags.
 let _=(relative,source);
 let coordination=coordination_under(base,root,true)?;
 let lease=SourceCoordinationLease{coordination:Some(coordination),bound,_ordinary:ordinary};lease.check()?;Ok(lease)
}
pub(crate) fn acquire_source_coordination(root:&Path,path:&Path,source:&str)->Result<SourceCoordinationLease,String>{
 let ordinary=crate::engine::private_policy::ordinary();ordinary_root(root)?;
 #[cfg(not(test))]let base=hickory_workspace::data_root().map_err(|error|error.to_string())?;
 #[cfg(test)]let base=root.join(".entry-coordination-state");
 source_coordination_under(&base,root,path,source,Some(ordinary))
}
</ui:copy>

## Exercise nested readers, contention and unknown storage

The focused witnesses distinguish two simultaneous shared reads from an exclusive publisher, reject stale implicit stamps, and keep pending/lost/inaccessible state distinct from genuine absence. They test provider admission, not a usable Move command or physical input.

<ui:copy id="workspace-entry-read-lease-witness">
 #[cfg(test)]
 mod tests {
  use super::*;
  fn fixture()->(tempfile::TempDir,std::path::PathBuf,std::path::PathBuf){let temporary=tempfile::tempdir().unwrap();let root=temporary.path().join("workspace");std::fs::create_dir(&root).unwrap();let root=root.canonicalize().unwrap();let base=temporary.path().join("state");(temporary,root,base)}
  #[test]fn explicit_read_never_initializes_a_main_and_shared_reads_do_not_block_each_other(){let(_temporary,root,base)=fixture();let first=read_under(&base,&root,None,None).unwrap();let second=read_under(&base,&root,None,None).unwrap();first.check().unwrap();second.check().unwrap();assert!(!first.matches_current(&EntryStamp{version:1,identity:"0".repeat(32),generation:0,path:Some("project.md".into())}));assert!(!base.join("workspace-entry-v1").exists());assert!(coordination_under(&base,&root,true).is_err());drop(first);drop(second);coordination_under(&base,&root,true).unwrap();}
  #[test]fn original_implicit_binding_must_be_present_and_exact(){let(_temporary,root,base)=fixture();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project/></h:doc>").unwrap();let bound=super::super::storage::Bound::resolve_under(&base,&root).unwrap();let stamp=bound.payload.current.clone();drop(bound);let lease=read_under(&base,&root,Some(&stamp),None).unwrap();lease.check().unwrap();assert!(lease.matches_current(&stamp));let stale=stamp.advance(stamp.path.clone()).unwrap();assert!(!lease.matches_current(&stale));assert!(read_under(&base,&root,Some(&stale),None).is_err());drop(lease);let other=root.parent().unwrap().join("other");std::fs::create_dir(&other).unwrap();assert!(read_under(&base,&other,Some(&stamp),None).is_err());}
  #[test]fn pending_binding_is_not_explicit_absence(){let(_temporary,root,base)=fixture();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project/></h:doc>").unwrap();let mut bound=super::super::storage::Bound::resolve_under(&base,&root).unwrap();let stamp=bound.payload.current.clone();bound.payload.pending=Some(Pending{request_id:"1".repeat(32),manifest_sha256:"2".repeat(64),change:EntryChange{version:1,kind:TransitionKind::Move,before:stamp.clone(),after:stamp.advance(Some("next.md".into())).unwrap(),source_identity:Some("owned".into())},after_guard:None});bound.persist().unwrap();drop(bound);assert!(read_under(&base,&root,None,None).is_err());}
  #[test]fn lost_or_inaccessible_role_store_refuses_instead_of_creating_explicit_authority(){let(_temporary,root,base)=fixture();let bound=super::super::storage::Bound::resolve_under(&base,&root).unwrap();let path=bound.held.path.clone();drop(bound);std::fs::remove_file(path.join("entry.json")).unwrap();assert!(read_under(&base,&root,None,None).is_err());let fresh_base=base.with_file_name("another-state");std::fs::create_dir(root.join(".hick-cache")).unwrap();std::fs::write(root.join(".hick-cache/workspace-entry-role.json"),serde_json::to_vec(&serde_json::json!({"version":1,"identity":"f".repeat(32)})).unwrap()).unwrap();assert!(read_under(&fresh_base,&root,None,None).is_err());assert!(!fresh_base.join("workspace-entry-v1").exists());std::fs::remove_file(root.join(".hick-cache/workspace-entry-role.json")).unwrap();std::fs::write(fresh_base.join("workspace-entry-v1"),"not a directory").unwrap();assert!(read_under(&fresh_base,&root,None,None).is_err());}

  #[test]fn tag_free_contributor_write_is_exclusive_until_inventory_proves_independence(){let(_temporary,root,base)=fixture();let path=root.join("part.md");std::fs::write(&path,"ordinary prose\r\n").unwrap();let reading=read_under(&base,&root,None,None).unwrap();assert!(source_coordination_under(&base,&root,&path,"new prose",None).is_err());assert!(!base.join("workspace-entry-v1").exists());drop(reading);let writing=source_coordination_under(&base,&root,&path,"new prose",None).unwrap();assert!(read_under(&base,&root,None,None).is_err());writing.check().unwrap();drop(writing);read_under(&base,&root,None,None).unwrap();}
  #[test]fn source_creation_and_malformed_canonical_repair_use_same_exclusive_provider(){let(_temporary,root,base)=fixture();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project/></h:doc>").unwrap();drop(super::super::storage::Bound::resolve_under(&base,&root).unwrap());std::fs::write(root.join("project.md"),"<h:doc").unwrap();let writing=source_coordination_under(&base,&root,&root.join("project.md"),"repaired source",None).unwrap();assert!(read_under(&base,&root,None,None).is_err());drop(writing);let reading=read_under(&base,&root,None,None).unwrap();assert!(source_coordination_under(&base,&root,&root.join("new.txt"),"new",None).is_err());drop(reading);source_coordination_under(&base,&root,&root.join("new.txt"),"new",None).unwrap();}
  #[test]fn independent_outside_source_does_not_borrow_workspace_role_or_block_reading(){let(_temporary,root,base)=fixture();let outside=root.parent().unwrap().join("outside.txt");std::fs::write(&outside,"outside").unwrap();let reading=read_under(&base,&root,None,None).unwrap();let writing=source_coordination_under(&base,&root,&outside,"outside edit",None).unwrap();writing.check().unwrap();assert!(!base.join("workspace-entry-v1").exists());std::os::unix::fs::symlink(&outside,root.parent().unwrap().join("alias.txt")).unwrap();assert!(source_coordination_under(&base,&root,&root.parent().unwrap().join("alias.txt"),"edit",None).is_err());std::fs::hard_link(&outside,root.join("hardlink.txt")).unwrap();assert!(source_coordination_under(&base,&root,&outside,"edit",None).is_err());reading.check().unwrap();}
  #[test]fn write_lease_refuses_role_record_mutation_instead_of_blessing_equal_source(){let(_temporary,root,base)=fixture();let bound=super::super::storage::Bound::resolve_under(&base,&root).unwrap();let record=bound.held.path.join("entry.json");drop(bound);let writing=source_coordination_under(&base,&root,&root.join("plain.txt"),"new",None).unwrap();let mut bytes=std::fs::read(&record).unwrap();bytes.push(b' ');std::fs::write(&record,&bytes).unwrap();assert!(writing.check().is_err());}

 }
}
#[cfg(unix)]
pub(crate) use leases::{EntryReadLease,SourceCoordinationLease,acquire_source_coordination,acquire_entry_read_lease,default_entry_binding};
#[cfg(not(unix))]pub(crate) struct SourceCoordinationLease;
#[cfg(not(unix))]impl SourceCoordinationLease{pub(crate) fn check(&self)->Result<(),String>{Ok(())}}
#[cfg(not(unix))]pub(crate) fn acquire_source_coordination(_root:&std::path::Path,_path:&std::path::Path,_source:&str)->Result<SourceCoordinationLease,String>{Ok(SourceCoordinationLease)}
#[cfg(not(unix))]pub(crate) struct EntryReadLease;
#[cfg(not(unix))]impl EntryReadLease{pub(crate) fn matches_current(&self,_expected:&EntryStamp)->bool{false}pub(crate) fn check(&self)->Result<(),String>{Err("Workspace-entry publication interlocks are unavailable on this engine platform.".into())}}
#[cfg(not(unix))]pub(crate) fn acquire_entry_read_lease(_root:&std::path::Path,_expected:Option<&EntryStamp>)->Result<EntryReadLease,String>{Err("Workspace-entry publication interlocks are unavailable on this engine platform.".into())}
#[cfg(not(unix))]pub(crate) fn default_entry_binding(_root:&std::path::Path)->Result<EntryStamp,String>{Err("Durable workspace-entry bindings are unavailable on this engine platform.".into())}
</ui:copy>

</ui:doc>
