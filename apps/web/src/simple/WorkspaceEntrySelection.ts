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
