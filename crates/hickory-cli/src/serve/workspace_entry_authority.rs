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
struct Pending {request_id:String,manifest_sha256:String,change:EntryChange,after_guard:Option<String>,#[serde(default,skip_serializing_if="Option::is_none")] namespace_guards:Option<BTreeMap<String,String>>,#[serde(default,skip_serializing_if="Option::is_none")] before_guard:Option<String>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedTransition {request_id:String,manifest_sha256:String,change:EntryChange,after_guard:Option<String>,#[serde(default,skip_serializing_if="Option::is_none")] before_guard:Option<String>,#[serde(default,skip_serializing_if="Option::is_none")] namespace_guards:Option<BTreeMap<String,String>>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {version:u32,physical_root:String,namespace:String,physical_store:String,current:EntryStamp,bootstrap_default:bool,pending:Option<Pending>,history:Vec<RetainedTransition>,#[serde(default,skip_serializing_if="Option::is_none")] ownership:Option<claims::Inventory>}
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
mod claims {
    use super::*;
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::Path,
        sync::{Arc, LazyLock, Mutex},
        time::Instant,
    };
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub(super) struct Inventory {
        version: u32,
        entry: EntryStamp,
        outputs: BTreeSet<String>,
        volumes: BTreeSet<String>,
        root_volume: bool,
    }
    #[derive(Clone)]
    pub(crate) struct PositiveClaims {
        entry: String,
        outputs: BTreeSet<String>,
        volumes: BTreeSet<String>,
        root_volume: bool,
    }
    impl PositiveClaims {
        pub(crate) fn entry(&self) -> &str {
            &self.entry
        }
        pub(crate) fn outputs(&self) -> impl Iterator<Item = &String> {
            self.outputs.iter()
        }
        pub(crate) fn volumes(&self) -> impl Iterator<Item = &String> {
            self.volumes.iter()
        }
        pub(crate) fn root_volume(&self) -> bool {
            self.root_volume
        }
    }
    #[derive(Clone)]
    struct Current {
        root: String,
        entry: EntryStamp,
        inventory: Inventory,
        guards: BTreeMap<String, String>,
        at: Instant,
    }
    static CURRENT: LazyLock<Mutex<BTreeMap<std::path::PathBuf, Arc<Current>>>> =
        LazyLock::new(|| Mutex::new(BTreeMap::new()));
    impl Inventory {
        pub(super) fn validate(&self) -> Result<(), String> {
            self.entry.validate()?;
            if self.version != 1
                || self.entry.path.is_none()
                || self.outputs.len() > 10000
                || self.volumes.len() > 10000
            {
                return Err("Canonical ownership inventory has invalid bounds.".into());
            }
            for path in self.outputs.iter().chain(self.volumes.iter()) {
                super::super::resource_changes::validate_path(path)?;
            }
            Ok(())
        }
        fn projection(&self) -> Result<PositiveClaims, String> {
            self.validate()?;
            Ok(PositiveClaims {
                entry: self
                    .entry
                    .path
                    .clone()
                    .ok_or("Absent main has no output projection")?,
                outputs: self.outputs.clone(),
                volumes: self.volumes.clone(),
                root_volume: self.root_volume,
            })
        }
        fn union(&mut self, other: &Self) {
            self.outputs.extend(other.outputs.clone());
            self.volumes.extend(other.volumes.clone());
            self.root_volume |= other.root_volume;
        }
    }
    fn captured(
        root: &Path,
        entry: &EntryStamp,
        proposed: Option<(&str, &str)>,
    ) -> Result<Current, String> {
        let path = entry
            .path
            .as_ref()
            .ok_or("No canonical main is configured")?;
        let observed = super::entry_observation(root, path)?;
        let source = proposed
            .filter(|(changed, _)| *changed == path)
            .map(|(_, text)| text.to_owned())
            .unwrap_or(observed.source);
        if hick_lang::parse(&source)
            .map_err(|error| error.to_string())?
            .find_tags("project")
            .is_empty()
        {
            return Err("The retained main no longer declares its project; its previous ownership remains protected.".into());
        }
        let sources = proposed
            .filter(|(changed, _)| *changed != path)
            .map(|(path, text)| BTreeMap::from([(path.to_owned(), text.to_owned())]))
            .unwrap_or_default();
        let scan = super::super::source_tests::scan_with_namespace(
            root,
            &super::super::source_tests::Request {
                path: path.clone(),
                source,
                project_entry: Some(path.clone()),
                sources,
                capture: false,
            },
            None,
            &BTreeSet::new(),
        )?;
        let tree = scan
            .tree
            .as_ref()
            .ok_or("Canonical construction has no complete compiler tree")?;
        let base = Path::new(path).parent().unwrap_or(Path::new(""));
        let mut outputs = BTreeSet::new();
        for output in tree.entries().keys() {
            outputs.insert(super::super::source_tests::relative(base, output)?);
        }
        let mut volumes =
            super::super::resource_inputs::declared_volume_folders(root, &scan.source_texts)?;
        let root_volume = volumes.remove("");
        let inventory = Inventory {
            version: 1,
            entry: entry.clone(),
            outputs,
            volumes,
            root_volume,
        };
        inventory.validate()?;
        let mut guards = BTreeMap::new();
        for (path, expected) in &scan.source_texts {
            let observed =
                super::super::source_history::read_observed(root, &root.join(path), path)?;
            if proposed.is_none_or(|(changed, _)| changed != path) && observed.source != *expected {
                return Err("Canonical input changed during ownership capture.".into());
            }
            guards.insert(
                path.clone(),
                observed
                    .save_guard
                    .ok_or("Canonical input has no physical source guard")?,
            );
        }
        let value = Current {
            root: super::super::resource_inputs::RootIdentity::fingerprint(root)?,
            entry: entry.clone(),
            inventory,
            guards,
            at: Instant::now(),
        };
        if !current(root, &value, entry)? {
            return Err("Canonical input changed before ownership capture completed.".into());
        }
        Ok(value)
    }
    fn current(root: &Path, value: &Current, entry: &EntryStamp) -> Result<bool, String> {
        if &value.entry != entry
            || value.root != super::super::resource_inputs::RootIdentity::fingerprint(root)?
        {
            return Ok(false);
        }
        for (path, guard) in &value.guards {
            if !super::super::source_history::cached_source_guard_current(root, path, guard)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn actual(root: &Path, entry: &EntryStamp) -> Result<Arc<Current>, String> {
        let address = root.canonicalize().map_err(|error| error.to_string())?;
        let cached = CURRENT
            .lock()
            .map_err(|_| "Canonical ownership cache is unavailable")?
            .get(&address)
            .cloned();
        if let Some(value) = cached {
            if current(root, &value, entry)? {
                return Ok(value);
            }
        }
        let value = Arc::new(captured(root, entry, None)?);
        let mut values = CURRENT
            .lock()
            .map_err(|_| "Canonical ownership cache is unavailable")?;
        if values.len() >= 32 {
            if let Some(oldest) = values
                .iter()
                .min_by_key(|(_, value)| value.at)
                .map(|(path, _)| path.clone())
            {
                values.remove(&oldest);
            }
        }
        values.insert(address, value.clone());
        Ok(value)
    }
    pub(super) fn inventory(root: &Path, entry: &EntryStamp) -> Result<Inventory, String> {
        Ok(actual(root, entry)?.inventory.clone())
    }
    #[cfg(unix)]
    pub(crate) fn positive(root: &Path) -> Result<Option<PositiveClaims>, String> {
        let base = hickory_workspace::data_root().map_err(|error| error.to_string())?;
        let Some(bound) = super::storage::Bound::optional_under(&base, root)? else {
            return Ok(None);
        };
        if bound.payload.current.path.is_none() {
            return Ok(None);
        }
        if bound.payload.history.is_empty()
            && bound.payload.current.path.as_deref() == Some("project.md")
            && bound.payload.ownership.is_none()
        {
            return Ok(None);
        }
        let mut retained = bound.payload.ownership.clone();
        let fresh = actual(root, &bound.payload.current);
        match fresh {
            Ok(value) => {
                let mut inventory = value.inventory.clone();
                if let Some(before) = retained.take() {
                    inventory.union(&before);
                }
                inventory.projection().map(Some)
            }
            Err(error) => match retained {
                Some(inventory) => inventory.projection().map(Some),
                None => Err(error),
            },
        }
    }
    #[cfg(not(unix))]
    pub(crate) fn positive(_root: &Path) -> Result<Option<PositiveClaims>, String> {
        Ok(None)
    }
    pub(super) fn protect_body(root: &Path, path: &Path, source: &str) -> Result<(), String> {
        #[cfg(unix)]
        {
            let base = hickory_workspace::data_root().map_err(|error| error.to_string())?;
            let Some(bound) = super::storage::Bound::optional_under(&base, root)? else {
                return Ok(());
            };
            let Some(main) = &bound.payload.current.path else {
                return Ok(());
            };
            if bound.payload.history.is_empty()
                && main == "project.md"
                && bound.payload.ownership.is_none()
            {
                return Ok(());
            }
            let full = if path.exists() {
                path.canonicalize().map_err(|error| error.to_string())?
            } else {
                path.parent()
                    .ok_or("Source target has no parent")?
                    .canonicalize()
                    .map_err(|error| error.to_string())?
                    .join(path.file_name().ok_or("Source target has no name")?)
            };
            let canonical = root.canonicalize().map_err(|error| error.to_string())?;
            let Ok(relative) = full.strip_prefix(&canonical) else {
                return Ok(());
            };
            let relative = relative
                .to_str()
                .ok_or("Source target is not UTF-8")?
                .replace(std::path::MAIN_SEPARATOR, "/");
            if let Some(before) = &bound.payload.ownership {
                let projection = before.projection()?;
                if owns(&projection, &relative) {
                    return Err(
                        "A retained canonical generated claim owns this source target.".into(),
                    );
                }
            }
            let before = actual(root, &bound.payload.current);
            let input = before
                .as_ref()
                .map(|value| value.guards.contains_key(&relative))
                .unwrap_or(relative == *main);
            if let Ok(before) = &before {
                if owns(&before.inventory.projection()?, &relative) {
                    return Err(
                        "The actual canonical compiler owns this generated source target.".into(),
                    );
                }
            }
            if before.is_err() && relative != *main {
                return Err("Canonical ownership is incomplete; repair the retained main before authorizing other source writes. Readable source and drafts are retained.".into());
            }
            if input {
                let after = captured(root, &bound.payload.current, Some((&relative, source)))?;
                if owns(&after.inventory.projection()?, &relative) {
                    return Err("The proposed construction would generate its own edited source; no authored write was granted.".into());
                }
            }
        }
        Ok(())
    }
    fn owns(claims: &PositiveClaims, path: &str) -> bool {
        claims.root_volume
            || claims.outputs.contains(path)
            || claims.volumes.iter().any(|folder| {
                path == folder
                    || path
                        .strip_prefix(folder.as_str())
                        .is_some_and(|suffix| suffix.starts_with('/'))
            })
    }
}
pub(crate) use claims::PositiveClaims;
pub(crate) fn positive_entry_claims(
    root: &std::path::Path,
) -> Result<Option<PositiveClaims>, String> {
    claims::positive(root)
}
pub(crate) fn protect_source_body(
    root: &std::path::Path,
    path: &std::path::Path,
    source: &str,
) -> Result<(), String> {
    claims::protect_body(root, path, source)
}
fn history_transition_after_lineage(root:&std::path::Path,binding:&Binding,record:&super::resource_journal::Recovery,proof:&super::source_history::ResourceAfterProof)->Result<EntryChange,String>{
 proof.check(root,record)?;
 let known=binding.history.iter().find(|old|old.request_id==record.id&&old.manifest_sha256==record.manifest_sha256).ok_or("This retained record has no sealed workspace-entry transition authority")?;
 if binding.current.identity!=known.change.after.identity||binding.current.path!=known.change.after.path{return Err("Retained entry restoration no longer follows the same construction identity and address.".into())}
 let mut rebased=binding.clone();rebased.current=known.change.after.clone();let mut change=history_transition(&rebased,record)?;change.before=binding.current.clone();change.after=binding.current.advance(change.after.path)?;change.validate(&record.manifest.reviewed_plan.inverse())?;Ok(change)
}
#[cfg(unix)]
mod storage {
 use super::*;
 use super::super::private_record::{HeldRecord,Spec};
 const SPEC:Spec=Spec{namespace:"workspace-entry-v1",key:"entry.key",lock:"entry.lock",record:"entry.json",limit:32*1024*1024,physical_bucket:true};
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
   if let Some(inventory)=&payload.ownership{inventory.validate()?;}payload.current.validate()?;super::leases::validate_role_hint(root,&payload)?;
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
   let coordination=super::leases::coordination_under(base,root,true)?;Self::resolve_coordinated(base,root,&coordination)
  }
  pub(super) fn resolve_coordinated(base:&std::path::Path,root:&std::path::Path,coordination:&super::leases::Coordination)->Result<Self,String>{
   coordination.check_exclusive_for(root)?;
   let existing=HeldRecord::try_snapshot(base,root,SPEC)?;let absent=existing.is_none();drop(existing);
   if absent&&super::leases::role_loss_hint(root)?{return Err("Retained workspace-entry history has lost its original private binding. Restore that binding; no default entry was inferred.".into())}
   let held=HeldRecord::under(base,root,SPEC)?;let original=held.read()?;
   let mut bound=if original.is_some(){Self::decode(root,held,original)?}else{
    if !held.created_key{return Err("Workspace-entry binding is missing from its existing private store; missing metadata does not restore project.md authority.".into())}
    let path=legacy_main(root)?;let namespace=root.canonicalize().map_err(|error|error.to_string())?.to_str().ok_or("Workspace namespace is not exact UTF-8")?.to_owned();
    let payload=Binding{version:1,physical_root:held.root_id.clone(),namespace,physical_store:held.store_id.clone(),bootstrap_default:path.is_none(),current:initial_stamp(path)?,pending:None,history:vec![],ownership:None};
    let mut bound=Self{held,payload,original};bound.persist()?;bound
   };
   if bound.payload.bootstrap_default&&bound.payload.pending.is_none(){if let Some(path)=legacy_main(root)?{bound.payload.current=bound.payload.current.advance(Some(path))?;bound.payload.bootstrap_default=false;bound.persist()?;}}
   Ok(bound)
  }
  pub(super) fn optional_under(base:&std::path::Path,root:&std::path::Path)->Result<Option<Self>,String>{match HeldRecord::try_snapshot(base,root,SPEC)?{Some(held)=>{let original=held.read()?;Self::decode(root,held,original).map(Some)},None=>{if super::leases::role_loss_hint(root)?{Err("Original workspace-entry binding is unavailable while its retained transition hint remains. No missing-entry permission was granted.".into())}else{Ok(None)}}}}
  pub(super) fn snapshot_or_initialize(root:&std::path::Path)->Result<Self,String>{let base=hickory_workspace::data_root().map_err(|error|error.to_string())?;let bound=Self::snapshot_or_initialize_under(&base,root)?;if bound.payload.pending.as_ref().is_some_and(|pending|pending.namespace_guards.is_some()){drop(bound);super::transitions::recover_completed_under(&base,root)?;return Self::optional_under(&base,root)?.ok_or("Completed entry recovery lost its original binding".into())}Ok(bound)}
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
#[path="workspace_entry_transitions.rs"]pub(crate) mod transitions;
#[cfg(unix)]pub(crate) fn configured_entry(root:&std::path::Path)->Result<Option<String>,String>{let bound=storage::Bound::snapshot_or_initialize(root)?;if bound.payload.pending.is_some(){return Err("The retained main transition is unconfirmed. Read its recovery; no project.md fallback was selected.".into())}Ok(bound.payload.current.path.clone())}

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
#[cfg(unix)]
mod leases {
 use super::*;
 use std::{fs::File,path::Path};
 use super::super::private_record::{HeldRecord,Spec};
 const COORDINATION:Spec=Spec{namespace:"workspace-source-publication-v1",key:"coordination.key",lock:"coordination.lock",record:"coordination.state",limit:256,physical_bucket:true};
 pub(super) struct Coordination {held:HeldRecord,publication:File,exclusive:bool}
 impl Coordination {
  pub(super) fn check_exclusive_for(&self,root:&Path)->Result<(),String>{self.check()?;if !self.exclusive||self.held.root_id!=super::super::resource_inputs::RootIdentity::fingerprint(root)?{return Err("Workspace role mutation requires its own original exclusive root lease.".into())}Ok(())}
  pub(super) fn check(&self)->Result<(),String>{self.held.check_publication_lock(&self.publication)?;if self.held.read()?.is_some(){return Err("Source publication coordination contains unexpected record material.".into())}Ok(())}
 }
 pub(super) fn coordination_under(base:&Path,root:&Path,exclusive:bool)->Result<Coordination,String>{
  let held=match HeldRecord::try_snapshot(base,root,COORDINATION)?{
   Some(held)=>held,
   None=>{let initialized=HeldRecord::under(base,root,COORDINATION)?;if initialized.read()?.is_some(){return Err("Source publication coordination is not an empty lock provider.".into())}let publication=initialized.publication_lock(exclusive)?;drop(initialized);let held=HeldRecord::snapshot(base,root,COORDINATION)?;let lease=Coordination{held,publication,exclusive};lease.check()?;return Ok(lease)}
  };
  let publication=held.publication_lock(exclusive)?;let lease=Coordination{held,publication,exclusive};lease.check()?;Ok(lease)
 }
 pub(super) fn coordination(root:&Path,exclusive:bool)->Result<Coordination,String>{let base=hickory_workspace::data_root().map_err(|error|error.to_string())?;coordination_under(&base,root,exclusive)}
 #[derive(Serialize,Deserialize)]#[serde(deny_unknown_fields)]struct RoleLossHint{version:u32,identity:String,#[serde(default)]generation_floor:Option<u32>}
 fn role_hint(root:&Path)->Result<Option<RoleLossHint>,String>{
  let directory=root.join(".hick-cache");match std::fs::symlink_metadata(&directory){Err(error)if error.kind()==std::io::ErrorKind::NotFound=>return Ok(None),Err(error)=>return Err(error.to_string()),Ok(metadata)if !metadata.is_dir()||metadata.file_type().is_symlink()=>return Err("Workspace entry recovery metadata is not an ordinary directory.".into()),Ok(_)=>{}}
  let path=directory.join("workspace-entry-role.json");match std::fs::symlink_metadata(&path){Err(error)if error.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(error)=>Err(error.to_string()),Ok(_)=>{let bytes=super::super::resource_journal::read_regular(&path,4096)?;let hint:RoleLossHint=serde_json::from_slice(&bytes).map_err(|_|"Workspace-entry role hint is damaged; missing state cannot become a fresh default.")?;if hint.version!=1||hint.identity.len()!=32||!hint.identity.bytes().all(|byte|byte.is_ascii_digit()||(b'a'..=b'f').contains(&byte)){return Err("Workspace-entry role hint is unsupported; restore the original private binding.".into())}Ok(Some(hint))}}
 }
 pub(super) fn role_loss_hint(root:&Path)->Result<bool,String>{Ok(role_hint(root)?.is_some())}
 pub(super) fn validate_role_hint(root:&Path,binding:&Binding)->Result<(),String>{match role_hint(root)?{Some(hint)=>{if hint.identity!=binding.current.identity{return Err("Entry recovery hint belongs to a different retained construction identity.".into())}if let Some(floor)=hint.generation_floor{let expected=binding.pending.as_ref().map(|pending|pending.change.after.generation).unwrap_or(binding.current.generation);if floor!=expected{return Err("Entry generation conflicts with its retained recovery floor; an older sealed frame was not revived.".into())}}else if !binding.history.is_empty(){return Err("Retained entry transitions have no current recovery generation floor.".into())}Ok(())},None if !binding.history.is_empty()=>Err("Retained entry transition recovery hint is missing; no old frame was revived.".into()),None=>Ok(())}}

 pub(super) fn ordinary_root(root:&Path)->Result<(),String>{
  if crate::engine::private_policy::active().map_err(|error|error.to_string())?||crate::engine::private_policy::exact_private_root(root).map_err(|error|error.to_string())?{return Err("Ordinary workspace entry and generation are unavailable for a retained private-practice scope. Use its captured source provider.".into())}Ok(())
 }
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
fn source_coordination_under(base:&Path,root:&Path,path:&Path,source:&str,ordinary:Option<crate::engine::private_policy::Ordinary>)->Result<SourceCoordinationLease,String>{source_coordination_after_preflight(base,root,path,source,ordinary,||Ok(()))}
fn source_coordination_after_preflight(base:&Path,root:&Path,path:&Path,source:&str,ordinary:Option<crate::engine::private_policy::Ordinary>,before_lock:impl FnOnce()->Result<(),String>)->Result<SourceCoordinationLease,String>{
 let Some(relative)=source_target(root,path)? else{return Ok(SourceCoordinationLease{coordination:None,bound:None,_ordinary:ordinary})};
 // Until the captured compiler inventory proves this file unrelated, even
 // tag-free contributors may feed the main graph. Never authorize a shared
 // write merely because the edited text has no construction tags.
 let _=(relative,source);
 before_lock()?;
 let coordination=coordination_under(base,root,true)?;
 // Absence is meaningful only while holding the same exclusive provider.
 // A prior None must not conceal a newly prepared role publication.
 let bound=super::storage::Bound::optional_under(base,root)?;
 let lease=SourceCoordinationLease{coordination:Some(coordination),bound,_ordinary:ordinary};lease.check()?;Ok(lease)
}
pub(crate) fn acquire_source_coordination(root:&Path,path:&Path,source:&str)->Result<SourceCoordinationLease,String>{
 let ordinary=crate::engine::private_policy::ordinary();ordinary_root(root)?;
 #[cfg(not(test))]let base=hickory_workspace::data_root().map_err(|error|error.to_string())?;
 #[cfg(test)]let base=root.join(".entry-coordination-state");
 source_coordination_under(&base,root,path,source,Some(ordinary))
}
pub(crate) struct SourceBatchCoordinationLease{lease:SourceCoordinationLease,root:std::path::PathBuf,paths:std::collections::BTreeMap<std::path::PathBuf,String>}
impl SourceBatchCoordinationLease{
 pub(crate) fn check(&self)->Result<(),String>{self.lease.check()?;for(path,relative)in &self.paths{if source_target(&self.root,path)?.as_ref()!=Some(relative){return Err("An intended source address changed after batch admission. No namespace or outside source authority was renewed.".into())}}Ok(())}
}
fn source_batch_under(base:&Path,root:&Path,paths:&[std::path::PathBuf],ordinary:Option<crate::engine::private_policy::Ordinary>)->Result<SourceBatchCoordinationLease,String>{
 if paths.is_empty()||paths.len()>10000{return Err("Source batch coordination requires a bounded explicit path set.".into())}
 let mut admitted=std::collections::BTreeMap::new();
 for path in paths{let relative=source_target(root,path)?.ok_or("Source batch coordination requires exact in-workspace files, not independent outside addresses")?;if admitted.insert(path.clone(),relative).is_some(){return Err("Source batch coordination repeats an intended address.".into())}}
 let lease=source_coordination_under(base,root,&paths[0],"",ordinary)?;let result=SourceBatchCoordinationLease{lease,root:root.to_owned(),paths:admitted};result.check()?;Ok(result)
}
pub(crate) fn acquire_source_batch_coordination(root:&Path,paths:&[std::path::PathBuf])->Result<SourceBatchCoordinationLease,String>{
 let ordinary=crate::engine::private_policy::ordinary();ordinary_root(root)?;
 #[cfg(not(test))]let base=hickory_workspace::data_root().map_err(|error|error.to_string())?;
 #[cfg(test)]let base=root.join(".entry-coordination-state");
 source_batch_under(&base,root,paths,Some(ordinary))
}
pub(super) fn protects_main_namespace(root:&Path,main:&str,path:&str)->Result<bool,String>{
 // Case variants are conservatively protected even when the configured source
 // is temporarily absent. Existing physical aliases also cover hardlinks and
 // filesystem-specific equivalence; a text address never overrides that proof.
 let main_folded=main.to_lowercase();let path_folded=path.to_lowercase();
 if main_folded==path_folded||main_folded.strip_prefix(path_folded.as_str()).is_some_and(|suffix|suffix.starts_with('/')){return Ok(true)}
 let candidate=root.join(path);let main=root.join(main);let mut related=Some(main.as_path());
 while let Some(original)=related{match same_file::is_same_file(&candidate,original){Ok(true)=>return Ok(true),Ok(false)=>{},Err(error)if error.kind()==std::io::ErrorKind::NotFound=>{},Err(error)=>return Err(format!("Main construction alias identity is unavailable: {error}. No raw namespace authority was granted."))}if original==root{break}related=original.parent();if related.is_some_and(|parent|!parent.starts_with(root)){break}}
 Ok(false)
}
fn namespace_coordination_for_under(base:&Path,root:&Path,paths:&[&str],retired:&[&str],ordinary:Option<crate::engine::private_policy::Ordinary>)->Result<SourceCoordinationLease,String>{
 if paths.is_empty()||paths.len()>10000{return Err("Namespace coordination requires a bounded explicit affected-path set.".into())}
 let affected:std::collections::HashSet<_>=paths.iter().copied().collect();
 if retired.len()>paths.len()||retired.iter().any(|path|!affected.contains(path)){return Err("A retired source is outside the affected namespace.".into())}
 for path in paths{super::super::resource_changes::validate_path(path)?;}
 let coordination=coordination_under(base,root,true)?;
 let bound=super::storage::Bound::optional_under(base,root)?;
 let main=if retired.is_empty(){None}else{match bound.as_ref(){
  Some(bound) if !bound.payload.bootstrap_default=>bound.payload.current.path.clone(),
  _=>super::legacy_main(root)?,
 }};
 if let Some(main)=main{
  for path in retired{if protects_main_namespace(root,&main,path)?{return Err("The main construction or its parent requires an explicit reviewed workspace-entry transition. Inspect the current entry before changing these paths.".into())}}
 }
 let lease=SourceCoordinationLease{coordination:Some(coordination),bound,_ordinary:ordinary};lease.check()?;Ok(lease)
}
#[cfg(test)]
fn namespace_coordination_under(base:&Path,root:&Path,paths:&[&str],ordinary:Option<crate::engine::private_policy::Ordinary>)->Result<SourceCoordinationLease,String>{
 namespace_coordination_for_under(base,root,paths,paths,ordinary)
}
pub(crate) fn acquire_namespace_coordination_for(root:&Path,paths:&[&str],retired:&[&str])->Result<SourceCoordinationLease,String>{
 let ordinary=crate::engine::private_policy::ordinary();ordinary_root(root)?;
 #[cfg(not(test))]let base=hickory_workspace::data_root().map_err(|error|error.to_string())?;
 #[cfg(test)]let base=root.join(".entry-coordination-state");
 namespace_coordination_for_under(&base,root,paths,retired,Some(ordinary))
}
pub(crate) fn acquire_namespace_coordination(root:&Path,paths:&[&str])->Result<SourceCoordinationLease,String>{
 acquire_namespace_coordination_for(root,paths,paths)
}

#[cfg(test)]
mod namespace_retirement_tests {
 use super::*;
 #[test]
 fn content_revert_and_ordinary_removal_share_one_lease_without_retiring_main(){
  let temporary=tempfile::tempdir().unwrap();
  let root=temporary.path().join("workspace");let base=temporary.path().join("state");
  std::fs::create_dir(&root).unwrap();
  std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\"/></h:doc>").unwrap();
  drop(super::super::storage::Bound::resolve_under(&base,&root).unwrap());
  let paths=["project.md","ordinary.txt"];
  let lease=namespace_coordination_for_under(&base,&root,&paths,&["ordinary.txt"],None).unwrap();
  lease.check().unwrap();assert!(read_under(&base,&root,None,None).is_err());drop(lease);
  assert!(namespace_coordination_for_under(&base,&root,&paths,&["project.md"],None).is_err());
  assert!(namespace_coordination_for_under(&base,&root,&paths,&["foreign.txt"],None).is_err());
 }
 #[test]
 fn legacy_entry_candidate_is_protected_before_the_first_read_without_creating_a_role(){
  let temporary=tempfile::tempdir().unwrap();
  let root=temporary.path().join("workspace");let base=temporary.path().join("state");
  std::fs::create_dir(&root).unwrap();
  std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"part.md\"/></h:doc>").unwrap();
  assert!(namespace_coordination_for_under(&base,&root,&["project.md"],&["project.md"],None).is_err());
  assert!(!base.join("workspace-entry-v1").exists());
  std::fs::write(root.join("project.md"),"# ordinary note\n").unwrap();
  namespace_coordination_for_under(&base,&root,&["project.md"],&["project.md"],None).unwrap();
  assert!(!base.join("workspace-entry-v1").exists());
 }
}
 #[cfg(test)]
 mod tests {
  use super::*;
  fn fixture()->(tempfile::TempDir,std::path::PathBuf,std::path::PathBuf){let temporary=tempfile::tempdir().unwrap();let root=temporary.path().join("workspace");std::fs::create_dir(&root).unwrap();let root=root.canonicalize().unwrap();let base=temporary.path().join("state");(temporary,root,base)}
  #[test]fn explicit_read_never_initializes_a_main_and_shared_reads_do_not_block_each_other(){let(_temporary,root,base)=fixture();let first=read_under(&base,&root,None,None).unwrap();let second=read_under(&base,&root,None,None).unwrap();first.check().unwrap();second.check().unwrap();assert!(!first.matches_current(&EntryStamp{version:1,identity:"0".repeat(32),generation:0,path:Some("project.md".into())}));assert!(!base.join("workspace-entry-v1").exists());assert!(coordination_under(&base,&root,true).is_err());drop(first);drop(second);coordination_under(&base,&root,true).unwrap();}
  #[test]fn original_implicit_binding_must_be_present_and_exact(){let(_temporary,root,base)=fixture();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project/></h:doc>").unwrap();let bound=super::super::storage::Bound::resolve_under(&base,&root).unwrap();let stamp=bound.payload.current.clone();drop(bound);let lease=read_under(&base,&root,Some(&stamp),None).unwrap();lease.check().unwrap();assert!(lease.matches_current(&stamp));let stale=stamp.advance(stamp.path.clone()).unwrap();assert!(!lease.matches_current(&stale));assert!(read_under(&base,&root,Some(&stale),None).is_err());drop(lease);let other=root.parent().unwrap().join("other");std::fs::create_dir(&other).unwrap();assert!(read_under(&base,&other,Some(&stamp),None).is_err());}
  #[test]fn pending_binding_is_not_explicit_absence(){let(_temporary,root,base)=fixture();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project/></h:doc>").unwrap();let mut bound=super::super::storage::Bound::resolve_under(&base,&root).unwrap();let stamp=bound.payload.current.clone();bound.payload.pending=Some(Pending{request_id:"1".repeat(32),manifest_sha256:"2".repeat(64),change:EntryChange{version:1,kind:TransitionKind::Move,before:stamp.clone(),after:stamp.advance(Some("next.md".into())).unwrap(),source_identity:Some("owned".into())},after_guard:None,namespace_guards:None,before_guard:None});bound.persist().unwrap();drop(bound);assert!(read_under(&base,&root,None,None).is_err());}
  #[test]fn lost_or_inaccessible_role_store_refuses_instead_of_creating_explicit_authority(){let(_temporary,root,base)=fixture();let bound=super::super::storage::Bound::resolve_under(&base,&root).unwrap();let path=bound.held.path.clone();drop(bound);std::fs::remove_file(path.join("entry.json")).unwrap();assert!(read_under(&base,&root,None,None).is_err());let fresh_base=base.with_file_name("another-state");std::fs::create_dir(root.join(".hick-cache")).unwrap();std::fs::write(root.join(".hick-cache/workspace-entry-role.json"),serde_json::to_vec(&serde_json::json!({"version":1,"identity":"f".repeat(32)})).unwrap()).unwrap();assert!(read_under(&fresh_base,&root,None,None).is_err());assert!(!fresh_base.join("workspace-entry-v1").exists());std::fs::remove_file(root.join(".hick-cache/workspace-entry-role.json")).unwrap();std::fs::write(fresh_base.join("workspace-entry-v1"),"not a directory").unwrap();assert!(read_under(&fresh_base,&root,None,None).is_err());}

