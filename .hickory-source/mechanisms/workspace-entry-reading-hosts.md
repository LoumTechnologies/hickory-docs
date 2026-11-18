<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Freeze the optional workspace-entry reading before compiler work

For maintainers integrating entry-dependent capabilities with the existing simple hosts. Capture the original editor and retained-source roster first; only a capability that needs the main closure awaits this reader. Source content remains available independently. An explicitly selected projection stays explicit rather than being replaced with an automatic main.

These adapters validate the versioned shape and expected runtime/namespace, retain immutable original material, and keep the caller's original current-context predicate. That predicate reports local editor/source ownership; it is not a remote-entry freshness claim. The backend must validate the original entry proof before and after compiler work, including Apply preparation. Neither a callback nor a menu command grants source publication authority.

Move and its inverse use the same entry reading. The server returns an exact
intent for the current generation and destination. The host carries its token
through the resource preview and accepts the save only when the receipt confirms
that same transition. A recovery record supplies the historical identity and
addresses. Its inverse still needs a fresh entry proof and complete review.
An older record may ask for review when the current entry has returned to that
address. Only the engine can establish that complete intervening publications
returned the same producer there.

## Browser: retain exact typed proof material

Retain the exact original scope. Reject unsupported fields, literal-type mismatches, invalid generations and incomplete configured-source observations. Returning JSON must not expose mutable storage that can retarget the retained selection; a late response cannot borrow a newer source lease.

<ui:copy id="workspace-entry-reading-browser-helper">
export type EntryStamp=Readonly<{version:1;identity:string;generation:number;path:string|null}>;
export type EntrySelection=Readonly<{version:1;entry_protocol:1;workspace:string;namespace:string;entry:EntryStamp;guard:string;source_guard?:string;source_hash?:string;movable:boolean;source_effects:false}>;
export type FrozenEntrySelection=Readonly<{entryPath:string|null;proof:EntrySelection;current:()=>boolean}>;
export type EntryReadingLease={workspace:string;namespace:string;current:()=>boolean};
function object(value:unknown):Record<string,unknown>{if(!value||typeof value!=="object"||Array.isArray(value))throw new Error("Workspace entry response is malformed.");return value as Record<string,unknown>;}
function hex(value:unknown,size:number):value is string{return typeof value==="string"&&value.length===size&&/^[0-9a-f]+$/.test(value);}
function entryStamp(value:unknown):EntryStamp {
 const entry=object(value),path=entry.path;
 if(Object.keys(entry).some(key=>!["version","identity","generation","path"].includes(key))||entry.version!==1||!hex(entry.identity,32)||typeof entry.generation!=="number"||!Number.isInteger(entry.generation)||entry.generation<0||entry.generation>0xffffffff)throw new Error('Workspace entry stamp is malformed.');
 if(path!==null&&(typeof path!=="string"||!path||path.includes('\\')||path.includes(':')||path.split('/').some(part=>!part||part==='.'||part==='..')||/[\u0000-\u001f\u007f]/.test(path)))throw new Error("Workspace entry path is outside the supported source namespace.");
 return Object.freeze({version:1,identity:entry.identity,generation:entry.generation,path});
}
function sameEntry(left:EntryStamp,right:EntryStamp):boolean {
 return left.identity===right.identity&&left.generation===right.generation&&left.path===right.path;
}
export function parseEntrySelection(value:unknown):EntrySelection{
 const body=object(value),stamp=entryStamp(body.entry),path=stamp.path;
 if(Object.keys(body).some(key=>!["version","entry_protocol","workspace","namespace","entry","guard","source_guard","source_hash","movable","source_effects"].includes(key)))throw new Error("Workspace entry response has unsupported fields.");
 if(body.version!==1||body.entry_protocol!==1||!hex(body.workspace,32)||typeof body.namespace!=="string"||!body.namespace||!hex(body.guard,64)||body.source_effects!==false||typeof body.movable!=="boolean")throw new Error("Workspace entry response has no supported exact authority.");
 if(path===null){if(body.source_guard!==undefined||body.source_hash!==undefined||body.movable)throw new Error("An absent entry cannot carry source authority.");}
 else if(typeof body.source_guard!=="string"||!body.source_guard||new TextEncoder().encode(body.source_guard).length>16384||typeof body.source_hash!=="string"||!/^sha256:[0-9a-f]{64}$/.test(body.source_hash))throw new Error("Workspace entry lacks an original physical source proof.");
 return Object.freeze({version:1 as const,entry_protocol:1 as const,workspace:body.workspace,namespace:body.namespace,entry:stamp,guard:body.guard,...(path===null?{}:{source_guard:body.source_guard as string,source_hash:body.source_hash as string}),movable:body.movable,source_effects:false as const});
}
export async function readFrozenEntrySelection(load:()=>Promise<unknown>,lease:EntryReadingLease):Promise<FrozenEntrySelection>{
 if(!lease.current())throw new Error("Original source context changed before entry acquisition.");
 const proof=parseEntrySelection(await load());
 if(!lease.current()||proof.workspace!==lease.workspace||proof.namespace!==lease.namespace)throw new Error("Workspace entry response belongs to an obsolete source context.");
 return Object.freeze({entryPath:proof.entry.path,proof,current:()=>lease.current()});
}

export type EntryOperation =
 | {kind:'move';destination:string}
 | {kind:'restoration';recovery_id:string;recovery_fingerprint:string};
export type EntryIntent = Readonly<{context:string;token:string;title:string;edit:unknown;before:EntryStamp;after:EntryStamp;interrupted?:InterruptedEntry}>;

function parseEntryIntent(value:unknown,proof:EntrySelection,operation:EntryOperation,expectedPath:string|null):EntryIntent {
 const body=object(value),before=entryStamp(body.before_entry),after=entryStamp(body.after_entry);
 if(body.version!==1||body.workspace!==proof.workspace||body.namespace!==proof.namespace||
    body.kind!==operation.kind||body.source_files_modified!==false||!hex(body.token,64)||body.context!==body.token||
    !sameEntry(before,proof.entry)||after.identity!==before.identity||after.generation!==before.generation+1||
    typeof body.title!=='string'||!body.title||after.path!==expectedPath) {
  throw new Error('The entry review did not retain the requested construction and address.');
 }
 const edit=JSON.stringify(object(body.edit));
 return Object.freeze({context:body.token,token:body.token,title:body.title,before,after,get edit(){return JSON.parse(edit);}});
}

export async function prepareEntryIntent(selection:FrozenEntrySelection,operation:EntryOperation,load:(body:unknown)=>Promise<unknown>,expectedPath:string|null,current:()=>boolean):Promise<EntryIntent> {
 const check=()=>{if(!selection.current()||!current())throw new Error('The original workspace entry or source review changed. Open the move again.');};
 check();
 const reply=await load({version:1,entry_selection:selection.proof,operation});
 check();
 return parseEntryIntent(reply,selection.proof,operation,expectedPath);
}

// A sealed interrupted reading can outlive the first request and server process.
// Its exact original body is resubmitted once when preparing a new review.
export type InterruptedEntry = Readonly<{
  workspace: string;
  namespace: string;
  request: string;
  fingerprint: string;
  manifestHash: string;
  source: string;
  destination: string;
  currentEntry: EntryStamp;
  reservationFloor: number;
  canComplete: boolean;
  notice: string | null;
  missingOriginal: boolean;
  body: () => unknown;
}>;

