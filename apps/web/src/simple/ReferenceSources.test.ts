import {expect,it} from 'vitest';
import {acquireReferenceSources} from './ReferenceSources';
function held(){let resolve!:(value:{text:string;writable:boolean})=>void;const promise=new Promise<{text:string;writable:boolean}>(done=>resolve=done);return {promise,resolve};}
const flush=async()=>{for(let i=0;i<6;i++)await Promise.resolve();};
it('loads at most two unknown targets and queries again after acquisition',async()=>{
 let queries=0;const reads:{path:string;done:ReturnType<typeof held>}[]=[];
 const result=acquireReferenceSources(new Map([['retained',{text:'original draft',writable:true}]]),async()=>{queries++;return ['a','b','a','c','retained'];},path=>{const done=held();reads.push({path,done});return done.promise;},()=>true);
 await flush();expect(reads.map(item=>item.path)).toEqual(['a','b']);expect(queries).toBe(1);reads[0].done.resolve({text:'one',writable:true});await flush();expect(reads.map(item=>item.path)).toEqual(['a','b','c']);expect(queries).toBe(1);
 reads[1].done.resolve({text:'two',writable:true});reads[2].done.resolve({text:'three',writable:true});const value=await result;expect(queries).toBe(2);expect(value.references).toEqual(['a','b','c','retained']);expect(value.sources.get('retained')?.text).toBe('original draft');expect(value.sources.size).toBe(4);
});
it('stale requests drain started reads and never dispatch remaining targets',async()=>{
 let current=true;const reads:ReturnType<typeof held>[]=[];const result=acquireReferenceSources(new Map(),async()=>['a','b','c'],()=>{const read=held();reads.push(read);return read.promise;},()=>current);const rejected=expect(result).rejects.toThrow('changed');await flush();current=false;reads.forEach(read=>read.resolve({text:'source',writable:true}));await rejected;expect(reads).toHaveLength(2);
 let round=0;await expect(acquireReferenceSources(new Map(),async()=>[`new-${round++}`],async()=>({text:'source',writable:true}),()=>true)).rejects.toThrow('kept expanding');expect(round).toBe(4);
});
