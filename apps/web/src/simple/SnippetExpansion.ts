import {sourceChangeOffset} from './ChangePlan';
import {readSnippetTransform,applySnippetTransform,snippetTransformBytes,type SnippetTransform,type SnippetTransformBudget} from './SnippetTransform';
import {completionInsertion,type CompletionExpansion} from './CompletionPlan';
import {utf16ToPosition} from '../lsp/positions';
export interface SnippetField {index:number;from:number;to:number;choices:string[];id?:number;parent?:number|null;transform?:SnippetTransform}
export function completionExpansion(item:unknown,budget:SnippetTransformBudget={work:0}):CompletionExpansion|undefined {
 const value=item as {insertTextFormat?:unknown;data?:{hickory_snippet?:unknown}};
 if(value?.insertTextFormat!==2)return undefined;
 const raw=value.data?.hickory_snippet as {version?:unknown;syntax?:unknown;text?:unknown;fields?:unknown}|undefined;
 if(!raw||![1,2].includes(raw.version as number)||raw.syntax!==completionInsertion(item)||typeof raw.text!=='string'||new TextEncoder().encode(raw.text).length>1048576||!Array.isArray(raw.fields)||!raw.fields.length||raw.fields.length>4096)throw new Error('Snippet expansion unavailable or out of date. Request completions again.');
 const text=raw.text;let metadataBytes=0;
 const number=(value:unknown)=>{if(typeof value!=='number'||!Number.isSafeInteger(value)||value<0)throw new Error('Malformed snippet field');return value;};
 const fields:SnippetField[]=raw.fields.map(value=>{
  if(!value||typeof value!=='object')throw new Error('Malformed snippet field');
  const field=value as Record<string,unknown>,index=number(field.index),from=number(field.from),to=number(field.to);
  if(index>1004096||from>to||to>text.length||!Array.isArray(field.choices))throw new Error('Malformed snippet field');
  if(field.choices.length>4096||metadataBytes+64+field.choices.length*32>1048576)throw new Error('Snippet metadata expansion limit exceeded');
  metadataBytes+=64;for(const choice of field.choices){if(typeof choice!=='string')throw new Error('Malformed snippet field');metadataBytes+=32+new TextEncoder().encode(choice).length;if(metadataBytes>1048576)throw new Error('Snippet metadata expansion limit exceeded');}
  sourceChangeOffset(text,utf16ToPosition(text,from));sourceChangeOffset(text,utf16ToPosition(text,to));
  const ancestry:Pick<SnippetField,'id'|'parent'>={};
  if(field.id!==undefined){ancestry.id=number(field.id);ancestry.parent=field.parent===null?null:number(field.parent);if(ancestry.id>=4096||ancestry.parent!==null&&ancestry.parent>=4096)throw new Error('Malformed snippet ancestry');}
  else if(field.parent!==undefined)throw new Error('Malformed snippet ancestry');
  const transform=field.transform===undefined||field.transform===null?undefined:readSnippetTransform(field.transform);
  if(transform){if(raw.version!==2||index===0||field.choices.length||ancestry.id===undefined)throw Error('Computed snippet metadata requires protocol 2 and an editable source group');metadataBytes+=snippetTransformBytes(transform);if(metadataBytes>1048576)throw Error('Snippet metadata expansion limit exceeded');}
  return {index,from,to,choices:field.choices as string[],...ancestry,...(transform?{transform}:{})};
 });
 if(fields.some(field=>field.id!==undefined)){
  const ids=new Map(fields.map(field=>[field.id,field]));
  if(ids.has(undefined)||ids.size!==fields.length)throw new Error('Malformed snippet ancestry');
  const visited=new Set<number>();
  for(const field of fields){const chain=new Set<number>();let next:SnippetField|undefined=field;while(next&&!visited.has(next.id!)){if(chain.has(next.id!))throw new Error('Cyclic snippet ancestry');chain.add(next.id!);if(next.parent===null)break;const parent=ids.get(next.parent!);if(!parent||parent.from>next.from||parent.to<next.to)throw new Error('Malformed snippet ancestry');next=parent;}for(const id of chain)visited.add(id);}
 }
 if(fields.filter(field=>field.index===0).length!==1)throw new Error('Malformed snippet exit');
 validateSnippetMirrors(text,fields,budget);
 const active=Math.min(...fields.filter(field=>!field.transform&&field.index>0).map(field=>field.index),Infinity),selected=fields.find(field=>!field.transform&&field.index===(active===Infinity?0:active))!;
 return {text,fields,selection:{from:selected.from,to:selected.to}};
}

export function snippetTransformOrder(fields:readonly SnippetField[]):number[] {
 const ids=new Map(fields.map(field=>[field.id,field])),ordinary=new Set(fields.filter(field=>!field.transform).map(field=>field.index)),dependencies=new Map<number,Set<number>>([...ordinary].map(index=>[index,new Set()]));let work=0,edges=0;
 for(const field of fields.filter(field=>field.transform)){
  if(field.index===0||field.id===undefined||field.choices.length||!ordinary.has(field.index)||fields.some(child=>child.parent===field.id))throw Error('Malformed computed snippet occurrence');
  const seen=new Set<number>();let next=field.parent==null?undefined:ids.get(field.parent);
  while(next){if(++work>65536||next.id===undefined||seen.has(next.id))throw Error('Computed snippet ancestry limit exceeded');seen.add(next.id);if(next.index>0){if(next.index===field.index)throw Error('Cyclic computed snippet dependency');const values=dependencies.get(next.index);if(!values)throw Error('Computed snippet parent is not editable');if(!values.has(field.index)){if(++edges>4096)throw Error('Computed snippet dependency limit exceeded');values.add(field.index);}}next=next.parent==null?undefined:ids.get(next.parent);}
 }
 const parents=new Map<number,number[]>();for(const [parent,values] of dependencies)for(const child of values){const list=parents.get(child)??[];list.push(parent);parents.set(child,list);}
 const queue=[...dependencies].filter(([,values])=>!values.size).map(([index])=>index).sort((a,b)=>a-b),order:number[]=[];
 for(let at=0;at<queue.length;at++){const child=queue[at];order.push(child);for(const parent of parents.get(child)??[]){const values=dependencies.get(parent)!;values.delete(child);if(!values.size)queue.push(parent);}}
 if(order.length!==ordinary.size)throw Error('Cyclic computed snippet dependency');return order;
}
export function validateSnippetMirrors(text:string,fields:readonly SnippetField[],budget:SnippetTransformBudget={work:0}):number[] {
 const first=new Map<number,SnippetField>();for(const field of fields.filter(field=>!field.transform)){const previous=first.get(field.index);if(previous&&(text.slice(previous.from,previous.to)!==text.slice(field.from,field.to)||JSON.stringify(previous.choices)!==JSON.stringify(field.choices)))throw Error('Inconsistent snippet mirrors');first.set(field.index,field);}
 const order=snippetTransformOrder(fields);for(const field of fields.filter(field=>field.transform)){const source=first.get(field.index);if(!source||applySnippetTransform(field.transform!,text.slice(source.from,source.to),budget)!==text.slice(field.from,field.to))throw Error('Inconsistent computed snippet occurrence');}return order;
}
