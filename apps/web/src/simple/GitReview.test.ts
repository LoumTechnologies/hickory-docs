import {it,expect} from 'vitest';
import {GitReviewModel} from './GitReview';
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
