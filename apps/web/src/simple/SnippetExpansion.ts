import {sourceChangeOffset} from './ChangePlan';
import {completionInsertion,type CompletionExpansion} from './CompletionPlan';
import {utf16ToPosition} from '../lsp/positions';
export interface SnippetField {index:number;from:number;to:number;choices:string[]}
export function completionExpansion(item:unknown):CompletionExpansion|undefined {
 const value=item as {insertTextFormat?:unknown;data?:{hickory_snippet?:unknown}};
 if(value?.insertTextFormat!==2)return undefined;
 const raw=value.data?.hickory_snippet as {version?:unknown;syntax?:unknown;text?:unknown;fields?:unknown}|undefined;
 if(!raw||raw.version!==1||raw.syntax!==completionInsertion(item)||typeof raw.text!=='string'||raw.text.length>1048576||!Array.isArray(raw.fields)||!raw.fields.length||raw.fields.length>4096)throw new Error('Snippet expansion unavailable or out of date. Request completions again.');
 const text=raw.text;
 const number=(value:unknown)=>{if(typeof value!=='number'||!Number.isSafeInteger(value)||value<0)throw new Error('Malformed snippet field');return value;};
 const fields:SnippetField[]=raw.fields.map(value=>{
  if(!value||typeof value!=='object')throw new Error('Malformed snippet field');
  const field=value as Record<string,unknown>,index=number(field.index),from=number(field.from),to=number(field.to);
  if(index>1004096||from>to||to>text.length||!Array.isArray(field.choices)||field.choices.some(value=>typeof value!=='string'))throw new Error('Malformed snippet field');
  sourceChangeOffset(text,utf16ToPosition(text,from));sourceChangeOffset(text,utf16ToPosition(text,to));
  return {index,from,to,choices:field.choices as string[]};
 });
 if(fields.filter(field=>field.index===0).length!==1)throw new Error('Malformed snippet exit');
 const first=new Map<number,SnippetField>();
 for(const field of fields){const previous=first.get(field.index);if(previous&&(text.slice(previous.from,previous.to)!==text.slice(field.from,field.to)||JSON.stringify(previous.choices)!==JSON.stringify(field.choices)))throw new Error('Inconsistent snippet mirrors');first.set(field.index,field);}
 const active=[...first.keys()].filter(index=>index>0).sort((a,b)=>a-b)[0]??0,selected=fields.find(field=>field.index===active)!;
 return {text,fields,selection:{from:selected.from,to:selected.to}};
}