  #[test]fn tag_free_contributor_write_is_exclusive_until_inventory_proves_independence(){let(_temporary,root,base)=fixture();let path=root.join("part.md");std::fs::write(&path,"ordinary prose\r\n").unwrap();let reading=read_under(&base,&root,None,None).unwrap();assert!(source_coordination_under(&base,&root,&path,"new prose",None).is_err());assert!(!base.join("workspace-entry-v1").exists());drop(reading);let writing=source_coordination_under(&base,&root,&path,"new prose",None).unwrap();assert!(read_under(&base,&root,None,None).is_err());writing.check().unwrap();drop(writing);read_under(&base,&root,None,None).unwrap();}
  #[test]fn source_creation_and_malformed_canonical_repair_use_same_exclusive_provider(){let(_temporary,root,base)=fixture();std::fs::write(root.join("project.md"),"<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project/></h:doc>").unwrap();drop(super::super::storage::Bound::resolve_under(&base,&root).unwrap());std::fs::write(root.join("project.md"),"<h:doc").unwrap();let writing=source_coordination_under(&base,&root,&root.join("project.md"),"repaired source",None).unwrap();assert!(read_under(&base,&root,None,None).is_err());drop(writing);let reading=read_under(&base,&root,None,None).unwrap();assert!(source_coordination_under(&base,&root,&root.join("new.txt"),"new",None).is_err());drop(reading);source_coordination_under(&base,&root,&root.join("new.txt"),"new",None).unwrap();}
  #[test]fn independent_outside_source_does_not_borrow_workspace_role_or_block_reading(){let(_temporary,root,base)=fixture();let outside=root.parent().unwrap().join("outside.txt");std::fs::write(&outside,"outside").unwrap();let reading=read_under(&base,&root,None,None).unwrap();let writing=source_coordination_under(&base,&root,&outside,"outside edit",None).unwrap();writing.check().unwrap();assert!(!base.join("workspace-entry-v1").exists());std::os::unix::fs::symlink(&outside,root.parent().unwrap().join("alias.txt")).unwrap();assert!(source_coordination_under(&base,&root,&root.parent().unwrap().join("alias.txt"),"edit",None).is_err());std::fs::hard_link(&outside,root.join("hardlink.txt")).unwrap();assert!(source_coordination_under(&base,&root,&outside,"edit",None).is_err());reading.check().unwrap();}
  #[test]fn write_lease_refuses_role_record_mutation_instead_of_blessing_equal_source(){let(_temporary,root,base)=fixture();let bound=super::super::storage::Bound::resolve_under(&base,&root).unwrap();let record=bound.held.path.join("entry.json");drop(bound);let writing=source_coordination_under(&base,&root,&root.join("plain.txt"),"new",None).unwrap();let mut bytes=std::fs::read(&record).unwrap();bytes.push(b' ');std::fs::write(&record,&bytes).unwrap();assert!(writing.check().is_err());}

