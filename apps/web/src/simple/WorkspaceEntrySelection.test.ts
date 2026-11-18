import {describe,it,expect} from "vitest";
import {confirmEntryOutcome,parseEntrySelection,prepareEntryIntent,readEntryTransition,readFrozenEntrySelection,requireEntryTransitionCurrent} from "./WorkspaceEntrySelection";
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
  await expect(prepareEntryIntent(selection,{kind:'move',destination:path},async()=>({...reply,after_entry:{...after,generation:proof.entry.generation}}),path,()=>true)).rejects.toThrow();
 });
 it('keeps retained history evidence separate from current entry permission',async()=>{
  const proof=original(),id='a'.repeat(32),after={...proof.entry,generation:8,path:'renamed.spec'};
  expect(readEntryTransition(null,id)).toBeNull();
  expect(()=>readEntryTransition({version:1,state:'unavailable',notice:'The original role proof was lost.'},id)).toThrow('role proof');
  const record={version:1,state:'retained',request_id:id,manifest_sha256:'b'.repeat(64),kind:'move',before_entry:proof.entry,after_entry:after};
  const transition=readEntryTransition(record,id)!;
  const selection=await readFrozenEntrySelection(async()=>proof,{workspace:proof.workspace,namespace:proof.namespace,current:()=>true});
  expect(()=>requireEntryTransitionCurrent(selection,transition.after)).toThrow('earlier');
  expect(()=>readEntryTransition({...record,kind:['move']},id)).toThrow();
 });
});
