import {validateSnippetMirrors,snippetTransformOrder,type SnippetField} from './SnippetExpansion';
import {applySnippetTransform,readSnippetTransform,snippetTransformBytes,type SnippetTransformBudget} from './SnippetTransform';
export interface SnippetScope {order:number[];exitIndex:number;exitId:number;outerActive:number;outerCurrent:number}
export interface SnippetSession {source:string;fields:SnippetField[];active:number;current:number;scopes?:SnippetScope[]}
export interface SnippetEdit {from:number;to:number;text:string}
export interface SnippetInput {session:SnippetSession;edits:SnippetEdit[];selection:{from:number;to:number}}
const scalar=(text:string,at:number)=>Number.isSafeInteger(at)&&at>=0&&at<=text.length&&!(at>0&&at<text.length&&((text.charCodeAt(at-1)>=0xd800&&text.charCodeAt(at-1)<=0xdbff&&text.charCodeAt(at)>=0xdc00&&text.charCodeAt(at)<=0xdfff)||(text[at-1]==='\r'&&text[at]==='\n')));
export function snippetMetadataCost(fields:readonly SnippetField[]):number|null {
 if(fields.length>4096)return null;let bytes=0;
 for(const field of fields){if(!field||!Array.isArray(field.choices)||field.choices.length>4096)return null;bytes+=64+field.choices.length*32;try{if(field.transform)bytes+=snippetTransformBytes(field.transform);}catch{return null;}if(bytes>1048576)return null;for(const choice of field.choices){if(typeof choice!=='string'||choice.length>1048576-bytes)return null;bytes+=choice.length?new TextEncoder().encode(choice).length:0;if(bytes>1048576)return null;}}
 return bytes;
}
export function beginSnippet(source:string,fields:readonly SnippetField[],budget:SnippetTransformBudget={work:0}):SnippetSession|null {
 if(!fields.length||fields.length>4096||snippetMetadataCost(fields)===null||fields.some(field=>field.id===undefined||!Number.isSafeInteger(field.id)||field.id!<0||field.id!>=4096||!scalar(source,field.from)||!scalar(source,field.to)||field.from>field.to||!Number.isSafeInteger(field.index)||field.index<0||field.index>1004096||field.parent!=null&&!Number.isSafeInteger(field.parent)))return null;
 const ids=new Map(fields.map(field=>[field.id!,field]));if(ids.size!==fields.length||fields.filter(field=>field.index===0).length!==1||fields.some(field=>field.index===0&&field.from!==field.to))return null;
 for(const field of fields){const seen=new Set<number>();let next:SnippetField|undefined=field;while(next){if(seen.has(next.id!))return null;seen.add(next.id!);if(next.parent==null)break;const parent=ids.get(next.parent!);if(!parent||parent.from>next.from||parent.to<next.to)return null;next=parent;}}
 if(fields.some(field=>field.transform)){try{validateSnippetMirrors(source,fields,budget);}catch{return null;}}
 const active=Math.min(...fields.filter(field=>!field.transform).map(field=>field.index).filter(index=>index>0));
 const first=fields.find(field=>!field.transform&&field.index===active);return first?{source,fields:fields.map(field=>({...field,choices:[...field.choices],parent:field.parent??null,...(field.transform?{transform:readSnippetTransform(field.transform)}:{})})),active,current:first.id!}:null;
}
export function advanceSnippet(session:SnippetSession,source:string,from:number,to:number,direction:1|-1):{session:SnippetSession|null;selection:{from:number;to:number}}|null {
 if(source!==session.source||from>to||!session.fields.some(field=>!field.transform&&field.index===session.active&&field.from<=from&&field.to>=to))return null;
 const scope=session.scopes?.at(-1),order=scope?[...scope.order,scope.exitIndex]:[...new Set(session.fields.filter(field=>!field.transform).map(field=>field.index).filter(index=>index>0))].sort((a,b)=>a-b).concat(0);
 const at=order.indexOf(session.active),next=at+direction;if(at<0||next<0||next>=order.length)return null;
 const active=order[next];
 if(scope&&active===scope.exitIndex){const field=session.fields.find(field=>field.id===scope.exitId);if(!field)return null;return {session:{...session,fields:session.fields.filter(field=>!scope.order.includes(field.index)&&field.index!==scope.exitIndex),active:scope.outerActive,current:scope.outerCurrent,scopes:session.scopes!.slice(0,-1)},selection:{from:field.from,to:field.to}};}
 const field=session.fields.find(field=>!field.transform&&field.index===active)!;
 return {session:active===0?null:{...session,active,current:field.id!},selection:{from:field.from,to:field.to}};
}

