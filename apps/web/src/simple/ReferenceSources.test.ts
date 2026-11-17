import {expect,it} from 'vitest';
import {acquireReferenceSources,acquireSourceProposal} from './ReferenceSources';
function held(){let resolve!:(value:{text:string;writable:boolean})=>void;const promise=new Promise<{text:string;writable:boolean}>(done=>resolve=done);return {promise,resolve};}
const flush=async()=>{for(let i=0;i<6;i++)await Promise.resolve();};
it('synchronizes retained drafts once and returns only a fresh proposal without replacing snapshots',async()=>{
 const original={text:'original draft',writable:true};let round=0;const reads:string[]=[];
 const result=await acquireSourceProposal(new Map([['retained',original]]),async()=>({paths:['retained','retained'],value:++round}),async path=>{reads.push(path);return {...original};},()=>true);
 expect(result.value).toBe(2);expect(reads).toEqual(['retained']);expect(result.sources.get('retained')).toBe(original);
});
it('acquires new and retained targets sequentially, deduplicates and queries again',async()=>{
 let queries=0;const reads:{path:string;done:ReturnType<typeof held>}[]=[];
 const task=acquireReferenceSources(new Map([['retained',{text:'original draft',writable:true}]]),async()=>{queries++;return ['a','b','a','retained'];},path=>{const done=held();reads.push({path,done});return done.promise;},()=>true);
 await flush();expect(reads.map(row=>row.path)).toEqual(['a']);reads[0].done.resolve({text:'one',writable:true});await flush();expect(reads.map(row=>row.path)).toEqual(['a','b']);expect(queries).toBe(1);
 reads[1].done.resolve({text:'two',writable:true});await flush();expect(reads.map(row=>row.path)).toEqual(['a','b','retained']);reads[2].done.resolve({text:'original draft',writable:true});const result=await task;expect(queries).toBe(2);expect(result.references).toEqual(['a','b','retained']);
});
it('refuses retained text or writable drift and readonly new sources',async()=>{
 for(const next of [{text:'changed',writable:true},{text:'original',writable:false}])await expect(acquireReferenceSources(new Map([['a',{text:'original',writable:true}]]),async()=>['a'],async()=>next,()=>true)).rejects.toThrow('changed');
 await expect(acquireReferenceSources(new Map(),async()=>['a'],async()=>({text:'source',writable:false}),()=>true)).rejects.toThrow('read-only');
});
it('stale acquisition settles its read without dispatching another source or query',async()=>{
 let current=true,queries=0;const reads:ReturnType<typeof held>[]=[];const task=acquireReferenceSources(new Map(),async()=>{queries++;return ['a','b','c'];},()=>{const read=held();reads.push(read);return read.promise;},()=>current);const rejected=expect(task).rejects.toThrow('changed');await flush();current=false;reads[0].resolve({text:'source',writable:true});await rejected;expect(reads).toHaveLength(1);expect(queries).toBe(1);
});
it('refuses expanding and oversized discovery without dispatching past the bound',async()=>{
 let round=0;await expect(acquireReferenceSources(new Map(),async()=>[`new-${round++}`],async()=>({text:'source',writable:true}),()=>true)).rejects.toThrow('kept expanding');expect(round).toBe(4);
 let reads=0;await expect(acquireReferenceSources(new Map(),async()=>Array.from({length:10001},(_,i)=>String(i)),async()=>{reads++;return {text:'',writable:true};},()=>true)).rejects.toThrow('10,000');expect(reads).toBe(0);
});
it('admits readonly context only with explicit nonediting policy and still refuses drift',async()=>{
 const readonly={text:'context',writable:false};const result=await acquireSourceProposal(new Map([['context',readonly]]),async()=>({paths:['context'],value:1}),async()=>({...readonly}),()=>true,()=>false);expect(result.sources.get('context')).toBe(readonly);
 await expect(acquireSourceProposal(new Map([['context',readonly]]),async()=>({paths:['context'],value:1}),async()=>({text:'context',writable:true}),()=>true,()=>false)).rejects.toThrow('changed');
});