  #[test]fn direct_namespace_operations_cannot_retire_a_bound_non_markdown_main_or_its_parent(){let(_temporary,root,base)=fixture();std::fs::create_dir(root.join("docs")).unwrap();let mut bound=super::super::storage::Bound::resolve_under(&base,&root).unwrap();bound.payload.current=bound.payload.current.advance(Some("docs/main.spec".into())).unwrap();bound.payload.bootstrap_default=false;bound.persist().unwrap();drop(bound);for path in ["docs/main.spec","docs"]{assert!(namespace_coordination_under(&base,&root,&[path],None).is_err())}namespace_coordination_under(&base,&root,&["docs2"],None).unwrap();}
  #[test]fn direct_namespace_admission_is_nonblocking_without_creating_a_role(){let(_temporary,root,base)=fixture();let reading=read_under(&base,&root,None,None).unwrap();assert!(namespace_coordination_under(&base,&root,&["new"],None).is_err());assert!(!base.join("workspace-entry-v1").exists());drop(reading);let namespace=namespace_coordination_under(&base,&root,&["from","to"],None).unwrap();assert!(read_under(&base,&root,None,None).is_err());namespace.check().unwrap();drop(namespace);read_under(&base,&root,None,None).unwrap();}


  #[test]fn one_content_batch_holds_all_paths_without_self_conflict_and_keeps_main_eligible(){let(_temporary,root,base)=fixture();let mut bound=super::super::storage::Bound::resolve_under(&base,&root).unwrap();bound.payload.current=bound.payload.current.advance(Some("main.spec".into())).unwrap();bound.payload.bootstrap_default=false;bound.persist().unwrap();drop(bound);std::fs::write(root.join("main.spec"),"original").unwrap();std::fs::write(root.join("part.txt"),"original").unwrap();let batch=source_batch_under(&base,&root,&[root.join("main.spec"),root.join("part.txt")],None).unwrap();batch.check().unwrap();assert!(read_under(&base,&root,None,None).is_err());drop(batch);assert!(namespace_coordination_under(&base,&root,&["main.spec"],None).is_err());read_under(&base,&root,None,None).unwrap();}
  #[test]fn blocked_or_aliased_content_batch_does_not_create_a_role_or_select_an_outside_file(){let(_temporary,root,base)=fixture();let paths=[root.join("one.txt"),root.join("two.txt")];for path in &paths{std::fs::write(path,"original").unwrap()}let reading=read_under(&base,&root,None,None).unwrap();assert!(source_batch_under(&base,&root,&paths,None).is_err());assert!(!base.join("workspace-entry-v1").exists());drop(reading);assert!(source_batch_under(&base,&root,&[paths[0].clone(),paths[0].clone()],None).is_err());let outside=root.parent().unwrap().join("outside.txt");std::fs::write(&outside,"outside").unwrap();assert!(source_batch_under(&base,&root,&[outside.clone()],None).is_err());std::os::unix::fs::symlink(outside,&paths[0].with_file_name("alias.txt")).unwrap();assert!(source_batch_under(&base,&root,&[root.join("alias.txt")],None).is_err());}


