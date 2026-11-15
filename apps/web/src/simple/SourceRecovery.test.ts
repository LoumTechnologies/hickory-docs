import {it,expect} from 'vitest';
import {RecoveryReader,type RecoveryPage,type RecoveryReading} from './SourceRecovery';
const id='a'.repeat(32),workspace='b'.repeat(32);
const page=():RecoveryPage=>({version:1,workspace,ids:[id],next:null,ignored:0});
const reading=(fingerprint='c'.repeat(64)):RecoveryReading=>({version:1,workspace,id,fingerprint,title:'Move draft',title_truncated:false,paths:[{path:'a #😀.md',saved_before:{kind:'file',bytes:0},draft_before:{kind:'file',bytes:2},saved_after:{kind:'missing'},draft_after:{kind:'missing'}}],all_paths_recorded_applied:false,unrecognized_files:[],draft_text:'😀',saved_text:''});
it('recovery reads retain empty saved text separately from dirty drafts and encode the exact path',async()=>{
 const routes:string[]=[];const model=new RecoveryReader(async<T>(route:string)=>{routes.push(route);return (route.includes('resource-recoveries')?page():reading()) as T;});await model.list();await model.read(id,'a #😀.md','saved_before');expect(model.value?.saved_text).toBe('');expect(model.value?.draft_text).toBe('😀');expect(routes.at(-1)).toContain('path=a%20%23%F0%9F%98%80.md&saved=before');
});
it('changed record evidence and workspace pages refuse; closed and superseded replies cannot replace selection',async()=>{
 let response:unknown=page();const model=new RecoveryReader(async<T>()=>response as T);await model.list();response=reading();await model.read(id);response=reading('d'.repeat(64));await model.read(id,'a #😀.md');expect(model.value).toBeNull();expect(model.notice).toContain('evidence changed');
 response=page();await model.list();response={...page(),workspace:'other'};await model.list(id);expect(model.page).toBeNull();
 let finish!:(value:unknown)=>void;let defer=false;const delayed=new RecoveryReader(async<T>(route:string)=>(defer?await new Promise<unknown>(resolve=>finish=resolve):route.includes('resource-recoveries')?page():reading()) as T);await delayed.list();defer=true;const old=delayed.read(id);defer=false;await delayed.list();finish(reading());await old;expect(delayed.value).toBeNull();defer=true;const closed=delayed.read(id);delayed.close();finish(reading());await closed;expect(delayed.value).toBeNull();expect(delayed.page).toBeNull();
});
