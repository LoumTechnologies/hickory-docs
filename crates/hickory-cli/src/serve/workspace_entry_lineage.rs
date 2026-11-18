const MAX_ROWS:usize=256;
#[derive(Clone,Debug,PartialEq,Eq)]
struct Point {identity:String,generation:u32,path:Option<String>}
#[derive(Clone,Debug)]
enum Action {Move,Inverse(usize),Barrier}
#[derive(Clone,Debug)]
struct Step {id:String,before:Point,after:Point,action:Action}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct Atom {basis:usize,inverse:bool}
fn push(stack:&mut Vec<Atom>,atom:Atom){if stack.last().is_some_and(|last|last.basis==atom.basis&&last.inverse!=atom.inverse){stack.pop();}else{stack.push(atom);}}
#[derive(Clone,Debug)]
struct NoWrite { before: Point, after: Point }

fn continued_by_no_write(before: &Point, after: &Point, rows: &[NoWrite]) -> bool {
 if before == after { return true; }
 if before.identity != after.identity || before.path != after.path || before.generation >= after.generation ||
    usize::try_from(after.generation-before.generation).map_or(true, |span| span>rows.len()) {
  return false;
 }
 let mut current = before.clone();
 while current.generation < after.generation {
  let expected = Point { generation: current.generation+1, ..current.clone() };
  let mut matching = rows.iter().filter(|row| row.before == current);
  let Some(row) = matching.next() else { return false; };
  if row.after != expected || matching.next().is_some() { return false; }
  current = expected;
 }
 current == *after
}