export function readInterruptedEntry(value: unknown, workspace: string, namespace: string): InterruptedEntry | null {
  const response = object(value);
  if (response.version !== 1 || !hex(workspace, 32) || !namespace || response.workspace !== workspace ||
      response.namespace !== namespace || response.source_files_modified !== false) {
    throw new Error('Interrupted construction evidence belongs to a different workspace.');
  }
  if (response.interrupted === null) return null;
  const reading = object(response.interrupted);
  const material = object(reading.material);
  const change = object(material.change);
  if(change.kind!=='move'&&material.can_complete===false&&typeof material.notice==='string'&&material.notice)throw new Error(material.notice);
  const currentEntry = entryStamp(material.current_entry);
  const before = entryStamp(change.before);
  const after = entryStamp(change.after);
  const floor = material.reservation_floor;
  if (material.version !== 1 || material.workspace !== workspace || material.namespace !== namespace ||
      material.source_files_modified !== false || !hex(reading.fingerprint, 64) || !hex(material.request_id, 32) ||
      !hex(material.manifest_sha256, 64) || typeof material.can_complete !== 'boolean' ||
      !(material.notice === null && material.can_complete || typeof material.notice === 'string' && material.notice) || typeof floor !== 'number' ||
      !Number.isInteger(floor) || floor < after.generation || floor > 0xffffffff ||
      !sameEntry(before, currentEntry) || before.identity !== after.identity || after.generation !== before.generation + 1 ||
      !before.path || !after.path || before.path === after.path || change.version !== 1 || change.kind !== 'move' ||
      !Array.isArray(material.paths) || material.paths.length !== 2) {
    throw new Error('Interrupted construction evidence does not describe one exact Move.');
  }
  const states = material.paths.map(path => object(path));
  const original = states.find(state => state.path === before.path);
  if (!original || !states.some(state => state.path === after.path) ||
      states.some(state => state.saved_kind !== 'file' && state.saved_kind !== 'missing' &&
        (state.saved_kind !== 'unknown' || material.can_complete))) {
    throw new Error('Interrupted construction evidence has invalid source paths.');
  }
  const snapshot = JSON.stringify(reading);
  return Object.freeze({
    workspace, namespace, request: material.request_id, fingerprint: reading.fingerprint, manifestHash: material.manifest_sha256,
    source: before.path, destination: after.path, currentEntry, reservationFloor: floor,
    canComplete: material.can_complete, notice: material.notice, missingOriginal: original.saved_kind === 'missing',
    body: () => JSON.parse(snapshot)
  });
}

export async function prepareInterruptedEntry(
  reading: InterruptedEntry, retainedCapture: unknown, load: (body: unknown) => Promise<unknown>, current: () => boolean
): Promise<EntryIntent> {
  if (!reading.canComplete || !current()) throw new Error('Refresh the interrupted Move evidence before preparing completion.');
  const reply = object(await load({version: 1, reading: reading.body(), retained_capture: retainedCapture}));
  const before = entryStamp(reply.before_entry), after = entryStamp(reply.after_entry);
  if (!current() || reply.version !== 1 || reply.kind !== 'interrupted_completion' ||
      reply.workspace !== reading.workspace || reply.namespace !== reading.namespace || reply.source_files_modified !== false ||
      reply.interrupted_reading_fingerprint !== reading.fingerprint || reply.supersedes !== null ||
      !hex(reply.token, 64) || reply.context !== reply.token || typeof reply.title !== 'string' || !reply.title ||
      !sameEntry(before, reading.currentEntry) || after.identity !== before.identity ||
      after.generation !== reading.reservationFloor + 1 || after.path !== reading.destination) {
    throw new Error('The interrupted Move review did not retain the original evidence and intended destination.');
  }
  const edit = JSON.stringify(object(reply.edit));
  return Object.freeze({context: reply.token, token: reply.token, title: reply.title, before, after,
    interrupted: reading, get edit() { return JSON.parse(edit); }});
}

export function readEntryTransition(value:unknown,recovery:string):{before:EntryStamp;after:EntryStamp}|null {
 if(value===null||value===undefined)return null;
 const body=object(value);
 if(body.version===1&&body.state==='unavailable'&&typeof body.notice==='string'&&body.notice)throw new Error(body.notice);
 if(body.version!==1||body.state!=='retained'||body.request_id!==recovery||!hex(body.manifest_sha256,64)||
    typeof body.kind!=='string'||!['move','restoration','create','repair'].includes(body.kind))throw new Error('This recovery record lacks a supported entry transition.');
 const before=entryStamp(body.before_entry),after=entryStamp(body.after_entry);
 if(before.identity!==after.identity||before.generation+1!==after.generation)throw new Error('The retained entry transition is inconsistent.');
 return {before,after};
}

export function requireEntryRestorationScope(selection:FrozenEntrySelection,after:EntryStamp):void {
 const current=selection.proof.entry;
 if(current.identity!==after.identity||current.path!==after.path||current.generation<after.generation) {
  throw new Error('The retained transition does not match the current construction. Restore later changes first.');
 }
}

export function confirmEntryOutcome(value:unknown,request:string,intent:EntryIntent|undefined):void {
 if(!intent){if(value!==undefined)throw new Error('Unexpected main construction publication.');return;}
 const body=object(value);
 const interrupted=intent.interrupted;
 if(body.version!==(interrupted?2:1)||body.state!=='confirmed'||body.request_id!==request||!sameEntry(entryStamp(body.before_entry),intent.before)||
    !sameEntry(entryStamp(body.after_entry),intent.after)||!sameEntry(entryStamp(body.current_entry),intent.after))throw new Error('The original main construction publication is unconfirmed.');
 if(interrupted&&(body.kind!=='interrupted_completion'||body.reservation_floor_before!==interrupted.reservationFloor||
     body.reservation_floor!==intent.after.generation||body.remaining_interrupted!==null||body.reservation_consumed!==true)) {
  throw new Error('The receipt does not confirm completion of the reviewed interruption.');
 }
}

export function entryPublicationHadNoEffect(value: unknown, request: string, intent: EntryIntent | undefined): boolean {
  if (!value || typeof value !== 'object' || !('state' in value) || value.state !== 'no_effect') return false;
  const body = object(value), consumed = body.reservation_consumed;
  const interrupted=intent?.interrupted;
  if (!intent || body.version !== (interrupted?2:1) || body.request_id !== request || typeof consumed !== 'boolean' ||
      !sameEntry(entryStamp(body.before_entry), intent.before) || !sameEntry(entryStamp(body.after_entry), intent.after)) {
    throw new Error('The no-effect receipt does not match the original entry review.');
  }
  if(interrupted){
   const remaining=object(body.remaining_interrupted);
   if(body.kind!=='interrupted_completion'||body.reservation_floor_before!==interrupted.reservationFloor||
      body.reservation_floor!==interrupted.reservationFloor+(consumed?1:0)||
      remaining.request_id!==interrupted.request||remaining.manifest_sha256!==interrupted.manifestHash||
      !sameEntry(entryStamp(body.current_entry),intent.before))throw new Error('The prior interrupted Move remains unconfirmed.');
   return true;
  }
  const retained = { ...intent.before, generation: intent.before.generation + (consumed ? 1 : 0) };
  if (!sameEntry(entryStamp(body.current_entry), retained)) throw new Error('The original entry namespace is unconfirmed.');
  return true;
}
</ui:copy>

## Browser: refuse incomplete authority and obsolete replies

The witness checks mutation isolation, strict wire types, positive no-main versus incomplete source authority, and obsolete source context before or during the optional request. It exercises models, not physical input or rendered accessibility.