  #[test]fn direct_namespace_refuses_main_physical_aliases_and_case_spellings_without_broad_prefix_claims(){let(_temporary,root,base)=fixture();std::fs::create_dir(root.join("docs")).unwrap();std::fs::write(root.join("docs/main.spec"),"main").unwrap();let mut bound=super::super::storage::Bound::resolve_under(&base,&root).unwrap();bound.payload.current=bound.payload.current.advance(Some("docs/main.spec".into())).unwrap();bound.payload.bootstrap_default=false;bound.persist().unwrap();drop(bound);std::fs::hard_link(root.join("docs/main.spec"),root.join("same-inode.txt")).unwrap();std::os::unix::fs::symlink(root.join("docs"),root.join("folder-alias")).unwrap();std::os::unix::fs::symlink("cycle",root.join("cycle")).unwrap();for path in ["same-inode.txt","folder-alias","docs/MAIN.SPEC","DOCS","cycle"]{assert!(namespace_coordination_under(&base,&root,&[path],None).is_err(),"{path}")}std::fs::remove_file(root.join("docs/main.spec")).unwrap();assert!(namespace_coordination_under(&base,&root,&["docs/MAIN.SPEC"],None).is_err());namespace_coordination_under(&base,&root,&["docs-sibling"],None).unwrap();}