fn trace_with_no_writes(rows:&[Step],target:usize,current:&Point,no_writes:&[NoWrite])->Result<(),String>{
 if rows.is_empty()||rows.len()>MAX_ROWS||target>=rows.len(){return Err("Entry lineage exceeds its bounded complete history".into());}
 let mut ids=std::collections::BTreeSet::new();let mut stack=Vec::new();let mut states=Vec::<Vec<Atom>>::new();let mut atoms=Vec::<Option<Atom>>::new();let mut epochs=Vec::new();let mut epoch=0usize;
 for (index,row) in rows.iter().enumerate(){
  let generation=if matches!(row.action,Action::Barrier){row.after.generation>row.before.generation}else{row.before.generation.checked_add(1)==Some(row.after.generation)};
  if !ids.insert(&row.id)||row.before.identity!=row.after.identity||!generation{return Err("Entry producer identity or generation is ambiguous".into());}
  if index>0 && !continued_by_no_write(&rows[index-1].after,&row.before,no_writes) {
   if index>target{return Err("Entry lineage crosses a missing or consumed generation".into());}
   epoch+=1;stack.clear();
  }
  let atom=match row.action {
   Action::Barrier=>{if index>=target{return Err("This entry lineage crosses an unsupported role transition".into());}epoch+=1;stack.clear();None},
   Action::Move=>{
    if row.before.path.is_none()||row.after.path.is_none()||row.before.path==row.after.path{return Err("Entry Move is not one exact address change".into());}
    Some(Atom{basis:index,inverse:false})
   },
   Action::Inverse(parent)=>{
    if parent>=index||epochs[parent]!=epoch||stack!=states[parent]||row.before.path!=rows[parent].after.path||row.after.path!=rows[parent].before.path{return Err("Retained inverse does not cancel its exact complete producer suffix".into());}
    let original=atoms[parent].ok_or("Retained inverse refers to an unsupported producer")?;
    Some(Atom{basis:original.basis,inverse:!original.inverse})
   }
  };
  if let Some(atom)=atom{push(&mut stack,atom);}
  states.push(stack.clone());atoms.push(atom);epochs.push(epoch);
 }
 if !continued_by_no_write(&rows.last().unwrap().after,current,no_writes) ||
    epochs[target]!=epoch||states[target]!=stack||atoms[target].is_none(){return Err("Current entry is not the selected producer after canceled descendants".into());}
 Ok(())
}
use super::{Binding, EntryStamp, TransitionKind, storage};
use crate::serve::{resource_inputs::SavedSource,resource_journal::Recovery};
use std::{collections::{BTreeMap,BTreeSet},path::Path};
const MAX_BYTES:usize=32*1024*1024;
/// Not serializable and not a historical record: minted from a held sealed frame.
pub(crate) struct Proof {
    frame_sha256: String,
    record_sha256: String,
    guards: BTreeMap<String, String>,
}
/// This borrow cannot outlive or mutate its original private publication frame.
pub(crate) struct Admission<'a>{proof:&'a Proof,bound:&'a storage::Bound}
fn encoded(value:&impl serde::Serialize)->Result<Vec<u8>,String>{serde_json::to_vec(value).map_err(|error|error.to_string())}
fn point(stamp:&EntryStamp)->Point{Point{identity:stamp.identity.clone(),generation:stamp.generation,path:stamp.path.clone()}}
fn scope(root:&Path,binding:&Binding)->Result<(),String>{
 if binding.pending.is_some(){return Err("Pending entry publication grants no older inverse authority".into());}
 if crate::serve::resource_inputs::RootIdentity::fingerprint(root)?!=binding.physical_root || root.canonicalize().map_err(|error|error.to_string())?.to_str()!=Some(binding.namespace.as_str()){return Err("Original entry physical root or namespace changed".into());}
 binding.current.validate()
}
fn file_keys(states:&BTreeMap<String,SavedSource>)->BTreeSet<&String>{states.iter().filter_map(|(path,state)|matches!(state,SavedSource::File{..}).then_some(path)).collect()}
fn validate_record(binding:&Binding,index:usize,record:&Recovery)->Result<(),String>{
 let row=&binding.history[index];row.change.validate_shape()?;row.change.validate(&record.manifest.reviewed_plan)?;
 if record.id!=row.request_id||record.manifest_sha256!=row.manifest_sha256||!record.all_paths_recorded_applied{return Err("Entry lineage requires every exact complete sealed producer".into());}
 let guards=row.namespace_guards.as_ref().ok_or("Historical entry row has no original complete producer namespace")?;
 if file_keys(&record.manifest.after)!=guards.keys().collect(){return Err("Sealed entry producer has an incomplete physical namespace".into());}
 if row.change.after.path.as_ref().and_then(|path|guards.get(path))!=row.after_guard.as_ref(){return Err("Sealed entry role guard differs from its complete producer namespace".into());}
 Ok(())
}
fn same_inverse(parent:&Recovery,child:&Recovery)->Result<bool,String>{
 let mut actual=child.manifest.reviewed_plan.clone();let expected=parent.manifest.reviewed_plan.inverse();
 actual.context=expected.context.clone();actual.title=expected.title.clone();
 // Current capture versions may advance; all identities, bytes, capabilities,
 // output versions, ordering and annotations must retain the exact inverse.
 if actual.patches.len()!=expected.patches.len(){return Ok(false);}
 for (a,b) in actual.patches.iter_mut().zip(&expected.patches){
  if let (crate::serve::resource_changes::SourceState::File{version:a,..},crate::serve::resource_changes::SourceState::File{version:b,..})=(&mut a.before,&b.before){*a=*b;}
 }
 Ok(encoded(&actual)?==encoded(&expected)?&&child.manifest.before==parent.manifest.after&&child.manifest.after==parent.manifest.before)
}
fn simple_move(binding:&Binding,index:usize,record:&Recovery)->Result<(),String>{
 let change=&binding.history[index].change;
 let from=change.before.path.as_ref().ok_or("Older entry lineage requires an original authored source")?;
 let to=change.after.path.as_ref().ok_or("Older entry lineage does not infer absent-role restoration")?;
 if from==to||record.manifest.before.len()!=2||record.manifest.after.len()!=2||record.manifest.reviewed_plan.patches.len()!=2
  ||!matches!(record.manifest.before.get(from),Some(SavedSource::File{..}))||record.manifest.before.get(to)!=Some(&SavedSource::Missing)
  ||record.manifest.after.get(from)!=Some(&SavedSource::Missing)||!matches!(record.manifest.after.get(to),Some(SavedSource::File{..})){
  return Err("Older entry lineage supports only complete exact Move/inverse namespaces".into());
 }
 Ok(())
}