<ui:copy id="workspace-entry-reading-browser-witness">
import {describe,it,expect} from "vitest";
import {confirmEntryOutcome,entryPublicationHadNoEffect,parseEntrySelection,prepareEntryIntent,prepareInterruptedEntry,readEntryTransition,readFrozenEntrySelection,readInterruptedEntry,requireEntryRestorationScope} from "./WorkspaceEntrySelection";
function original(){return {version:1,entry_protocol:1,workspace:"a".repeat(32),namespace:"/workspace",entry:{version:1,identity:"b".repeat(32),generation:7,path:"project.md"},guard:"c".repeat(64),source_guard:"opaque physical observation",source_hash:"sha256:"+"d".repeat(64),movable:true,source_effects:false};}
describe("workspace entry reading selection",()=>{
 it("retains an immutable exact original rather than a caller-owned response",()=>{const response=original(),proof=parseEntrySelection(response);response.entry.path="replacement.md";response.namespace="/other";expect(proof.entry.path).toBe("project.md");expect(proof.namespace).toBe("/workspace");expect(Object.isFrozen(proof)).toBe(true);expect(Object.isFrozen(proof.entry)).toBe(true);});
 it("rejects coerced booleans, incomplete source proofs and invalid generations",()=>{const proof=original();for(const change of [{unexpected:true},{source_effects:"false"},{source_effects:0},{movable:1},{workspace:"A".repeat(32)},{guard:"C".repeat(64)},{source_guard:null},{source_hash:"d".repeat(64)},{entry:{...proof.entry,generation:0x100000000}},{entry:{...proof.entry,generation:-1}},{entry:{...proof.entry,generation:1.5}},{entry:{...proof.entry,path:"../project.md"}},{entry:{...proof.entry,path:undefined}}])expect(()=>parseEntrySelection({...proof,...change})).toThrow();});
 it("distinguishes a positive no-main reading from a missing source proof",()=>{const proof=original();expect(()=>parseEntrySelection({...proof,entry:{...proof.entry,path:null}})).toThrow();const withoutSource:Record<string,unknown>={...proof};delete withoutSource.source_guard;delete withoutSource.source_hash;expect(parseEntrySelection({...withoutSource,entry:{...proof.entry,path:null},movable:false}).entry.path).toBeNull();});
 it("rejects stale asynchronous acquisition and keeps the original lease afterward",async()=>{const proof=original();let live=false,reads=0;const lease={workspace:proof.workspace,namespace:proof.namespace,current:()=>live};await expect(readFrozenEntrySelection(async()=>{reads++;return proof},lease)).rejects.toThrow();expect(reads).toBe(0);live=true;await expect(readFrozenEntrySelection(async()=>{live=false;return proof},lease)).rejects.toThrow();live=true;await expect(readFrozenEntrySelection(async()=>({...proof,namespace:"/other"}),lease)).rejects.toThrow();const frozen=await readFrozenEntrySelection(async()=>proof,lease);expect(frozen.entryPath).toBe("project.md");live=false;expect(frozen.current()).toBe(false);});
 it('retains the original intent and rejects pending or foreign publication receipts',async()=>{
  const proof=original(),path='docs/literal #.spec',token='e'.repeat(64),request='a'.repeat(32);
  const selection=await readFrozenEntrySelection(async()=>proof,{workspace:proof.workspace,namespace:proof.namespace,current:()=>true});
  const after={...proof.entry,generation:8,path},edit={documentChanges:[{kind:'rename'}]};
  const reply={version:1,workspace:proof.workspace,namespace:proof.namespace,source_files_modified:false,kind:'move',token,context:token,title:'Move main construction',before_entry:proof.entry,after_entry:after,edit};
  const intent=await prepareEntryIntent(selection,{kind:'move',destination:path},async()=>reply,path,()=>true);
  reply.edit.documentChanges.length=0;
  expect(intent.edit).toEqual({documentChanges:[{kind:'rename'}]});
  const confirmed={version:1,state:'confirmed',request_id:request,before_entry:proof.entry,after_entry:after,current_entry:after};
  confirmEntryOutcome(confirmed,request,intent);
  for(const receipt of [undefined,{...confirmed,state:'pending'},{...confirmed,current_entry:proof.entry},{...confirmed,request_id:'b'.repeat(32)}])expect(()=>confirmEntryOutcome(receipt,request,intent)).toThrow();
  const noEffect={...confirmed,state:'no_effect',current_entry:proof.entry,reservation_consumed:false};
  expect(entryPublicationHadNoEffect(noEffect,request,intent)).toBe(true);
  const consumed={...noEffect,reservation_consumed:true,current_entry:{...proof.entry,generation:8}};
  expect(entryPublicationHadNoEffect(consumed,request,intent)).toBe(true);
  for(const wrong of [{...consumed,current_entry:after},{...consumed,reservation_consumed:1},{...noEffect,request_id:'b'.repeat(32)}]){
   expect(()=>entryPublicationHadNoEffect(wrong,request,intent)).toThrow();
  }
  expect(entryPublicationHadNoEffect({...confirmed,state:'pending'},request,intent)).toBe(false);
  await expect(prepareEntryIntent(selection,{kind:'move',destination:path},async()=>({...reply,after_entry:{...after,generation:proof.entry.generation}}),path,()=>true)).rejects.toThrow();
 });
 it('keeps retained history evidence separate from current entry permission',async()=>{
  const proof=original(),id='a'.repeat(32),after={...proof.entry,generation:8,path:'renamed.spec'};
  expect(readEntryTransition(null,id)).toBeNull();
  expect(()=>readEntryTransition({version:1,state:'unavailable',notice:'The original role proof was lost.'},id)).toThrow('role proof');
  const record={version:1,state:'retained',request_id:id,manifest_sha256:'b'.repeat(64),kind:'move',before_entry:proof.entry,after_entry:after};
  const transition=readEntryTransition(record,id)!;
  const selection=await readFrozenEntrySelection(async()=>proof,{workspace:proof.workspace,namespace:proof.namespace,current:()=>true});
  expect(()=>requireEntryRestorationScope(selection,transition.after)).toThrow('current construction');
  const returned=await readFrozenEntrySelection(async()=>({...proof,entry:{...after,generation:10}}),{workspace:proof.workspace,namespace:proof.namespace,current:()=>true});
  expect(()=>requireEntryRestorationScope(returned,transition.after)).not.toThrow();
  expect(()=>readEntryTransition({...record,kind:['move']},id)).toThrow();
 });
 it('requires the exact interrupted evidence and reserves above every attempted generation',async()=>{
  const proof=original(),scope={workspace:proof.workspace,namespace:proof.namespace,version:1,source_files_modified:false};
  const request='2'.repeat(32),manifest='3'.repeat(64),fingerprint='4'.repeat(64),token='5'.repeat(64);
  const before=proof.entry,destination={...before,path:'moved.spec',generation:8};
  const material={...scope,current_entry:before,request_id:request,manifest_sha256:manifest,reservation_floor:9,
   change:{version:1,kind:'move',before,after:destination},can_complete:true,notice:null,
   paths:[{path:before.path,saved_kind:'missing'},{path:destination.path,saved_kind:'file'}]};
  const wire={...scope,interrupted:{material,fingerprint}};
  const evidence=readInterruptedEntry(wire,scope.workspace,scope.namespace)!;
  expect(evidence.missingOriginal).toBe(true);
  expect(readInterruptedEntry({...scope,interrupted:null},scope.workspace,scope.namespace)).toBeNull();
  expect(()=>readInterruptedEntry({...wire,namespace:'/other'},scope.workspace,scope.namespace)).toThrow();
  const after={...destination,generation:10};
  const reply={...scope,kind:'interrupted_completion',interrupted_reading_fingerprint:fingerprint,supersedes:null,
   title:'Complete Move',token,context:token,before_entry:before,after_entry:after,edit:{documentChanges:[]}};
  const load=(body:unknown)=>{expect(body).toEqual({version:1,reading:wire.interrupted,retained_capture:{drafts:{}}});return Promise.resolve(reply);};
  const intent=await prepareInterruptedEntry(evidence,{drafts:{}},load,()=>true);
  await expect(prepareInterruptedEntry(evidence,{},async()=>({...reply,after_entry:destination}),()=>true)).rejects.toThrow();
  await expect(prepareInterruptedEntry(evidence,{},async()=>({...reply,interrupted_reading_fingerprint:'0'.repeat(64)}),()=>true)).rejects.toThrow();
  let current=true;
  await expect(prepareInterruptedEntry(evidence,{},async()=>{current=false;return reply;},()=>current)).rejects.toThrow();
  const receipt={version:2,kind:'interrupted_completion',request_id:'6'.repeat(32),before_entry:before,after_entry:after,
   reservation_floor_before:9,reservation_floor:10,reservation_consumed:true};
  confirmEntryOutcome({...receipt,state:'confirmed',current_entry:after,remaining_interrupted:null},receipt.request_id,intent);
  const noEffect={...receipt,state:'no_effect',current_entry:before,remaining_interrupted:{request_id:request,manifest_sha256:manifest}};
  expect(entryPublicationHadNoEffect(noEffect,receipt.request_id,intent)).toBe(true);
  expect(entryPublicationHadNoEffect({...noEffect,reservation_consumed:false,reservation_floor:9},receipt.request_id,intent)).toBe(true);
  expect(()=>entryPublicationHadNoEffect({...noEffect,remaining_interrupted:null},receipt.request_id,intent)).toThrow();
  expect(()=>entryPublicationHadNoEffect({...noEffect,current_entry:after},receipt.request_id,intent)).toThrow();
 });
});
</ui:copy>

