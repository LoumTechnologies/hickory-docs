import {expect,it} from 'vitest';
import vectors from '../../../simple-contract/snippet-session-cases.json';
import type {SnippetField} from './SnippetExpansion';
import {beginSnippet,editSnippet,advanceSnippet,type SnippetSession} from './SnippetSession';
interface Step {action:string;from:number;to:number;text?:string;repeat?:number;currentSource?:string;refuse?:boolean;after?:string;fields?:SnippetField[];selection?:{from:number;to:number};active?:number|null}
interface Case {name:string;source:string;fields:SnippetField[];invalidBegin?:boolean;steps?:Step[]}
it('derives complete linked source edits, field scope and traversal from shared vectors',()=>{
 for(const value of vectors.cases as unknown as Case[]){
  let session:SnippetSession|null=beginSnippet(value.source,value.fields),source=value.source;
  if(value.invalidBegin){expect(session,value.name).toBeNull();continue;}expect(session,value.name).not.toBeNull();
  for(const step of value.steps!){const before=JSON.stringify(session),current=session!,input=step.currentSource??source;
   const result=step.action==='edit'?editSnippet(current,input,step.from,step.to,(step.text??'').repeat(step.repeat??1)):advanceSnippet(current,input,step.from,step.to,step.action==='next'?1:-1);
   expect(JSON.stringify(session),value.name).toBe(before);if(step.refuse){expect(result,value.name).toBeNull();continue;}expect(result,value.name).not.toBeNull();
   expect(result!.selection,value.name).toEqual(step.selection);
   if(step.action==='edit'){const edited=result as NonNullable<ReturnType<typeof editSnippet>>;expect(edited.session.source,value.name).toBe(step.after);source=edited.session.source;}
   session=result!.session;
   if(step.fields)expect(session?.fields,value.name).toEqual(step.fields);
   if('active' in step)expect(session?.active??null,value.name).toBe(step.active);
  }
 }
});
