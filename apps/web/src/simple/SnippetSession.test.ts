import {expect,it} from 'vitest';
import vectors from '../../../simple-contract/snippet-session-cases.json';
import type {SnippetField} from './SnippetExpansion';
import {beginSnippet,editSnippet,advanceSnippet,completeSnippet,nestSnippet,type SnippetSession} from './SnippetSession';
interface Step {inner?:SnippetField[];additional?:{from:number;to:number;text:string}[];action:string;from:number;to:number;text?:string;repeat?:number;currentSource?:string;refuse?:boolean;after?:string;fields?:SnippetField[];selection?:{from:number;to:number};active?:number|null}
interface Case {name:string;source:string;fields:SnippetField[];invalidBegin?:boolean;steps?:Step[]}
it('derives complete linked source edits, field scope and traversal from shared vectors',()=>{
 for(const value of vectors.cases as unknown as Case[]){
  let session:SnippetSession|null=beginSnippet(value.source,value.fields),source=value.source;
  if(value.invalidBegin){expect(session,value.name).toBeNull();continue;}expect(session,value.name).not.toBeNull();
  for(const step of value.steps!){const before=JSON.stringify(session),current=session!,input=step.currentSource??source;
   const result=step.action==='nest'?nestSnippet(current,input,{from:step.from,to:step.to,text:step.text??''},step.additional??[],step.inner??[]):step.action==='complete'?completeSnippet(current,input,{from:step.from,to:step.to,text:step.text??''},step.additional??[]):step.action==='edit'?editSnippet(current,input,step.from,step.to,(step.text??'').repeat(step.repeat??1)):advanceSnippet(current,input,step.from,step.to,step.action==='next'?1:-1);
   expect(JSON.stringify(session),value.name).toBe(before);if(step.refuse){expect(result,value.name).toBeNull();continue;}expect(result,value.name).not.toBeNull();
   expect(result!.selection,value.name).toEqual(step.selection);
   if(step.action==='edit'||step.action==='complete'||step.action==='nest'){const edited=result as NonNullable<ReturnType<typeof editSnippet>>;expect(edited.session.source,value.name).toBe(step.after);source=edited.session.source;}
   session=result!.session;
   if(step.fields)expect(session?.fields,value.name).toEqual(step.fields);
   if('active' in step)expect(session?.active??null,value.name).toBe(step.active);
  }
 }
});
it('rejects excessive begin metadata and predicts nested choice fanout before cloning',()=>{
 const field=(id:number,index:number,choices:string[]=[])=>({id,parent:null,index,from:0,to:0,choices});
 expect(beginSnippet('',[field(0,1,Array<string>(4097).fill('')),field(1,0)])).toBeNull();
 const outer=beginSnippet('',[...Array.from({length:8},(_,id)=>field(id,1)),field(8,0)])!,before=JSON.stringify(outer);let reads=0;
 const choices=new Proxy(Array<string>(4096).fill(''),{get(target,key,receiver){if(typeof key==='string'&&/^[0-9]+$/.test(key))reads++;return Reflect.get(target,key,receiver);}}),inner=[field(0,1,choices),field(1,0)];
 expect(nestSnippet(outer,'',{from:0,to:0,text:''},[],inner)).toBeNull();expect(reads).toBe(4096);expect(JSON.stringify(outer)).toBe(before);
 const small=nestSnippet(outer,'',{from:0,to:0,text:''},[],[field(0,1,['','']),field(1,0)]);expect(small?.session.scopes).toHaveLength(1);
});