## Apple: isolate immutable proof material from mutable JSON

Retain the exact original scope. Reject unsupported fields, literal-type mismatches, invalid generations and incomplete configured-source observations. Returning JSON must not expose mutable storage that can retarget the retained selection; a late response cannot borrow a newer source lease.

<ui:copy id="workspace-entry-reading-apple-helper">
import Foundation
struct WorkspaceEntryStamp: Decodable, Equatable {
 let version: UInt32
 let identity: String
 let generation: UInt32
 let path: String?
 enum CodingKeys:String,CodingKey {case version,identity,generation,path}
 init(from decoder:Decoder)throws {
  let values=try decoder.container(keyedBy:CodingKeys.self)
  version=try values.decode(UInt32.self,forKey:.version)
  identity=try values.decode(String.self,forKey:.identity)
  generation=try values.decode(UInt32.self,forKey:.generation)
  path=try values.decode(String?.self,forKey:.path)
  guard version==1,workspaceEntryHex(identity,32) else{throw EntrySelectionFailure("Workspace entry stamp is malformed.")}
 }
 static func ==(left:Self,right:Self)->Bool {
  left.version==right.version&&left.identity==right.identity&&left.generation==right.generation&&exactUTF8(left.path,right.path)
 }
 func permitsRestorationReview(of historical:Self)->Bool {
  identity==historical.identity && exactUTF8(path,historical.path) && generation>=historical.generation
 }
}
private func workspaceEntryHex(_ text:String,_ count:Int)->Bool {text.utf8.count==count&&text.utf8.allSatisfy{(48...57).contains($0)||(97...102).contains($0)}}
struct WorkspaceEntrySelectionProof {
 private struct Wire: Decodable {
  let version: UInt32
  let entry_protocol: UInt32
  let workspace: String
  let namespace: String
  let entry: WorkspaceEntryStamp
  let guardValue: String
  let source_guard: String?
  let source_hash: String?
  let movable: Bool
  let source_effects: Bool
  enum CodingKeys:String,CodingKey {case version,entry_protocol,workspace,namespace,entry,source_guard,source_hash,movable,source_effects;case guardValue="guard"}
 }
 private let material: Data
 let workspace: String
 let namespace: String
 let entry: WorkspaceEntryStamp
 let movable: Bool
 init(_ value:Any)throws {
  guard let object=value as? [String:Any],JSONSerialization.isValidJSONObject(object) else {throw EntrySelectionFailure("Workspace entry response is malformed.")}
  guard let entryBody=object["entry"] as? [String:Any],entryBody.keys.contains("path"),object.keys.allSatisfy({["version","entry_protocol","workspace","namespace","entry","guard","source_guard","source_hash","movable","source_effects"].contains($0)}),entryBody.keys.allSatisfy({["version","identity","generation","path"].contains($0)}) else {throw EntrySelectionFailure("Workspace entry path field is missing.")}
  let data=try JSONSerialization.data(withJSONObject:object,options:[.sortedKeys]),wire=try JSONDecoder().decode(Wire.self,from:data)
  let hex=workspaceEntryHex
  guard wire.version==1,wire.entry_protocol==1,hex(wire.workspace,32),!wire.namespace.isEmpty,hex(wire.guardValue,64),!wire.source_effects,wire.entry.version==1,hex(wire.entry.identity,32) else {throw EntrySelectionFailure("Workspace entry response has no supported exact authority.")}
  if let path=wire.entry.path {
   guard !path.isEmpty,!path.contains(":"),!path.contains("\\"),!path.unicodeScalars.contains(where:{$0.value<32||$0.value==127}),!path.components(separatedBy:"/").contains(where:{$0.isEmpty||$0=="."||$0==".."}),let physical=wire.source_guard,!physical.isEmpty,physical.utf8.count<=16384,let hash=wire.source_hash,hash.hasPrefix("sha256:"),hex(String(hash.dropFirst(7)),64) else {throw EntrySelectionFailure("Workspace entry lacks a confined path and original physical source proof.")}
  } else if object.keys.contains("source_guard") || object.keys.contains("source_hash") || wire.movable {throw EntrySelectionFailure("An absent entry cannot carry source or move authority.")}
  material=data;workspace=wire.workspace;namespace=wire.namespace;entry=wire.entry;movable=wire.movable
 }
 func body()throws->Any {try JSONSerialization.jsonObject(with:material)}
}
struct EntrySelectionFailure: LocalizedError {let message:String;init(_ message:String){self.message=message};var errorDescription:String?{message}}
@MainActor struct FrozenWorkspaceEntrySelection {
 let entryPath: String?
 let proof: WorkspaceEntrySelectionProof
 let current: ()->Bool
}
@MainActor enum WorkspaceEntrySelections {
 static func read(load:@MainActor () async throws->Any,workspace:String,namespace:String,current:@escaping @MainActor ()->Bool) async throws->FrozenWorkspaceEntrySelection {
  guard current() else {throw EntrySelectionFailure("Original source context changed before entry acquisition.")}
  let proof=try WorkspaceEntrySelectionProof(await load())
  guard current(),proof.workspace==workspace,proof.namespace==namespace else {throw EntrySelectionFailure("Workspace entry response belongs to an obsolete source context.")}
  return FrozenWorkspaceEntrySelection(entryPath:proof.entry.path,proof:proof,current:current)
 }
}

struct WorkspaceEntryIntent:Decodable {
 let version:Int
 let workspace:String
 let namespace:String
 let token:String
 let context:String
 let kind:String
 let title:String
 let before_entry:WorkspaceEntryStamp
 let after_entry:WorkspaceEntryStamp
 let edit:ResourceJSON
 let source_files_modified:Bool
 let interrupted_reading_fingerprint:String?
 let supersedes:WorkspaceEntryAttempt?
 var interrupted:InterruptedWorkspaceEntry?=nil

 enum CodingKeys:String,CodingKey {
  case version,workspace,namespace,token,context,kind,title,before_entry,after_entry,edit,source_files_modified
  case interrupted_reading_fingerprint,supersedes
 }

 func admit(_ proof:WorkspaceEntrySelectionProof,kind expected:String,path:String?)throws {
  guard version==1,workspace==proof.workspace,exactUTF8(namespace,proof.namespace),kind==expected,!source_files_modified,
   workspaceEntryHex(token,64),context==token,before_entry==proof.entry,after_entry.identity==before_entry.identity,
   UInt64(after_entry.generation)==UInt64(before_entry.generation)+1,exactUTF8(after_entry.path,path),!title.isEmpty else {
   throw EntrySelectionFailure("The entry review did not retain the requested construction and address.")
  }
 }

 mutating func admit(_ reading:InterruptedWorkspaceEntry,superseding attempt:WorkspaceEntryAttempt?)throws {
  let facts=reading.material
  guard facts.can_complete,version==1,kind=="interrupted_completion",workspace==facts.workspace,
   exactUTF8(namespace,facts.namespace),!source_files_modified,workspaceEntryHex(token,64),context==token,
   interrupted_reading_fingerprint==reading.fingerprint,supersedes==attempt,before_entry==facts.current_entry,
   after_entry.identity==before_entry.identity,UInt64(after_entry.generation)==UInt64(facts.reservation_floor)+1,
   exactUTF8(after_entry.path,facts.change.after.path),!title.isEmpty else {
   throw EntrySelectionFailure("The interrupted Move review did not retain the original evidence and intended destination.")
  }
  interrupted=reading
 }
}

struct WorkspaceEntryAttempt:Decodable,Equatable {
 let request_id:String
 let context:String
 let before_entry:WorkspaceEntryStamp
 let after_entry:WorkspaceEntryStamp
 let reviewed_fingerprint:String?

 var wellFormed:Bool {
  workspaceEntryHex(request_id,32) && workspaceEntryHex(context,64) &&
  before_entry.identity==after_entry.identity && after_entry.generation>before_entry.generation &&
  (reviewed_fingerprint.map{workspaceEntryHex($0,64)} ?? true)
 }
}

