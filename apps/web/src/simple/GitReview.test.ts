import {it,expect} from 'vitest';
import {GitReviewModel,reviewIndexPaths} from './GitReview';
import type {GitDiff} from '../api/git';
it('retains only the latest saved diff identity and retires closed work',async()=>{
 const pending:((value:GitDiff)=>void)[]=[];
 const model=new GitReviewModel({changes:async()=>({repository:true,branch:'main',files:['a.py','b.py'].map(path=>({path,index:'M',tree:'M'}))}),diff:()=>new Promise(resolve=>pending.push(resolve))});
 await model.refresh();const first=model.select('a.py',false),second=model.select('b.py',true);
 pending[1]({path:'b.py',diff:'new staged',binary:false});await second;pending[0]({path:'a.py',diff:'obsolete',binary:false});await first;
 expect(model.selected).toBe('b.py');expect(model.staged).toBe(true);expect(model.detail).toBe('new staged');
 const closed=model.select('a.py',false);model.close();pending[2]({path:'a.py',diff:'late after close',binary:false});await closed;expect(model.detail).toBe('');
 const wrong=model.select('a.py',false);pending[3]({path:'b.py',diff:'wrong source',binary:false});await wrong;expect(model.detail).toBe('');expect(model.notice).toContain('identity changed');
});

it('stages exact rename paths once and requires a fresh read after an unconfirmed write',async()=>{
 const calls:{paths:string[];unstage:boolean}[]=[];let fail=false;
 const files=[{path:'new.py',from:'old.py',index:'R',tree:'M'}];
 const model=new GitReviewModel({changes:async()=>({repository:true,files}),diff:async path=>({path,diff:'captured',binary:false}),writeIndex:async(paths,unstage)=>{calls.push({paths,unstage});if(fail)throw new Error('connection ended');return {ok:true};}});
 await model.refresh();await model.changeIndex('new.py',true);expect(calls).toEqual([{paths:['old.py','new.py'],unstage:true}]);expect(model.needsRefresh).toBe(false);
 fail=true;await model.changeIndex('new.py',false);expect(calls).toHaveLength(2);expect(model.needsRefresh).toBe(true);await model.changeIndex('new.py',false);expect(calls).toHaveLength(2);
 await model.refresh();fail=false;await model.changeIndex('new.py',false);expect(calls).toHaveLength(3);expect(model.needsRefresh).toBe(false);
 expect(()=>reviewIndexPaths({...files[0],from_outside_workspace:true},true)).toThrow('repository root');
});

it('keeps the index mutation guard across close and a read while the write is pending',async()=>{
 let finish!:(value:{ok:boolean})=>void,calls=0;
 const model=new GitReviewModel({changes:async()=>({repository:true,files:[{path:'a.py',index:' ',tree:'M'}]}),diff:async path=>({path,diff:'',binary:false}),writeIndex:()=>{calls++;return new Promise(resolve=>finish=resolve);}});
 await model.refresh();const write=model.changeIndex('a.py',false);model.close();await model.refresh();expect(model.indexPending).toBe(true);expect(model.needsRefresh).toBe(true);
 await model.changeIndex('a.py',false);expect(calls).toBe(1);finish({ok:true});await write;expect(model.needsRefresh).toBe(true);expect(model.indexPending).toBe(false);await model.refresh();expect(model.needsRefresh).toBe(false);
});
