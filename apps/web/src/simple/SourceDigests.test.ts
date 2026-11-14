import {expect,it} from 'vitest';
import {SourceDigests} from './SourceDigests';
function deferred(){let resolve!:(value:string)=>void;const promise=new Promise<string>(done=>resolve=done);return {promise,resolve};}
const flush=async()=>{await Promise.resolve();await Promise.resolve();};
it('bounds hashing, coalesces queued drafts and rejects obsolete bytes',async()=>{
 const jobs:{source:string;done:ReturnType<typeof deferred>}[]=[];const index=new SourceDigests(source=>{const done=deferred();jobs.push({source,done});return done.promise;});
 let source='é';index.enqueue('a',source,()=>source);source='e\u0301';index.enqueue('a','intermediate',()=>source);index.enqueue('a',source,()=>source);index.enqueue('b','other',()=> 'other');expect(jobs).toHaveLength(1);
 jobs[0].done.resolve('old');await flush();expect(index.current('a','é')).toBeNull();expect(jobs[1].source).toBe(source);jobs[1].done.resolve('new');await flush();expect(index.current('a',source)).toBe('new');expect(index.current('a','é')).toBeNull();expect(jobs[2].source).toBe('other');
 index.stop();jobs[2].done.resolve('late');await flush();expect(index.current('b','other')).toBeNull();
});
it('an optional digest failure does not prevent another source from completing',async()=>{
 const index=new SourceDigests(async source=>{if(source==='bad')throw new Error('unavailable');return 'good';});index.enqueue('a','bad',()=> 'bad');index.enqueue('b','ok',()=> 'ok');await flush();await flush();expect(index.current('a','bad')).toBeNull();expect(index.current('b','ok')).toBe('good');index.stop();
});