struct InterruptedWorkspaceEntry {
 struct Change:Decodable {
  let version:Int
  let kind:String
  let before:WorkspaceEntryStamp
  let after:WorkspaceEntryStamp
 }
 struct PathState:Decodable {
  let path:String
  let saved_kind:String
 }
 struct Material:Decodable {
  let version:Int
  let workspace:String
  let namespace:String
  let current_entry:WorkspaceEntryStamp
  let request_id:String
  let manifest_sha256:String
  let change:Change
  let reservation_floor:UInt32
  let paths:[PathState]
  let attempts:[WorkspaceEntryAttempt]
  let can_complete:Bool
  let notice:String?
  let source_files_modified:Bool
 }
 private struct Wire:Decodable {let material:Material;let fingerprint:String}
 private struct Response:Decodable {
  let version:Int
  let workspace:String
  let namespace:String
  let interrupted:ResourceJSON?
  let source_files_modified:Bool
 }
 let body:ResourceJSON
 let material:Material
 let fingerprint:String
 let source:String
 let destination:String
 let missingOriginal:Bool

 static func read(_ value:ResourceJSON,workspace:String,namespace:String)throws->Self? {
  let response=try JSONDecoder().decode(Response.self,from:JSONEncoder().encode(value))
  guard response.version==1,response.workspace==workspace,exactUTF8(response.namespace,namespace),
   workspaceEntryHex(workspace,32),!namespace.isEmpty,!response.source_files_modified else {
   throw EntrySelectionFailure("Interrupted construction evidence belongs to a different workspace.")
  }
  guard let body=response.interrupted else{return nil}
  let wire=try JSONDecoder().decode(Wire.self,from:JSONEncoder().encode(body)),facts=wire.material
  if facts.change.kind != "move",!facts.can_complete,let notice=facts.notice,!notice.isEmpty {throw EntrySelectionFailure(notice)}
  let before=facts.change.before,after=facts.change.after
  guard facts.version==1,facts.workspace==workspace,exactUTF8(facts.namespace,namespace),!facts.source_files_modified,
   workspaceEntryHex(wire.fingerprint,64),workspaceEntryHex(facts.request_id,32),workspaceEntryHex(facts.manifest_sha256,64),
   facts.change.version==1,facts.change.kind=="move",before==facts.current_entry,before.identity==after.identity,
   UInt64(after.generation)==UInt64(before.generation)+1,facts.reservation_floor>=after.generation,
   let source=before.path,let destination=after.path,!source.isEmpty,!destination.isEmpty,!exactUTF8(source,destination),
   facts.paths.count==2,let original=facts.paths.first(where:{exactUTF8($0.path,source)}),
   facts.paths.contains(where:{exactUTF8($0.path,destination)}),
   facts.paths.allSatisfy({$0.saved_kind=="file" || $0.saved_kind=="missing" || ($0.saved_kind=="unknown" && !facts.can_complete)}),
   facts.attempts.count<=256,facts.attempts.allSatisfy(\.wellFormed),facts.notice.map({!$0.isEmpty}) ?? facts.can_complete else {
   throw EntrySelectionFailure("Interrupted construction evidence does not describe one exact Move.")
  }
  return Self(body:body,material:facts,fingerprint:wire.fingerprint,source:source,destination:destination,
   missingOriginal:original.saved_kind=="missing")
 }
}

struct WorkspaceEntryRetainedTransition:Decodable {
 let version:Int
 let state:String
 let request_id:String?
 let manifest_sha256:String?
 let kind:String?
 let before_entry:WorkspaceEntryStamp?
 let after_entry:WorkspaceEntryStamp?
 let notice:String?

 static func read(_ value:ResourceJSON?,record:String)throws->(before:WorkspaceEntryStamp,after:WorkspaceEntryStamp)? {
  guard let value else{return nil}
  if case .null=value{return nil}
  let transition=try JSONDecoder().decode(Self.self,from:JSONEncoder().encode(value))
  if transition.version==1,transition.state=="unavailable",let notice=transition.notice,!notice.isEmpty{throw EntrySelectionFailure(notice)}
  guard transition.version==1,transition.state=="retained",transition.request_id==record,
   let manifest=transition.manifest_sha256,workspaceEntryHex(manifest,64),let kind=transition.kind,["move","restoration","create","repair"].contains(kind),
   let before=transition.before_entry,let after=transition.after_entry,before.identity==after.identity,
   UInt64(after.generation)==UInt64(before.generation)+1 else {throw EntrySelectionFailure("This recovery record lacks a supported entry transition.")}
  return (before,after)
 }
}

struct WorkspaceEntryPublication:Decodable {
 struct Interruption:Decodable {
  let request_id:String
  let manifest_sha256:String
 }
 let version:Int
 let state:String
 let request_id:String
 let before_entry:WorkspaceEntryStamp
 let after_entry:WorkspaceEntryStamp
 let current_entry:WorkspaceEntryStamp
 var reservation_consumed:Bool?=nil
 var kind:String?=nil
 var reservation_floor_before:UInt32?=nil
 var reservation_floor:UInt32?=nil
 var remaining_interrupted:Interruption?=nil

 static func hadNoEffect(_ outcome: Self?, request: String, intent: WorkspaceEntryIntent?) throws -> Bool {
  guard let outcome, outcome.state == "no_effect" else { return false }
  guard let intent,let consumed=outcome.reservation_consumed,outcome.request_id==request,
        outcome.before_entry==intent.before_entry,outcome.after_entry==intent.after_entry else {
   throw EntrySelectionFailure("The no-effect receipt does not match the original entry review.")
  }
  if let evidence=intent.interrupted {
   guard outcome.version==2,outcome.kind=="interrupted_completion",outcome.current_entry==intent.before_entry,
    outcome.reservation_floor_before==evidence.material.reservation_floor,
    let floor=outcome.reservation_floor,UInt64(floor)==UInt64(evidence.material.reservation_floor)+(consumed ? 1:0),
    outcome.remaining_interrupted?.request_id==evidence.material.request_id,
    outcome.remaining_interrupted?.manifest_sha256==evidence.material.manifest_sha256 else {
    throw EntrySelectionFailure("The prior interrupted Move remains unconfirmed.")
   }
   return true
  }
  guard outcome.version==1,outcome.current_entry.identity==intent.before_entry.identity,
        exactUTF8(outcome.current_entry.path, intent.before_entry.path),
        UInt64(outcome.current_entry.generation) == UInt64(intent.before_entry.generation) + (consumed ? 1 : 0) else {
   throw EntrySelectionFailure("The no-effect receipt does not match the original entry review.")
  }
  return true
 }

 static func confirm(_ outcome:Self?,request:String,intent:WorkspaceEntryIntent?)throws {
  guard let intent else{
   if outcome != nil{throw EntrySelectionFailure("Unexpected main construction publication.")}
   return
  }
  guard let outcome,outcome.version==(intent.interrupted == nil ? 1:2),outcome.state=="confirmed",outcome.request_id==request,
   outcome.before_entry==intent.before_entry,outcome.after_entry==intent.after_entry,outcome.current_entry==intent.after_entry else {
   throw EntrySelectionFailure("The original main construction publication is unconfirmed.")
  }
  if let evidence=intent.interrupted {
   guard outcome.kind=="interrupted_completion",outcome.reservation_consumed==true,
    outcome.reservation_floor_before==evidence.material.reservation_floor,
    outcome.reservation_floor==intent.after_entry.generation,outcome.remaining_interrupted==nil else {
    throw EntrySelectionFailure("The receipt does not confirm completion of the reviewed interruption.")
   }
  }
 }
}
</ui:copy>

## Apple: keep the original source lease across suspension

The witness checks mutation isolation, strict wire types, positive no-main versus incomplete source authority, and obsolete source context before or during the optional request. It exercises models, not physical input or rendered accessibility.

