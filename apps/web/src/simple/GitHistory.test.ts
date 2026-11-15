import {it,expect} from 'vitest';
import {GitHistoryModel} from './GitHistory';
import type {CommitReadingData} from '../api/git';
it('retains the requested full commit identity across held and closed reads',async()=>{
 const a='a'.repeat(40),b='b'.repeat(40),pending:((value:CommitReadingData)=>void)[]=[];
 const model=new GitHistoryModel({log:async()=>({repository:true,commits:[a,b].map(sha=>({sha,short:sha.slice(0,7),author:'fixture',subject:'saved commit'}))}),read:()=>new Promise(resolve=>pending.push(resolve))});
 await model.refresh();const first=model.select(a),second=model.select(b);pending[1]({sha:b,parent:a,message:'new',files:[]});await second;pending[0]({sha:a,parent:null,message:'obsolete',files:[]});await first;expect(model.reading?.sha).toBe(b);expect(model.reading?.message).toBe('new');
 const closed=model.select(a);model.close();pending[2]({sha:a,parent:null,message:'closed',files:[]});await closed;expect(model.reading).toBeNull();
 const foreign=model.select(a);pending[3]({sha:b,parent:null,message:'foreign',files:[]});await foreign;expect(model.reading).toBeNull();expect(model.notice).toContain('identity changed');
});
it('refuses malformed log identities before admitting immutable reading choices',async()=>{
 const model=new GitHistoryModel({log:async()=>({repository:true,commits:[{sha:'HEAD',short:'HEAD',author:'fixture',subject:'ambiguous'}]}),read:async()=>{throw new Error('must not read');}});
 await model.refresh(2000);expect(model.limit).toBe(1000);expect(model.commits).toEqual([]);expect(model.notice).toContain('Malformed commit identity');
});

it('an observed full commit request reads directly without manufacturing a log choice',async()=>{
 const sha='c'.repeat(40),calls:string[]=[];
 const model=new GitHistoryModel({log:async()=>{throw new Error('must not discover');},read:async value=>{calls.push(value);return {sha:value,parent:null,message:'observed',files:[]};}},()=>{},{kind:'commit',sha,title:'Saved attribution'});
 await model.refresh();expect(calls).toEqual([sha]);expect(model.reading?.sha).toBe(sha);await model.readCommit('12345678');expect(calls).toEqual([sha]);expect(model.reading).toBeNull();
 await model.readCommit('d'.repeat(40));expect(calls).toEqual([sha]);expect(model.reading).toBeNull();
});
