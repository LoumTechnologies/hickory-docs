import {beforeAll,expect,it,vi} from 'vitest';
import {ConflictResolutionModel,decodeConflictPreview,conflictDraftable,type ConflictPorts,type ConflictPreview,type ConflictReading,type ConflictOutput} from './GitConflictResolution';
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
function reading():ConflictReading{return {version:1,resolution_protocol:1,members:['a.py'],paths:[{path:'a.py',working:null,stages:[],unavailable:null}],observation:{fingerprint:'index'},source_files_modified:false,index_modified:false};}
function preview(noop=false):ConflictPreview{return {version:1,resolution_protocol:1,token:'a'.repeat(32),fingerprint:'b'.repeat(64),members:['a.py'],remaining_conflicts:[],plan:{version:1,context:'context',title:'Resolve',patches:noop?[]:[{path:'a.py',before:{kind:'file',identity:'a',text:'before'},after:{kind:'file',identity:'a',text:'after'}}]},consequences:{version:1,workspace:'workspace',context:'context',fingerprint:'c'.repeat(64),protected_targets:[],saved:{entries:[],removed_entries:[],output_paths:[]},draft:{entries:[],removed_entries:[],output_paths:[]}},saved_sources:{'a.py':{kind:'file',text:'after',sha256:'d'.repeat(64),readonly:false,executable:false,mode:420}},needs_save:!noop,stage_available:true,stage_unavailable:null,source_files_modified:false,index_modified:false};}
const output:ConflictOutput={path:'a.py',identity_from:'a.py',mode:'100644',content:{kind:'text',text:'after'}};
function fixture(noop=false){const value=preview(noop),calls:{path:string;body:unknown}[]=[],draft=vi.fn(),adopt=vi.fn();let current=true,failStage=false,unknown=false;const ports:ConflictPorts={lease:()=>()=>current,capture:async()=>({inputs:{workspace:'workspace'},capture:{},current:()=>current}),draft,adopt,load:async<T>(_method:string,path:string,body?:unknown)=>{calls.push({path,body});const input=body as Record<string,unknown>;let result:unknown;if(path.endsWith('/read'))result=reading();else if(path.endsWith('/prepare'))result=value;else if(path.endsWith('/validate'))result={version:1,current:true,stage:input.stage??false,token:value.token,fingerprint:value.fingerprint,plan:value.plan,saved_sources:value.saved_sources,source_files_modified:false,index_modified:false};else if(path.endsWith('/save'))result={version:2,adoption_current:true,request_id:input.request_id};else if(path.endsWith('/stage'))result={version:1,operation:'git_conflict_stage',complete:true,index_modified:true,index_uncertain:false,source_files_modified:false,request_id:failStage?'foreign':input.request_id,token:value.token,review_fingerprint:value.fingerprint,resolved_members:value.members,remaining_conflicts:value.remaining_conflicts,fingerprint:'e'.repeat(64)};else result={version:1,request_id:path.split('request_id=')[1],state:unknown?'unknown':'completed',outcome:null,error:'refused'};return result as T;}};const model=new ConflictResolutionModel(ports);return {model,value,calls,draft,adopt,ports,stale:()=>{current=false;},foreignStage:()=>{failStage=true;},unknown:()=>{unknown=true;}};}
async function prepared(f:ReturnType<typeof fixture>){await f.model.read(['a.py']);f.model.change(['a.py'],[output]);await f.model.prepare();expect(f.model.review).not.toBeNull();}
it('keeps composition, draft application, source Save and explicit Stage separate',async()=>{const f=fixture();await prepared(f);await f.model.act(true);expect(f.calls.some(call=>call.path.endsWith('/stage'))).toBe(false);await f.model.draft();expect(f.draft).toHaveBeenCalledOnce();expect(f.adopt).not.toHaveBeenCalled();expect(f.calls.some(call=>call.path.endsWith('/save'))).toBe(false);await prepared(f);await f.model.act(false);expect(f.adopt).toHaveBeenCalledOnce();expect(f.model.saved).toBe(true);await f.model.act(true);expect(f.model.pending).toBeNull();expect(f.model.notice).toContain('group staged');});
it('holds Main editors before Save and releases them before the separate Stage lease',async()=>{
 const f=fixture(),steps:string[]=[];
 const load=f.ports.load,lease=f.ports.lease;
 f.ports.load=async<T>(method:'GET'|'POST',path:string,body?:unknown)=>{
  if(path.endsWith('/save'))steps.push('save');
  return load<T>(method,path,body);
 };
 f.ports.lease=()=>{steps.push('lease');return lease();};
 f.ports.holdSave=()=>{
  steps.push('hold');
  return {adopt:async()=>{steps.push('adopt');},release:()=>{steps.push('release');}};
 };
 await prepared(f);steps.length=0;
 await f.model.act(false);
 expect(steps).toEqual(['hold','save','adopt','release','lease']);
 expect(f.model.pending).toBeNull();
 expect(f.model.saved).toBe(true);
 expect(f.adopt).not.toHaveBeenCalled();
});
it('releases a failed Save hold and recovers only the original request',async()=>{
 const f=fixture(),release=vi.fn(),adopt=vi.fn();
 const original=f.ports.load;
 f.ports.holdSave=()=>({adopt,release});
 f.ports.load=async<T>(method:'GET'|'POST',path:string,body?:unknown)=>{
  if(path.endsWith('/save'))throw Error('connection ended');
  if(path.includes('/status?')){
   const request_id=path.split('request_id=')[1];
   return {version:1,request_id,state:'completed',outcome:{version:2,request_id}} as T;
  }
  return original<T>(method,path,body);
 };
 await prepared(f);await f.model.act(false);
 const request=f.model.pending?.id;
 expect(request).toBeTruthy();
 expect(release).toHaveBeenCalledOnce();
 expect(adopt).not.toHaveBeenCalled();
 f.model.close();await f.model.status();
 expect(f.model.pending).toBeNull();
 expect(adopt).toHaveBeenCalledWith({version:2,request_id:request},request);
 expect(release).toHaveBeenCalledTimes(2);
});
it('requires a fresh editor lease for Stage after adopting the saved group',async()=>{
 const f=fixture();await prepared(f);
 f.ports.lease=()=>{throw Error('The current editor lease is unavailable.');};
 await f.model.act(false);
 expect(f.adopt).toHaveBeenCalledOnce();
 expect(f.model.pending).toBeNull();
 expect(f.model.review).toBeNull();
 expect(f.model.notice).toContain('Complete saved group confirmed');
 await f.model.act(true);
 expect(f.calls.some(call=>call.path.endsWith('/stage'))).toBe(false);
});
it('stages an already-saved complete group without a no-op Save',async()=>{const f=fixture(true);await prepared(f);expect(conflictDraftable(f.model.review!)).toBe(false);const same=preview();same.plan.patches[0].after.text='before';expect(conflictDraftable({value:same,patches:[{path:'a.py',before:{identity:'a',text:'before'},after:{identity:'a',text:'before'}}],current:()=>true})).toBe(false);await f.model.act(true);expect(f.calls.some(call=>call.path.endsWith('/save'))).toBe(false);expect(f.model.notice).toContain('group staged');});
it('retains uncertain request identity across close and refuses a second mutation',async()=>{const f=fixture(true);await prepared(f);f.foreignStage();await f.model.act(true);const id=f.model.pending?.id;expect(id).toBeTruthy();f.model.close();await f.model.act(true);expect(f.model.pending?.id).toBe(id);expect(f.calls.filter(call=>call.path.endsWith('/stage'))).toHaveLength(1);f.unknown();await f.model.status();expect(f.model.pending?.id).toBe(id);expect(f.adopt).not.toHaveBeenCalled();});
it('retires only a confirmed refused request whose complete original preimage is current',async()=>{for(const stale of [false,true]){const f=fixture(true);await prepared(f);f.foreignStage();await f.model.act(true);if(stale)f.stale();await f.model.status();expect(!!f.model.pending).toBe(stale);if(!stale)expect(f.model.notice).toContain('original complete preimage');}});
it('refuses stale source before dispatch and keeps immutable decoded proposals independent of raw mutation',async()=>{const f=fixture();await prepared(f);const held=f.model.review!;f.value.plan.patches[0].after.text='tampered';expect(held.patches[0].after?.text).toBe('after');f.stale();await f.model.act(false);expect(f.calls.some(call=>call.path.endsWith('/save'))).toBe(false);for(const [key,bad] of [['token','foreign'],['fingerprint','old'],['version',true],['resolution_protocol',1.5],['index_modified',true]] as const){expect(()=>decodeConflictPreview({...preview(),[key]:bad} as ConflictPreview)).toThrow();}});
it('retires a delayed reading on close or lost source/namespace lease',async()=>{for(const closed of [false,true]){let release!:(value:unknown)=>void,current=true;const ports:ConflictPorts={load:<T>()=>new Promise<T>(done=>{release=done as (value:unknown)=>void;}),lease:()=>()=>current,capture:async()=>{throw Error('unused');},draft:()=>{},adopt:()=>{}};const model=new ConflictResolutionModel(ports),pending=model.read(['a.py']);if(closed)model.close();else current=false;release(reading());await pending;expect(model.reading).toBeNull();}});
it('retains composition when leaving a row and starts clean on another explicit group',async()=>{
 const f=fixture();await prepared(f);
 f.model.close();
 expect(f.model.anchor).toBe('a.py');
 expect(f.model.outputs).toEqual([output]);
 expect(f.model.review).toBeNull();
 await f.model.act(false);
 expect(f.adopt).not.toHaveBeenCalled();
 await f.model.read(['b.py'],'b.py');
 expect(f.model.anchor).toBe('b.py');
 expect(f.model.outputs).toEqual([]);
 expect(f.model.reading).toBeNull();
});
it('reports refused source lease acquisition without rejecting the optional reading promise',async()=>{let reads=0;const ports:ConflictPorts={lease:()=>{throw Error('Foreign retained source namespace');},load:async<T>()=>{reads++;return reading() as T;},capture:async()=>{throw Error('unused');},draft:()=>{},adopt:()=>{}};const model=new ConflictResolutionModel(ports);await expect(model.read(['a.py'])).resolves.toBeUndefined();expect(model.reading).toBeNull();expect(model.busy).toBe(false);expect(model.notice).toContain('Foreign retained source');expect(reads).toBe(0);});