<ui:copy id="workspace-entry-reading-apple-witness">
import XCTest
@testable import HickorySimple
final class WorkspaceEntrySelectionTests:XCTestCase {
 private func original()->[String:Any]{["version":1,"entry_protocol":1,"workspace":String(repeating:"a",count:32),"namespace":"/workspace","entry":["version":1,"identity":String(repeating:"b",count:32),"generation":7,"path":"project.md"],"guard":String(repeating:"c",count:64),"source_guard":"opaque physical observation","source_hash":"sha256:"+String(repeating:"d",count:64),"movable":true,"source_effects":false]}
 func testOriginalProofDoesNotFollowMutableResponseOrReturnedBody()throws {var response=original();let proof=try WorkspaceEntrySelectionProof(response);response["namespace"]="/other";var body=try XCTUnwrap(proof.body() as? [String:Any]);body["namespace"]="/mutated";XCTAssertEqual(proof.namespace,"/workspace");XCTAssertEqual(proof.entry.path,"project.md");XCTAssertEqual((try proof.body() as? [String:Any])?["namespace"] as? String,"/workspace")}
 func testStrictTypesAndGenerationsRefuseIncompleteAuthority(){for (key,value) in [("unexpected",1 as Any),("source_effects","false" as Any),("source_effects",0 as Any),("movable",1 as Any),("source_guard",NSNull()),("source_hash","wrong" as Any),("guard",String(repeating:"C",count:64) as Any)]{var response=original();response[key]=value;XCTAssertThrowsError(try WorkspaceEntrySelectionProof(response))};for generation in [-1,4_294_967_296]{var response=original(),entry=response["entry"] as! [String:Any];entry["generation"]=generation;response["entry"]=entry;XCTAssertThrowsError(try WorkspaceEntrySelectionProof(response))}}
 func testPositiveAbsenceCannotBorrowSourceOrMoveAuthority()throws {var response=original(),entry=response["entry"] as! [String:Any];entry["path"]=NSNull();response["entry"]=entry;XCTAssertThrowsError(try WorkspaceEntrySelectionProof(response));response.removeValue(forKey:"source_guard");response.removeValue(forKey:"source_hash");response["movable"]=false;XCTAssertNil(try WorkspaceEntrySelectionProof(response).entry.path)}
 func testNoEffectRequiresTheExactRequestAndOriginalAddress() throws {
  let response=original(),before=try XCTUnwrap(response["entry"] as? [String:Any]),request=String(repeating:"a",count:32)
  let after=before.merging(["generation":8,"path":"moved.spec"]){_,new in new}
  let token=String(repeating:"e",count:64)
  let intentBody:[String:Any]=["version":1,"workspace":response["workspace"]!,"namespace":"/workspace","context":token,
    "token":token,"kind":"move","title":"Move","edit":[:],"before_entry":before,"after_entry":after,"source_files_modified":false]
  let intent=try JSONDecoder().decode(WorkspaceEntryIntent.self,from:JSONSerialization.data(withJSONObject:intentBody))
  func outcome(consumed:Any,current:[String:Any])throws->WorkspaceEntryPublication {
   let body:[String:Any]=["version":1,"state":"no_effect","request_id":request,"before_entry":before,"after_entry":after,
      "current_entry":current,"reservation_consumed":consumed]
   return try JSONDecoder().decode(WorkspaceEntryPublication.self,from:JSONSerialization.data(withJSONObject:body))
  }
  XCTAssertTrue(try WorkspaceEntryPublication.hadNoEffect(outcome(consumed:false,current:before),request:request,intent:intent))
  let retained=before.merging(["generation":8]){_,new in new}
  XCTAssertTrue(try WorkspaceEntryPublication.hadNoEffect(outcome(consumed:true,current:retained),request:request,intent:intent))
  XCTAssertThrowsError(try WorkspaceEntryPublication.hadNoEffect(outcome(consumed:true,current:after),request:request,intent:intent))
  XCTAssertThrowsError(try WorkspaceEntryPublication.hadNoEffect(outcome(consumed:1,current:retained),request:request,intent:intent))
 }
 @MainActor func testAsynchronousAcquisitionNeverRebindsOriginalLease()async throws {let response=original();let workspace=response["workspace"] as! String;var live=false,reads=0;do{_ = try await WorkspaceEntrySelections.read(load:{reads+=1;return response},workspace:workspace,namespace:"/workspace",current:{live});XCTFail("stale source must refuse before request")}catch{};XCTAssertEqual(reads,0);live=true;do{_ = try await WorkspaceEntrySelections.read(load:{live=false;return response},workspace:workspace,namespace:"/workspace",current:{live});XCTFail("late reply must refuse")}catch{};live=true;let frozen=try await WorkspaceEntrySelections.read(load:{response},workspace:workspace,namespace:"/workspace",current:{live});XCTAssertEqual(frozen.entryPath,"project.md");live=false;XCTAssertFalse(frozen.current())}
}
</ui:copy>

## Android: check literal wire types before retaining proof

Retain the exact original scope. Reject unsupported fields, literal-type mismatches, invalid generations and incomplete configured-source observations. Returning JSON must not expose mutable storage that can retarget the retained selection; a late response cannot borrow a newer source lease.

<ui:copy id="workspace-entry-reading-android-helper">
package com.hickorydocs.simple
import org.json.JSONObject

data class WorkspaceEntryStamp(val version:Long,val identity:String,val generation:Long,val path:String?) {
 fun permitsRestorationReview(historical:WorkspaceEntryStamp)=
  identity==historical.identity&&path==historical.path&&generation>=historical.generation
}
private fun entryText(value:JSONObject,key:String)=value.opt(key) as? String ?: error("Workspace entry $key must be an exact string.")
private fun entryInteger(value:JSONObject,key:String):Long {
 val raw=value.opt(key)
 require(raw is Int||raw is Long){"Workspace entry $key must be an exact integer."}
 return (raw as Number).toLong()
}
private fun entryHex(value:String,count:Int)=value.length==count&&value.all{it in '0'..'9'||it in 'a'..'f'}
private fun entryStamp(source:JSONObject):WorkspaceEntryStamp {
 require(source.keys().asSequence().all{it in setOf("version","identity","generation","path")})
 val generation=entryInteger(source,"generation");val identity=entryText(source,"identity")
 require(entryInteger(source,"version")==1L&&entryHex(identity,32)&&generation in 0L..0xffffffffL&&source.has("path"))
 val rawPath=source.get("path")
 val path=if(rawPath===JSONObject.NULL)null else rawPath as? String ?: error("Workspace entry path is malformed.")
 if(path!=null)require(path.isNotEmpty()&&!path.contains(':')&&!path.contains('\\')&&path.none{it.code<32||it.code==127}&&path.split('/').none{it.isEmpty()||it=="."||it==".."})
 return WorkspaceEntryStamp(1,identity,generation,path)
}
class WorkspaceEntrySelectionProof(value:JSONObject) {
 private val material=value.toString()
 val workspace:String
 val namespace:String
 val entry:WorkspaceEntryStamp
 val movable:Boolean
 init {
  val body=JSONObject(material)
  require(body.keys().asSequence().all{it in setOf("version","entry_protocol","workspace","namespace","entry","guard","source_guard","source_hash","movable","source_effects")})
  require(entryInteger(body,"version")==1L&&entryInteger(body,"entry_protocol")==1L)
  workspace=entryText(body,"workspace");namespace=entryText(body,"namespace")
  require(entryHex(workspace,32)&&namespace.isNotEmpty()&&entryHex(entryText(body,"guard"),64))
  require(body.opt("source_effects") is Boolean&&body.get("source_effects")==false)
  movable=body.opt("movable") as? Boolean ?: error("Workspace entry move capability is malformed.")
  entry=entryStamp(body.getJSONObject("entry"))
  if(entry.path==null){require(!body.has("source_guard")&&!body.has("source_hash")&&!movable)}
  else {
   val guard=entryText(body,"source_guard");val hash=entryText(body,"source_hash")
   require(guard.isNotEmpty()&&guard.toByteArray(Charsets.UTF_8).size<=16384&&hash.startsWith("sha256:")&&entryHex(hash.removePrefix("sha256:"),64))
  }
 }
 fun body()=JSONObject(material)
}
data class FrozenWorkspaceEntrySelection(val entryPath:String?,val proof:WorkspaceEntrySelectionProof,val current:()->Boolean)
object WorkspaceEntrySelections {
 suspend fun read(load:suspend()->JSONObject,workspace:String,namespace:String,current:()->Boolean):FrozenWorkspaceEntrySelection {
  check(current()){"Original source context changed before entry acquisition."}
  val proof=WorkspaceEntrySelectionProof(load())
  check(current()&&proof.workspace==workspace&&proof.namespace==namespace){"Workspace entry response belongs to an obsolete source context."}
  return FrozenWorkspaceEntrySelection(proof.entry.path,proof,current)
 }
}