  #[test]fn pending_role_created_after_preflight_is_observed_under_the_exclusive_source_lease(){let(_temporary,root,base)=fixture();let path=root.join("plain.txt");std::fs::write(&path,"original").unwrap();assert!(super::super::storage::Bound::optional_under(&base,&root).unwrap().is_none());let admission=source_coordination_after_preflight(&base,&root,&path,"new",None,||{let mut bound=super::super::storage::Bound::resolve_under(&base,&root)?;let before=bound.payload.current.clone();bound.payload.pending=Some(Pending{request_id:"1".repeat(32),manifest_sha256:"2".repeat(64),change:EntryChange{version:1,kind:TransitionKind::Create,before:before.clone(),after:before.advance(Some("project.md".into()))?,source_identity:Some("owned".into())},after_guard:None,namespace_guards:None,before_guard:None});bound.persist()});assert!(admission.is_err());assert_eq!(std::fs::read_to_string(path).unwrap(),"original");assert!(super::super::storage::Bound::optional_under(&base,&root).unwrap().unwrap().payload.pending.is_some());}

 }
}
#[cfg(unix)]
pub(crate) use leases::{EntryReadLease,SourceCoordinationLease,SourceBatchCoordinationLease,acquire_source_batch_coordination,acquire_source_coordination,acquire_namespace_coordination,acquire_namespace_coordination_for,acquire_entry_read_lease,default_entry_binding};
#[cfg(not(unix))]pub(crate) struct SourceCoordinationLease;
#[cfg(not(unix))]impl SourceCoordinationLease{pub(crate) fn check(&self)->Result<(),String>{Ok(())}}
#[cfg(not(unix))]pub(crate) fn acquire_source_coordination(_root:&std::path::Path,_path:&std::path::Path,_source:&str)->Result<SourceCoordinationLease,String>{Ok(SourceCoordinationLease)}
#[cfg(not(unix))]pub(crate) fn acquire_namespace_coordination(_root:&std::path::Path,_paths:&[&str])->Result<SourceCoordinationLease,String>{Ok(SourceCoordinationLease)}
#[cfg(not(unix))]pub(crate) fn acquire_namespace_coordination_for(_root:&std::path::Path,_paths:&[&str],_retired:&[&str])->Result<SourceCoordinationLease,String>{Ok(SourceCoordinationLease)}
#[cfg(not(unix))]pub(crate) struct SourceBatchCoordinationLease;
#[cfg(not(unix))]impl SourceBatchCoordinationLease{pub(crate) fn check(&self)->Result<(),String>{Ok(())}}
#[cfg(not(unix))]pub(crate) fn acquire_source_batch_coordination(_root:&std::path::Path,_paths:&[std::path::PathBuf])->Result<SourceBatchCoordinationLease,String>{Ok(SourceBatchCoordinationLease)}
#[cfg(not(unix))]pub(crate) struct EntryReadLease;
#[cfg(not(unix))]impl EntryReadLease{pub(crate) fn matches_current(&self,_expected:&EntryStamp)->bool{false}pub(crate) fn check(&self)->Result<(),String>{Err("Workspace-entry publication interlocks are unavailable on this engine platform.".into())}}
#[cfg(not(unix))]pub(crate) fn acquire_entry_read_lease(_root:&std::path::Path,_expected:Option<&EntryStamp>)->Result<EntryReadLease,String>{Err("Workspace-entry publication interlocks are unavailable on this engine platform.".into())}
#[cfg(not(unix))]pub(crate) fn default_entry_binding(_root:&std::path::Path)->Result<EntryStamp,String>{Err("Durable workspace-entry bindings are unavailable on this engine platform.".into())}
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
 fn forged_selection_fields_and_unconfirmed_publication_refuse_whole_reading_scope(){let(_temporary,root,base)=fixture(true);let identity=super::super::resource_inputs::RootIdentity::bind(&root);let proof=issue(&base,&root,&identity);for mutation in ["namespace","source","generation","effects"]{let mut forged=proof.clone();match mutation{"namespace"=>forged.namespace.push_str("/other"),"source"=>forged.source_hash=Some(format!("sha256:{}","0".repeat(64))),"generation"=>forged.entry.generation+=1,_=>forged.source_effects=true};assert!(validate(&base,&root,&identity,&forged).is_err());}let mut bound=storage::Bound::open_under(&base,&root,true).unwrap();bound.payload.pending=Some(Pending{request_id:"1".repeat(32),manifest_sha256:"2".repeat(64),change:EntryChange{version:1,kind:TransitionKind::Move,before:proof.entry.clone(),after:proof.entry.advance(Some("next.md".into())).unwrap(),source_identity:Some("original".into())},after_guard:None,namespace_guards:None,before_guard:None});bound.persist().unwrap();assert!(issue_selection(&root,&identity,&bound).is_err());drop(bound);assert!(validate(&base,&root,&identity,&proof).is_err());}
}