fn sealed_no_writes(root:&Path,binding:&Binding,start:u32,cost:&mut usize)->Result<Vec<NoWrite>,String> {
 let mut verified=Vec::new();
 for row in binding.no_effects.iter().flatten() {
  if row.before.generation<start || row.after.generation>binding.current.generation {continue;}
  let record=crate::serve::resource_journal::read_recovery(root,&row.request_id)?;
  *cost=(*cost).saturating_add(encoded(&record)?.len());
  if *cost>MAX_BYTES {return Err("Entry receipts exceed the complete producer lineage budget".into());}
  if record.manifest_sha256!=row.manifest_sha256 || record.manifest.request_id.as_ref()!=Some(&row.request_id) ||
     record.all_paths_recorded_applied {
   return Err("Consumed entry reservation no longer has its exact no-write receipt".into());
  }
  verified.push(NoWrite{before:point(&row.before),after:point(&row.after)});
 }
 Ok(verified)
}

fn current_namespace(root:&Path,states:&BTreeMap<String,SavedSource>,guards:&BTreeMap<String,String>)->Result<(),String>{
 if file_keys(states)!=guards.keys().collect(){return Err("Transported entry namespace lost an original producer guard".into());}
 for (path,expected) in states {
  let (full,actual)=crate::serve::resource_inputs::saved_target(root,path)?;
  if &actual!=expected{return Err(format!("Producer namespace changed at {path}; equal old names are not authority"));}
  if let Some(guard)=guards.get(path){crate::serve::source_history::validate_observed_guard(root,&full,path,guard)?;}
 }
 Ok(())
}
fn transport(root:&Path,binding:&Binding,records:&[Option<Recovery>],target:usize)->Result<BTreeMap<String,String>,String>{
 let selected=records[target].as_ref().ok_or("Selected producer is unavailable")?;
 let mut states=selected.manifest.after.clone();let mut guards=binding.history[target].namespace_guards.clone().ok_or("Selected entry result has no sealed physical guards")?;
 for index in target+1..records.len(){
  let row=&binding.history[index];let record=records[index].as_ref().ok_or("Needed producer is unavailable")?;simple_move(binding,index,record)?;
  for (path,before) in &record.manifest.before {
   if let Some(previous)=states.get(path){if previous!=before{return Err("Intervening producer consumed different saved source material".into());}}
   else{states.insert(path.clone(),before.clone());}
  }
  let from=row.change.before.path.as_ref().ok_or("Entry source is absent")?;
  if guards.get(from)!=row.before_guard.as_ref(){return Err("Entry source was edited, replaced or adopted outside its sealed producer chain".into());}
  for (path,after) in &record.manifest.after {
   states.insert(path.clone(),after.clone());guards.remove(path);
   if let Some(guard)=row.namespace_guards.as_ref().and_then(|items|items.get(path)){guards.insert(path.clone(),guard.clone());}
  }
 }
 for (path,expected) in &selected.manifest.after{if states.get(path)!=Some(expected){return Err("Canceled producer chain does not restore the selected exact saved namespace".into());}}
 current_namespace(root,&states,&guards)?;
 Ok(guards.into_iter().filter(|(path,_)|selected.manifest.after.contains_key(path)).collect())
}
pub(super) fn prove(root:&Path,binding:&Binding,selected:&Recovery)->Result<Proof,String>{
 scope(root,binding)?;
 if binding.history.is_empty()||binding.history.len()>MAX_ROWS{return Err("Entry producer lineage exceeds its retained bound".into());}
 let frame=encoded(binding)?;if frame.len()>MAX_BYTES{return Err("Entry producer frame exceeds its reading budget".into());}
 let mut ids=BTreeMap::new();
 for(index,row)in binding.history.iter().enumerate(){if ids.insert(row.request_id.clone(),index).is_some(){return Err("Entry producer identities are ambiguous".into());}}
 let target=*ids.get(&selected.id).ok_or("Selected resource has no sealed entry producer")?;
 if binding.history[target].manifest_sha256!=selected.manifest_sha256{return Err("Selected entry manifest changed".into());}
 let mut records:Vec<Option<Recovery>>=(0..binding.history.len()).map(|_|None).collect();let mut cost=0usize;
 // An unrelated historical prefix is not needed authority. Extend backward
 // only when an exact Restoration in the required interval names an ancestor.
 let mut needed=target;let mut loaded=records.len();
 loop {
  for index in needed..loaded {
   let row=&binding.history[index];let record=crate::serve::resource_journal::read_recovery(root,&row.request_id)?;validate_record(binding,index,&record)?;
   cost=cost.saturating_add(encoded(&record)?.len());if cost>MAX_BYTES{return Err("Entry complete producer lineage exceeds its bounded reading".into());}
   records[index]=Some(record);
  }
  loaded=needed;let mut earlier=needed;
  for index in needed..records.len(){
   if binding.history[index].change.kind==TransitionKind::Restoration {
    let record=records[index].as_ref().ok_or("Required entry producer disappeared")?;
    let origin=record.manifest.restore_of.as_ref().ok_or("Entry inverse has no exact original producer")?;
    let parent=*ids.get(&origin.recovery_id).ok_or("Required inverse ancestry is missing")?;
    if parent>=index{return Err("Entry inverse ancestry is cyclic or forward-linked".into());}earlier=earlier.min(parent);
   }
  }
  if earlier==needed{break;}needed=earlier;
 }
 let selected_current=records[target].as_ref().ok_or("Selected entry producer is unavailable")?;
 if encoded(selected_current)?!=encoded(selected)?{return Err("Selected entry record or fingerprint changed".into());}
 simple_move(binding,target,selected_current)?;
 let mut rows=Vec::new();
 for (index,row) in binding.history.iter().enumerate(){
  let action=if let Some(record)=&records[index]{match row.change.kind {
   TransitionKind::Move=>{if record.manifest.restore_of.is_some(){return Err("Move producer carries a foreign restoration origin".into());}Action::Move},
   TransitionKind::Restoration=>{
    let origin=record.manifest.restore_of.as_ref().ok_or("Entry inverse has no exact original producer")?;
    let parent=*ids.get(&origin.recovery_id).ok_or("Entry inverse original producer is missing")?;
    let original=records[parent].as_ref().ok_or("Required inverse ancestry is unavailable")?;
    if parent>=index||original.fingerprint!=origin.fingerprint||!same_inverse(original,record)?{return Err("Entry inverse changed its complete original source/draft plan or receipt".into());}
    Action::Inverse(parent)
   },
   _=>Action::Barrier,
  }}else{Action::Barrier};
  rows.push(Step{id:row.request_id.clone(),before:point(&row.change.before),after:point(&row.change.after),action});
 }
 let first_generation = binding.history[needed].change.after.generation;
 let no_writes = sealed_no_writes(root, binding, first_generation, &mut cost)?;
 trace_with_no_writes(&rows,target,&point(&binding.current),&no_writes)?;
 // Only the latest original producer may establish current physical liveness.
 // Historical predecessors supply sealed edges, never refreshed source guards.
 let anchor=records.last().and_then(Option::as_ref).ok_or("Entry producer anchor is absent")?;
 crate::serve::source_history::validate_resource_after(root,anchor)?;
 let guards=transport(root,binding,&records,target)?;scope(root,binding)?;
 Ok(Proof {
     frame_sha256: super::digest(&frame),
     record_sha256: super::digest(&encoded(selected)?),
     guards,
 })
}
impl Proof {
    pub(super) fn check(&self, root: &Path, binding: &Binding, record: &Recovery) -> Result<(), String> {
        if self.frame_sha256 != super::digest(&encoded(binding)?)
            || self.record_sha256 != super::digest(&encoded(record)?)
        {
            return Err("Original entry lineage frame changed. Prepare a fresh review.".into());
        }
        let current = prove(root, binding, record)?;
        if current.guards != self.guards {
            return Err("Original producer physical guards changed. Prepare a fresh review.".into());
        }
        Ok(())
    }
    pub(super) fn admit<'a>(&'a self, bound: &'a storage::Bound) -> Result<Admission<'a>, String> {
        bound.unchanged()?;
        Ok(Admission { proof: self, bound })
    }
}
impl Admission<'_> {
    pub(crate) fn check(&self, root: &Path, record: &Recovery) -> Result<(), String> {
        self.bound.unchanged()?;
        self.proof.check(root, &self.bound.payload, record)?;
        self.bound.unchanged()
    }
}
#[cfg(test)]mod tests{
 use super::*;
 fn point(g:u32,path:&str)->Point{Point{identity:"role".into(),generation:g,path:Some(path.into())}}
 fn step(i:usize,a:&str,b:&str,action:Action)->Step{Step{id:format!("{i:032x}"),before:point(i as u32,a),after:point(i as u32+1,b),action}}
 fn chain()->Vec<Step>{vec![step(0,"a","b",Action::Move),step(1,"b","c",Action::Move),step(2,"c","b",Action::Inverse(1)),step(3,"b","a",Action::Inverse(0)),step(4,"a","b",Action::Inverse(3)),step(5,"b","c",Action::Inverse(2))]}
 fn trace(rows:&[Step],target:usize,current:&Point)->Result<(),String>{trace_with_no_writes(rows,target,current,&[])}
 #[test]fn sealed_no_writes_bridge_only_exact_unchanged_roles() {
  let mut rows=chain();
  rows[2].before.generation=3;rows[2].after.generation=4;
  let bridged=NoWrite{before:point(2,"c"),after:point(3,"c")};
  let terminal=NoWrite{before:point(4,"b"),after:point(5,"b")};
  assert!(trace_with_no_writes(&rows[..3],0,&point(5,"b"),&[bridged.clone(),terminal.clone()]).is_ok());
  assert!(trace_with_no_writes(&rows[..3],0,&point(5,"b"),&[bridged.clone()]).is_err());
  let foreign=NoWrite{before:point(2,"c"),after:point(3,"different")};
  assert!(trace_with_no_writes(&rows[..3],0,&point(5,"b"),&[foreign,terminal.clone()]).is_err());
  assert!(trace_with_no_writes(&rows[..3],0,&point(5,"b"),&[bridged.clone(),bridged,terminal]).is_err());
 }
 #[test]fn older_undo_and_redo_follow_exact_inverse_ancestry(){let rows=chain();assert!(trace(&rows[..2],1,&point(2,"c")).is_ok());assert!(trace(&rows[..3],0,&point(3,"b")).is_ok());assert!(trace(&rows[..4],3,&point(4,"a")).is_ok());assert!(trace(&rows[..5],2,&point(5,"b")).is_ok());assert!(trace(&rows,5,&point(6,"c")).is_ok());}
 #[test]fn independent_equal_path_cycle_is_not_cancellation(){let rows=vec![step(0,"a","b",Action::Move),step(1,"b","c",Action::Move),step(2,"c","b",Action::Move)];assert!(trace(&rows,0,&point(3,"b")).is_err());}
 #[test]fn wrong_inverse_after_repeated_path_is_rejected(){let rows=vec![step(0,"a","b",Action::Move),step(1,"b","a",Action::Move),step(2,"a","b",Action::Move),step(3,"b","a",Action::Inverse(0))];assert!(trace(&rows,3,&point(4,"a")).is_err());}
 #[test]fn real_repeated_path_moves_can_be_undone_in_order(){let rows=vec![step(0,"a","b",Action::Move),step(1,"b","a",Action::Move),step(2,"a","b",Action::Inverse(1))];assert!(trace(&rows,0,&point(3,"b")).is_ok());}
 #[test]fn cycles_forward_links_forks_and_wrong_live_stamp_refuse(){let mut rows=chain();rows[2].action=Action::Inverse(2);assert!(trace(&rows[..3],0,&point(3,"b")).is_err());let mut rows=chain();rows[1].before.identity="foreign".into();assert!(trace(&rows[..3],0,&point(3,"b")).is_err());assert!(trace(&chain()[..3],0,&point(4,"b")).is_err());}
 #[test]fn consumed_generations_and_barriers_cannot_be_crossed(){let mut rows=chain();rows[1].before.generation+=1;rows[1].after.generation+=1;assert!(trace(&rows[..3],0,&point(3,"b")).is_err());let mut rows=chain();rows[1].action=Action::Barrier;assert!(trace(&rows[..3],0,&point(3,"b")).is_err());}
 #[test]fn validated_barrier_jump_and_missing_ancestry_do_not_poison_a_later_chain(){let mut rows=vec![step(0,"old","a",Action::Barrier),step(1,"a","b",Action::Move),step(2,"b","c",Action::Move),step(3,"c","b",Action::Inverse(2))];rows[0].after.generation=4;for row in &mut rows[1..]{row.before.generation+=3;row.after.generation+=3;}assert!(trace(&rows,1,&point(7,"b")).is_ok());assert!(trace(&rows,0,&point(7,"b")).is_err());}
 #[test]fn earlier_nonmove_role_can_anchor_a_later_independent_chain(){let rows=vec![step(0,"old","a",Action::Barrier),step(1,"a","b",Action::Move),step(2,"b","c",Action::Move),step(3,"c","b",Action::Inverse(2))];assert!(trace(&rows,1,&point(4,"b")).is_ok());}
}
#[cfg(test)]mod inverse_tests {
 use super::*;
 use crate::serve::{resource_changes::{ResourcePlan,ResourcePatch,SourceState},resource_journal::{Manifest,RestorationOrigin}};
 fn draft(version:i32)->SourceState{SourceState::File{identity:"retained-editor".into(),text:"dirty draft".into(),writable:true,removable:true,version:Some(version)}}
 fn saved()->SavedSource{SavedSource::File{text:"saved baseline".into(),sha256:super::super::digest(b"saved baseline"),readonly:false,executable:false,mode:Some(0o644)}}
 fn original()->Recovery{
  Recovery{id:"1".repeat(32),manifest_sha256:"a".repeat(64),fingerprint:"b".repeat(64),manifest:Manifest{
   version:2,workspace:"workspace".into(),context:"original-context".into(),title:"Move".into(),restore_of:None,request_id:Some("1".repeat(32)),
   reviewed_plan:ResourcePlan{version:1,context:"original-context".into(),title:"Move".into(),patches:vec![ResourcePatch{path:"a.md".into(),before:draft(1),after:SourceState::Missing{creatable:true}},ResourcePatch{path:"b.md".into(),before:SourceState::Missing{creatable:true},after:draft(1)}],annotations:vec![]},
   before:BTreeMap::from([("a.md".into(),saved()),("b.md".into(),SavedSource::Missing)]),after:BTreeMap::from([("a.md".into(),SavedSource::Missing),("b.md".into(),saved())]),
  },receipts:vec![],unrecognized_files:vec![],all_paths_recorded_applied:true}
 }
 fn inverse(parent:&Recovery)->Recovery{
  let mut result=original();result.id="2".repeat(32);result.manifest.version=3;result.manifest.restore_of=Some(RestorationOrigin{recovery_id:parent.id.clone(),fingerprint:parent.fingerprint.clone()});
  result.manifest.before=parent.manifest.after.clone();result.manifest.after=parent.manifest.before.clone();result.manifest.reviewed_plan=parent.manifest.reviewed_plan.inverse();result.manifest.reviewed_plan.context="fresh-live-intent".into();
  for patch in &mut result.manifest.reviewed_plan.patches{if let SourceState::File{version,..}=&mut patch.before{*version=Some(99);}}
  result
 }
 #[test]fn inverse_refreshes_only_current_preparation_version_not_draft_identity_or_content(){
  let parent=original();let mut child=inverse(&parent);assert!(same_inverse(&parent,&child).unwrap());
  for patch in &mut child.manifest.reviewed_plan.patches{if let SourceState::File{identity,..}=&mut patch.before{*identity="same-text-new-editor".into();}}
  assert!(!same_inverse(&parent,&child).unwrap());
  let mut child=inverse(&parent);for patch in &mut child.manifest.reviewed_plan.patches{if let SourceState::File{text,..}=&mut patch.before{*text="changed draft".into();}}assert!(!same_inverse(&parent,&child).unwrap());
  let mut child=inverse(&parent);for patch in &mut child.manifest.reviewed_plan.patches{if let SourceState::File{version,..}=&mut patch.after{*version=Some(99);}}assert!(!same_inverse(&parent,&child).unwrap());
 }
 #[test]fn inverse_cannot_hide_saved_metadata_or_extra_operations(){
  let parent=original();let mut child=inverse(&parent);if let SavedSource::File{mode,..}=child.manifest.after.get_mut("a.md").unwrap(){*mode=Some(0o600);}assert!(!same_inverse(&parent,&child).unwrap());
  let mut child=inverse(&parent);child.manifest.reviewed_plan.annotations.push(serde_json::json!({"foreign":true}));assert!(!same_inverse(&parent,&child).unwrap());
  let mut child=inverse(&parent);child.manifest.reviewed_plan.patches.reverse();assert!(!same_inverse(&parent,&child).unwrap());
 }
}