class WorkspaceEntryIntent private constructor(value:JSONObject) {
 val token=entryText(value,"token")
 val context=entryText(value,"context")
 val title=entryText(value,"title")
 val before=entryStamp(value.getJSONObject("before_entry"))
 val after=entryStamp(value.getJSONObject("after_entry"))
 private val material=value.getJSONObject("edit").toString()
 var interrupted:InterruptedWorkspaceEntry?=null
  private set

 constructor(value:JSONObject,proof:WorkspaceEntrySelectionProof,kind:String,path:String?):this(value) {
  check(entryInteger(value,"version")==1L&&value.opt("workspace")==proof.workspace&&value.opt("namespace")==proof.namespace&&
   value.opt("kind")==kind&&value.opt("source_files_modified")==false&&entryHex(token,64)&&context==token&&title.isNotEmpty()&&
   before==proof.entry&&after.identity==before.identity&&after.generation==before.generation+1&&after.path==path){
   "The entry review did not retain the requested construction and address."
  }
 }
 fun edit()=JSONObject(material)

 companion object {
  fun forCompletion(value:JSONObject,reading:InterruptedWorkspaceEntry,superseding:WorkspaceEntryAttempt?):WorkspaceEntryIntent {
   val intent=WorkspaceEntryIntent(value)
   val supersedes=value.get("supersedes").let { raw ->
    if(raw===JSONObject.NULL)null else WorkspaceEntryAttempt.read(raw as? JSONObject ?: error("Invalid interrupted publication attempt."))
   }
   check(reading.canComplete&&entryInteger(value,"version")==1L&&value.opt("kind")=="interrupted_completion"&&
    value.opt("workspace")==reading.workspace&&value.opt("namespace")==reading.namespace&&value.opt("source_files_modified")==false&&
    value.opt("interrupted_reading_fingerprint")==reading.fingerprint&&supersedes==superseding&&
    entryHex(intent.token,64)&&intent.context==intent.token&&intent.title.isNotEmpty()&&intent.before==reading.currentEntry&&
    intent.after.identity==intent.before.identity&&intent.after.generation==reading.reservationFloor+1&&intent.after.path==reading.destination) {
    "The interrupted Move review did not retain the original evidence and intended destination."
   }
   intent.interrupted=reading
   return intent
  }
 }
}

data class WorkspaceEntryAttempt(val request:String,val context:String,val before:WorkspaceEntryStamp,
 val after:WorkspaceEntryStamp,val fingerprint:String?) {
 companion object {
  fun read(value:JSONObject):WorkspaceEntryAttempt {
   val request=entryText(value,"request_id")
   val context=entryText(value,"context")
   val before=entryStamp(value.getJSONObject("before_entry"))
   val after=entryStamp(value.getJSONObject("after_entry"))
   val fingerprint=value.get("reviewed_fingerprint").let{if(it===JSONObject.NULL)null else it as? String ?: error("Malformed reviewed fingerprint.")}
   check(entryHex(request,32)&&entryHex(context,64)&&before.identity==after.identity&&after.generation>before.generation&&
    (fingerprint==null||entryHex(fingerprint,64))){"Invalid interrupted publication attempt."}
   return WorkspaceEntryAttempt(request,context,before,after,fingerprint)
  }
 }
}

class InterruptedWorkspaceEntry private constructor(val workspace:String,val namespace:String,val fingerprint:String,val request:String,val manifestHash:String,
 val source:String,val destination:String,val currentEntry:WorkspaceEntryStamp,val reservationFloor:Long,
 val canComplete:Boolean,val notice:String?,val missingOriginal:Boolean,val attempts:List<WorkspaceEntryAttempt>,private val encoded:String) {
 fun body()=JSONObject(encoded)

 companion object {
  fun read(response:JSONObject,workspace:String,namespace:String):InterruptedWorkspaceEntry? {
   check(entryInteger(response,"version")==1L&&entryHex(workspace,32)&&namespace.isNotEmpty()&&
    response.opt("workspace")==workspace&&response.opt("namespace")==namespace&&response.opt("source_files_modified")==false) {
    "Interrupted construction evidence belongs to a different workspace."
   }
   if(response.get("interrupted")===JSONObject.NULL)return null
   val reading=response.getJSONObject("interrupted")
   val facts=reading.getJSONObject("material")
   val change=facts.getJSONObject("change")
   val notice=facts.get("notice").let{if(it===JSONObject.NULL)null else it as? String ?: error("Interrupted Move notice is malformed.")}
   val canComplete=facts.get("can_complete") as? Boolean ?: error("Interrupted Move capability is malformed.")
   if(change.opt("kind")!="move"&&!canComplete&&!notice.isNullOrEmpty())error(notice)
   val current=entryStamp(facts.getJSONObject("current_entry"))
   val before=entryStamp(change.getJSONObject("before"))
   val after=entryStamp(change.getJSONObject("after"))
   val floor=entryInteger(facts,"reservation_floor")
   val fingerprint=entryText(reading,"fingerprint")
   val source=before.path ?: error("Interrupted Move has no original source.")
   val destination=after.path ?: error("Interrupted Move has no destination.")
   val paths=facts.getJSONArray("paths")
   val attempts=facts.getJSONArray("attempts")
   val request=entryText(facts,"request_id")
   val manifest=entryText(facts,"manifest_sha256")
   check(entryInteger(facts,"version")==1L&&facts.opt("workspace")==workspace&&facts.opt("namespace")==namespace&&
    facts.opt("source_files_modified")==false&&entryHex(fingerprint,64)&&entryHex(request,32)&&
    entryHex(manifest,64)&&entryInteger(change,"version")==1L&&change.opt("kind")=="move"&&
    current==before&&before.identity==after.identity&&after.generation==before.generation+1&&floor in after.generation..0xffffffffL&&
    source!=destination&&(notice?.isNotEmpty() ?: canComplete)&&paths.length()==2&&attempts.length()<=256){
    "Interrupted construction evidence does not describe one exact Move."
   }
   val states=(0 until paths.length()).associate { index ->
    val item=paths.getJSONObject(index)
    entryText(item,"path") to entryText(item,"saved_kind")
   }
   check(states.keys==setOf(source,destination)&&states.values.all{it=="file"||it=="missing"||(it=="unknown"&&!canComplete)}) {
    "Interrupted construction evidence has invalid source paths."
   }
   val observed=(0 until attempts.length()).map{WorkspaceEntryAttempt.read(attempts.getJSONObject(it))}
   return InterruptedWorkspaceEntry(workspace,namespace,fingerprint,request,manifest,source,destination,current,floor,canComplete,notice,
    states[source]=="missing",observed,reading.toString())
  }
 }
}

data class WorkspaceEntryTransition(val before:WorkspaceEntryStamp,val after:WorkspaceEntryStamp) {
 companion object {
  fun read(raw:Any?,record:String):WorkspaceEntryTransition? {
   if(raw==null||raw===JSONObject.NULL)return null
   val value=raw as? JSONObject ?: error("This recovery record lacks a supported entry transition.")
   if(entryInteger(value,"version")==1L&&value.opt("state")=="unavailable")error(entryText(value,"notice"))
   check(entryInteger(value,"version")==1L&&value.opt("state")=="retained"&&value.opt("request_id")==record&&
    entryHex(entryText(value,"manifest_sha256"),64)&&value.opt("kind") in listOf("move","restoration","create","repair")){
    "This recovery record lacks a supported entry transition."
   }
   val before=entryStamp(value.getJSONObject("before_entry"))
   val after=entryStamp(value.getJSONObject("after_entry"))
   check(before.identity==after.identity&&before.generation+1==after.generation){"The retained entry transition is inconsistent."}
   return WorkspaceEntryTransition(before,after)
  }
 }
}