export function editSnippet(session:SnippetSession,source:string,from:number,to:number,text:string,budget:SnippetTransformBudget={work:0}):SnippetInput|null {
 if(session.fields.some(field=>field.transform))return editDerivedSnippet(session,source,from,to,text,budget);
 if(source!==session.source||from>to||!scalar(source,from)||!scalar(source,to)||session.active===0)return null;
 const terminal=(field:SnippetField)=>field.index===0||!!session.scopes?.some(scope=>scope.exitIndex===field.index);
 const fields=session.fields,ids=new Map(fields.map(field=>[field.id!,field]));
 const ancestors=(parent:number,child:number)=>{let current=ids.get(child);for(let steps=0;current&&steps<=fields.length;steps++){if(current.id===parent)return true;current=current.parent===null?undefined:ids.get(current.parent!);}return false;};
 const occurrences=fields.filter(field=>field.index===session.active),matches=occurrences.filter(field=>field.from<=from&&field.to>=to),origin=matches.find(field=>field.id===session.current)??matches[0];
 if(!origin||new TextEncoder().encode(text).length>1048576/Math.max(occurrences.length,1))return null;
 const before=source.slice(origin.from,origin.to),relativeFrom=from-origin.from,relativeTo=to-origin.from;
 if(occurrences.some(field=>source.slice(field.from,field.to)!==before))return null;
 const replacements:{from:number;to:number;text:string;owners:Set<number>}[]=[];
 for(const field of occurrences){const from=field.from+relativeFrom,to=field.from+relativeTo;if(!scalar(source,from)||!scalar(source,to))return null;const same=replacements.find(edit=>edit.from===from&&edit.to===to);if(same){if(from!==to)return null;same.text+=text;same.owners.add(field.id!);}else replacements.push({from,to,text,owners:new Set([field.id!])});}
 replacements.sort((a,b)=>a.from-b.from||a.to-b.to);
 if(replacements.some((edit,index)=>index>0&&replacements[index-1].to>edit.from))return null;
 let after=source;for(const edit of [...replacements].reverse())after=after.slice(0,edit.from)+edit.text+after.slice(edit.to);
 if(after===source)return {session:{...session,current:origin.id!},edits:replacements.map(({from,to,text})=>({from,to,text})),selection:{from:from+text.length,to:from+text.length}};
 const ordinals=new Map(fields.map((field,index)=>[field.id!,index]));
 const map=(field:SnippetField,position:number,end:boolean)=>{let delta=0;for(const edit of replacements){if(position<edit.from)break;if(position>edit.to){delta+=edit.text.length-(edit.to-edit.from);continue;}
   if(edit.from===edit.to){
    const owners=[...edit.owners],related=owners.map((id,index)=>({id,index})).filter(({id})=>ancestors(field.id!,id));
    if(related.length)return edit.from+delta+text.length*(end?related.at(-1)!.index+1:related[0].index);
    const enclosing=owners.map((id,index)=>({id,index})).filter(({id})=>terminal(field)&&ancestors(id,field.id!));
    if(enclosing.length)return edit.from+delta+text.length*(enclosing.at(-1)!.index+1);
    return edit.from+delta+text.length*owners.filter(id=>ordinals.get(id)!<ordinals.get(field.id!)!).length;
   }
   const owns=edit.owners.has(field.id!)||[...edit.owners].some(id=>ancestors(field.id!,id));
   const exit=terminal(field)&&[...edit.owners].some(id=>ancestors(id,field.id!));
   const right=owns?end:exit?true:position===edit.to;
   return edit.from+delta+(right?edit.text.length:0);
  }return position+delta;};
 const retained=fields.filter(field=>after===source||terminal(field)||field.index===session.active || !occurrences.some(parent=>ancestors(parent.id!,field.id!)));
 const retainedIds=new Set(retained.map(field=>field.id!));
 const updated=retained.map(field=>{let parent=field.parent;while(parent!==null&&parent!==undefined&&!retainedIds.has(parent))parent=ids.get(parent)?.parent;return {...field,parent:parent??null,from:map(field,field.from,false),to:map(field,field.to,true)};});
 if(updated.some(field=>field.from>field.to||!scalar(after,field.from)||!scalar(after,field.to)))return null;
 const groups=new Map<number,string>();for(const field of updated){const value=after.slice(field.from,field.to),previous=groups.get(field.index);if(previous!==undefined&&previous!==value)return null;groups.set(field.index,value);}
 const caret=map(origin,from,false)+text.length;
 return {session:{source:after,fields:updated,active:session.active,current:origin.id!,scopes:session.scopes},edits:replacements.map(({from,to,text})=>({from,to,text})),selection:{from:caret,to:caret}};
}
export function completeSnippet(session:SnippetSession,source:string,primary:SnippetEdit,additional:readonly SnippetEdit[],budget:SnippetTransformBudget={work:0}):SnippetInput|null {
 const input=editSnippet(session,source,primary.from,primary.to,primary.text,budget);if(!input)return null;
 if(!additional.length)return input;
 const start=Math.min(...session.fields.map(field=>field.from)),end=Math.max(...session.fields.map(field=>field.to));
 let shift=0,bytes=0;const edits=input.edits.map(edit=>({...edit}));
 for(const edit of [...additional].reverse()){
  if(edit.from>edit.to||!scalar(source,edit.from)||!scalar(source,edit.to))return null;
  bytes+=new TextEncoder().encode(edit.text).length;if(bytes>1048576)return null;
  if(edit.to<=start)shift+=edit.text.length-(edit.to-edit.from);else if(edit.from<end)return null;
  const same=edits.find(value=>value.from===edit.from);
  if(same){if(edit.from!==edit.to||edit.to>start)return null;same.text=edit.text+same.text;}else edits.push({...edit});
 }
 edits.sort((a,b)=>a.from-b.from||a.to-b.to);
 if(edits.some((edit,index)=>index>0&&(edits[index-1].to>edit.from||edits[index-1].from===edit.from)))return null;
 let after=source;for(const edit of [...edits].reverse())after=after.slice(0,edit.from)+edit.text+after.slice(edit.to);
 const fields=input.session.fields.map(field=>({...field,from:field.from+shift,to:field.to+shift}));
 if(!beginSnippet(after,fields,budget))return null;
 return {session:{...input.session,source:after,fields},edits,selection:{from:input.selection.from+shift,to:input.selection.to+shift}};
}
export function nestSnippet(session:SnippetSession,source:string,primary:SnippetEdit,additional:readonly SnippetEdit[],inner:readonly SnippetField[],budget:SnippetTransformBudget={work:0}):SnippetInput|null {
 if((session.scopes?.length??0)>=32||!inner.length||snippetMetadataCost(session.fields)===null)return null;
 const innerBytes=snippetMetadataCost(inner),exit=inner.find(field=>field.index===0);if(innerBytes===null||!exit)return null;
 const origin=session.fields.find(field=>field.id===session.current&&field.from<=primary.from&&field.to>=primary.to);if(!origin)return null;
 const input=completeSnippet(session,source,primary,additional,budget);if(!input)return null;
 const parents=input.session.fields.filter(field=>!field.transform&&field.index===session.active),current=parents.find(field=>field.id===input.session.current);if(!current)return null;
 const relative=primary.from-origin.from,order=[...new Set(inner.filter(field=>!field.transform&&field.index>0).map(field=>field.index))].sort((a,b)=>a-b);
 if(order.length){const retainedBytes=snippetMetadataCost(input.session.fields);if(retainedBytes===null||retainedBytes+parents.length*innerBytes>1048576||input.session.fields.length+parents.length*inner.length>4096)return null;}
 // Reserve the complete mirrored forest before beginSnippet can clone inner choices.
 const parsed=beginSnippet(primary.text,inner,budget);if(!parsed&&(inner.length!==1||exit.id===undefined||exit.parent!=null||exit.from!==exit.to||!scalar(primary.text,exit.from)))return null;
 if(!order.length){const caret=current.from+relative+exit.from;return {...input,selection:{from:caret,to:caret}};}
 let nextIndex=Math.max(...input.session.fields.map(field=>field.index))+1;const indices=new Map([...order,0].map(index=>[index,nextIndex++]));if(nextIndex>1004097)return null;
 const used=new Set(input.session.fields.map(field=>field.id!)),fields=[...input.session.fields];let nextId=0,exitId=-1,currentId=-1;
 for(const parent of parents){const ids=new Map<number,number>();for(const field of inner){while(used.has(nextId))nextId++;if(nextId>=4096)return null;ids.set(field.id!,nextId);used.add(nextId++);}
  for(const field of inner){const id=ids.get(field.id!)!;fields.push({...field,id,index:indices.get(field.index)!,parent:field.parent==null?parent.id!:ids.get(field.parent)!,from:parent.from+relative+field.from,to:parent.from+relative+field.to});if(parent.id===current.id){if(field.index===0)exitId=id;else if(field.id===parsed!.current)currentId=id;}}
 }
 if(snippetMetadataCost(fields)===null||!beginSnippet(input.session.source,fields,budget))return null;
 const scope:SnippetScope={order:order.map(index=>indices.get(index)!),exitIndex:indices.get(0)!,exitId,outerActive:session.active,outerCurrent:input.session.current},active=scope.order[0],selected=fields.find(field=>field.id===currentId);if(!selected)return null;
 return {...input,session:{...input.session,fields,active,current:currentId,scopes:[...(input.session.scopes??[]),scope]},selection:{from:selected.from,to:selected.to}};
}
// Computed occurrences are never editable sources. Build every base and derived
// replacement in original source coordinates before returning one action.
function editDerivedSnippet(session:SnippetSession,source:string,from:number,to:number,text:string,budget:SnippetTransformBudget):SnippetInput|null {
 if(source!==session.source||from>to||!scalar(source,from)||!scalar(source,to)||session.active===0||snippetMetadataCost(session.fields)===null)return null;
 try{
  validateSnippetMirrors(source,session.fields,budget);
  const fields=session.fields,ids=new Map(fields.map(field=>[field.id!,field])),ordinals=new Map(fields.map((field,index)=>[field.id!,index]));
  const ancestors=(parent:number,child:number)=>{let value=ids.get(child);for(let count=0;value&&count<=fields.length;count++){if(value.id===parent)return true;value=value.parent==null?undefined:ids.get(value.parent);}return false;};
  const terminal=(field:SnippetField)=>field.index===0||!!session.scopes?.some(scope=>scope.exitIndex===field.index),occurrences=fields.filter(field=>!field.transform&&field.index===session.active),matches=occurrences.filter(field=>field.from<=from&&field.to>=to),origin=matches.find(field=>field.id===session.current)??matches[0];
  if(!origin||new TextEncoder().encode(text).length>1048576/Math.max(occurrences.length,1))return null;
  const before=source.slice(origin.from,origin.to),relativeFrom=from-origin.from,relativeTo=to-origin.from;if(occurrences.some(field=>source.slice(field.from,field.to)!==before))return null;
  if(before.slice(0,relativeFrom)+text+before.slice(relativeTo)===before)return {session:{...session,current:origin.id!},edits:[],selection:{from:from+text.length,to:from+text.length}};
  let retained=fields.filter(field=>terminal(field)||field.index===session.active||!occurrences.some(parent=>ancestors(parent.id!,field.id!)));const ordinary=new Set(retained.filter(field=>!field.transform).map(field=>field.index));retained=retained.filter(field=>!field.transform||ordinary.has(field.index));const retainedIds=new Set(retained.map(field=>field.id!));
  retained=retained.map(field=>{let parent=field.parent;while(parent!=null&&!retainedIds.has(parent))parent=ids.get(parent)?.parent;return {...field,parent:parent??null};});
  interface Part {from:number;to:number;text:string;owner:number}
  const parts:Part[]=occurrences.map(field=>({from:field.from+relativeFrom,to:field.from+relativeTo,text,owner:field.id!}));let payload=parts.reduce((sum,part)=>sum+new TextEncoder().encode(part.text).length,0);
  if(parts.some(part=>!scalar(source,part.from)||!scalar(source,part.to)))return null;
  const compare=(a:Part,b:Part)=>a.from-b.from||a.to-b.to||ordinals.get(a.owner)!-ordinals.get(b.owner)!;
  const projected=(field:SnippetField)=>{const edits=parts.filter(part=>ancestors(field.id!,part.owner)).sort(compare);if(edits.some(part=>part.from<field.from||part.to>field.to))throw Error('Computed source edit escapes its parent');let value=source.slice(field.from,field.to);for(const part of edits.reverse())value=value.slice(0,part.from-field.from)+part.text+value.slice(part.to-field.from);return value;};
  const expected=new Map<number,string>();for(const index of snippetTransformOrder(retained)){const mirrors=retained.filter(field=>!field.transform&&field.index===index),computed=retained.filter(field=>field.transform&&field.index===index);if(!computed.length)continue;const value=projected(mirrors[0]);if(mirrors.some(field=>projected(field)!==value))return null;for(const field of computed){const result=applySnippetTransform(field.transform!,value,budget);expected.set(field.id!,result);if(result!==source.slice(field.from,field.to)){payload+=new TextEncoder().encode(result).length;if(payload>1048576)return null;parts.push({from:field.from,to:field.to,text:result,owner:field.id!});}}}
  interface Replacement {from:number;to:number;text:string;spans:{owner:number;from:number;to:number}[]}
  const replacements:Replacement[]=[];const sorted=parts.sort(compare);for(let at=0;at<sorted.length;){const start=sorted[at].from,group:Part[]=[];while(at<sorted.length&&sorted[at].from===start)group.push(sorted[at++]);if(group.filter(part=>part.to>part.from).length>1)return null;group.sort((a,b)=>ordinals.get(a.owner)!-ordinals.get(b.owner)!);const edit:Replacement={from:start,to:Math.max(...group.map(part=>part.to)),text:'',spans:[]};for(const part of group){const offset=edit.text.length;edit.text+=part.text;edit.spans.push({owner:part.owner,from:offset,to:edit.text.length});}if(replacements.length&&replacements.at(-1)!.to>start)return null;replacements.push(edit);}
  let after=source;for(const edit of [...replacements].reverse())after=after.slice(0,edit.from)+edit.text+after.slice(edit.to);
  const map=(field:SnippetField,position:number,end:boolean)=>{let delta=0;for(const edit of replacements){if(position<edit.from)break;if(position>edit.to||position===edit.to&&edit.from<edit.to){delta+=edit.text.length-(edit.to-edit.from);continue;}const related=edit.spans.filter(span=>ancestors(field.id!,span.owner));if(related.length)return edit.from+delta+(end?related.at(-1)!.to:related[0].from);const enclosing=edit.spans.filter(span=>terminal(field)&&ancestors(span.owner,field.id!));if(enclosing.length)return edit.from+delta+enclosing.at(-1)!.to;return edit.from+delta+edit.spans.filter(span=>ordinals.get(span.owner)!<ordinals.get(field.id!)!).reduce((sum,span)=>sum+span.to-span.from,0);}return position+delta;};
  const updated=retained.map(field=>({...field,from:map(field,field.from,false),to:map(field,field.to,true)}));if(updated.some(field=>field.from>field.to||!scalar(after,field.from)||!scalar(after,field.to)))return null;
  const groups=new Map<number,string>();for(const field of updated){const value=after.slice(field.from,field.to);if(field.transform){if(value!==expected.get(field.id!))return null;}else{const previous=groups.get(field.index);if(previous!==undefined&&previous!==value)return null;groups.set(field.index,value);}}
  const caret=map(origin,from,false)+text.length;return {session:{source:after,fields:updated,active:session.active,current:origin.id!,scopes:session.scopes},edits:replacements.map(({from,to,text})=>({from,to,text})),selection:{from:caret,to:caret}};
 }catch{return null;}
}
