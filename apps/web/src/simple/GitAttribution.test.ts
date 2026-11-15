import {it,expect} from 'vitest';
import {GitAttributionModel,attributionCommit} from './GitAttribution';
import type {BlameLine} from '../api/types';
const row=(line:number,full_commit?:string):BlameLine=>({line,commit:'12345678',full_commit,author:'Fixture',email:'',time:0,summary:'base',uncommitted:false,text:'saved'});
it('keeps captured paths and line identities separate from obsolete replies',async()=>{
 let finish!:(value:{path:string;lines:BlameLine[]})=>void,calls=0;const model=new GitAttributionModel('a.py',path=>++calls===1?new Promise(resolve=>finish=resolve):Promise.resolve({path,lines:[row(2,'a'.repeat(40))]}));
 const old=model.refresh();await model.refresh();finish({path:'a.py',lines:[row(1)]});await old;expect(model.lines.map(line=>line.line)).toEqual([2]);
 const closed=new GitAttributionModel('a.py',()=>new Promise(resolve=>finish=resolve));const held=closed.refresh();closed.close();finish({path:'a.py',lines:[row(1)]});await held;expect(closed.lines).toEqual([]);
 const foreign=new GitAttributionModel('cafe\u0301.py',async()=>({path:'caf\u00e9.py',lines:[row(1)]}));await foreign.refresh();expect(foreign.lines).toEqual([]);expect(foreign.notice).toContain('identity changed');
 const duplicate=new GitAttributionModel('a.py',async path=>({path,lines:[row(1),row(1)]}));await duplicate.refresh();expect(duplicate.lines).toEqual([]);expect(duplicate.notice).toContain('Ambiguous');
});
it('never turns a compact label or dirty line into an immutable reference',()=>{
 expect(attributionCommit(row(1))).toBeNull();expect(attributionCommit(row(1,'12345678'))).toBeNull();expect(attributionCommit({...row(1,'a'.repeat(40)),uncommitted:true})).toBeNull();expect(attributionCommit(row(1,'a'.repeat(40)))).toBe('a'.repeat(40));expect(attributionCommit(row(1,'b'.repeat(64)))).toBe('b'.repeat(64));
});