fun confirmWorkspaceEntryPublication(raw:Any?,request:String,intent:WorkspaceEntryIntent?) {
 if(intent==null){check(raw==null){"Unexpected main construction publication."};return}
 val value=raw as? JSONObject ?: error("The original main construction publication is unconfirmed.")
 val interrupted=intent.interrupted
 check(entryInteger(value,"version")==if(interrupted==null)1L else 2L) {"The publication uses an unsupported entry receipt."}
 check(value.opt("state")=="confirmed"&&value.opt("request_id")==request&&
  entryStamp(value.getJSONObject("before_entry"))==intent.before&&entryStamp(value.getJSONObject("after_entry"))==intent.after&&
  entryStamp(value.getJSONObject("current_entry"))==intent.after){"The original main construction publication is unconfirmed."}
 if(interrupted!=null)check(value.opt("kind")=="interrupted_completion"&&value.opt("reservation_consumed")==true&&
  entryInteger(value,"reservation_floor_before")==interrupted.reservationFloor&&entryInteger(value,"reservation_floor")==intent.after.generation&&
  value.get("remaining_interrupted")===JSONObject.NULL){"The receipt does not confirm completion of the reviewed interruption."}
}

fun workspaceEntryPublicationHadNoEffect(raw: Any?, request: String, intent: WorkspaceEntryIntent?): Boolean {
 val outcome = raw as? JSONObject ?: return false
 if (outcome.opt("state") != "no_effect") return false
 val expected = checkNotNull(intent) { "No original entry review exists for this receipt." }
 val consumed = outcome.opt("reservation_consumed") as? Boolean ?: error("The entry reservation outcome is malformed.")
 val interrupted=expected.interrupted
 val version=if(interrupted==null)1L else 2L
 check(entryInteger(outcome,"version")==version && outcome.opt("request_id")==request &&
       entryStamp(outcome.getJSONObject("before_entry")) == expected.before &&
       entryStamp(outcome.getJSONObject("after_entry")) == expected.after) {
  "The no-effect receipt does not match the original entry review."
 }
 val current=entryStamp(outcome.getJSONObject("current_entry"))
 val advance=if(consumed)1 else 0
 if(interrupted==null){
  check(current==expected.before.copy(generation=expected.before.generation+advance)) {"The original entry namespace is unconfirmed."}
 }else{
  val remaining=outcome.getJSONObject("remaining_interrupted")
  check(outcome.opt("kind")=="interrupted_completion"&&current==expected.before&&
   entryInteger(outcome,"reservation_floor_before")==interrupted.reservationFloor&&
   entryInteger(outcome,"reservation_floor")==interrupted.reservationFloor+advance&&
   remaining.opt("request_id")==interrupted.request&&remaining.opt("manifest_sha256")==interrupted.manifestHash) {
   "The prior interrupted Move remains unconfirmed."
  }
 }
 return true
}
</ui:copy>

## Android: reject coercion and stale acquisition

The witness checks mutation isolation, strict wire types, positive no-main versus incomplete source authority, and obsolete source context before or during the optional request. It exercises models, not physical input or rendered accessibility.

<ui:copy id="workspace-entry-reading-android-witness">
package com.hickorydocs.simple
import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
class WorkspaceEntrySelectionTest {
 private fun original()=JSONObject().put("version",1).put("entry_protocol",1).put("workspace","a".repeat(32)).put("namespace","/workspace").put("entry",JSONObject().put("version",1).put("identity","b".repeat(32)).put("generation",7).put("path","project.md")).put("guard","c".repeat(64)).put("source_guard","opaque physical observation").put("source_hash","sha256:"+"d".repeat(64)).put("movable",true).put("source_effects",false)
 private fun refuses(action:()->Unit){try{action();fail("Malformed entry authority was accepted")}catch(expected:IllegalArgumentException){}catch(expected:IllegalStateException){}}
 @Test fun originalProofDoesNotFollowMutableResponseOrReturnedBody(){val response=original();val proof=WorkspaceEntrySelectionProof(response);response.put("namespace","/other");proof.body().put("namespace","/mutated");assertEquals("/workspace",proof.namespace);assertEquals("project.md",proof.entry.path);assertEquals("/workspace",proof.body().getString("namespace"))}
 @Test fun strictTypesAndGenerationBoundsRefuseIncompleteAuthority(){for((key,value)in listOf("unexpected" to 1,"source_effects" to "false","source_effects" to 0,"movable" to 1,"source_guard" to JSONObject.NULL,"source_hash" to "wrong","guard" to "C".repeat(64))){refuses{WorkspaceEntrySelectionProof(original().put(key,value))}};for(value in listOf(-1L,0x100000000L)){val body=original();body.getJSONObject("entry").put("generation",value);refuses{WorkspaceEntrySelectionProof(body)}}}
 @Test fun positiveAbsenceCannotBorrowSourceOrMoveAuthority(){val response=original();response.getJSONObject("entry").put("path",JSONObject.NULL);refuses{WorkspaceEntrySelectionProof(response)};response.remove("source_guard");response.remove("source_hash");response.put("movable",false);assertNull(WorkspaceEntrySelectionProof(response).entry.path)}
 @Test fun noEffectRequiresTheExactRequestAndOriginalAddress(){
  val response=original();val proof=WorkspaceEntrySelectionProof(response);val before=response.getJSONObject("entry")
  val after=JSONObject(before.toString()).put("generation",8).put("path","moved.spec")
  val request="a".repeat(32);val token="e".repeat(64)
  val reply=JSONObject().put("version",1).put("workspace",proof.workspace).put("namespace",proof.namespace)
   .put("before_entry",before).put("after_entry",after).put("context",token).put("token",token)
   .put("kind","move").put("title","Move").put("edit",JSONObject()).put("source_files_modified",false)
  val intent=WorkspaceEntryIntent(reply,proof,"move","moved.spec")
  fun outcome(consumed:Any,retained:JSONObject)=JSONObject().put("version",1).put("state","no_effect").put("request_id",request)
   .put("before_entry",before).put("after_entry",after).put("reservation_consumed",consumed).put("current_entry",retained)
  val retained=JSONObject(before.toString()).put("generation",8)
  assertTrue(workspaceEntryPublicationHadNoEffect(outcome(false,before),request,intent))
  assertTrue(workspaceEntryPublicationHadNoEffect(outcome(true,retained),request,intent))
  refuses{workspaceEntryPublicationHadNoEffect(outcome(true,after),request,intent)}
  refuses{workspaceEntryPublicationHadNoEffect(outcome(1,retained),request,intent)}
 }
 @Test fun asynchronousAcquisitionNeverRebindsOriginalLease()=runBlocking {val response=original();var live=false;var reads=0;try{WorkspaceEntrySelections.read({reads++;response},"a".repeat(32),"/workspace"){live};fail("stale source must refuse before request")}catch(expected:IllegalStateException){};assertEquals(0,reads);live=true;try{WorkspaceEntrySelections.read({live=false;response},"a".repeat(32),"/workspace"){live};fail("late reply must refuse")}catch(expected:IllegalStateException){};live=true;val frozen=WorkspaceEntrySelections.read({response},"a".repeat(32),"/workspace"){live};assertEquals("project.md",frozen.entryPath);live=false;assertFalse(frozen.current())}
}
</ui:copy>

<ui:file path="apps/web/src/simple/WorkspaceEntrySelection.ts"><ui:paste select="#workspace-entry-reading-browser-helper"/></ui:file>
<ui:file path="apps/web/src/simple/WorkspaceEntrySelection.test.ts"><ui:paste select="#workspace-entry-reading-browser-witness"/></ui:file>
<ui:file path="apps/apple-simple/Sources/WorkspaceEntrySelection.swift"><ui:paste select="#workspace-entry-reading-apple-helper"/></ui:file>
<ui:file path="apps/apple-simple/Tests/WorkspaceEntrySelectionTests.swift"><ui:paste select="#workspace-entry-reading-apple-witness"/></ui:file>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/WorkspaceEntrySelection.kt"><ui:paste select="#workspace-entry-reading-android-helper"/></ui:file>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/WorkspaceEntrySelectionTest.kt"><ui:paste select="#workspace-entry-reading-android-witness"/></ui:file>
</ui:doc>
