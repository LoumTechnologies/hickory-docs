import type {SnippetField} from './SnippetExpansion';
export interface SnippetSession {source:string;fields:SnippetField[];active:number;current:number}
export interface SnippetEdit {from:number;to:number;text:string}
export interface SnippetInput {session:SnippetSession;edits:SnippetEdit[];selection:{from:number;to:number}}
const scalar=(text:string,at:number)=>Number.isSafeInteger(at)&&at>=0&&at<=text.length&&!(at>0&&at<text.length&&((text.charCodeAt(at-1)>=0xd800&&text.charCodeAt(at-1)<=0xdbff&&text.charCodeAt(at)>=0xdc00&&text.charCodeAt(at)<=0xdfff)||(text[at-1]==='\r'&&text[at]==='\n')));
export function beginSnippet(source:string,fields:readonly SnippetField[]):SnippetSession|null {
 if(!fields.length||fields.length>4096||fields.some(field=>field.id===undefined||!Number.isSafeInteger(field.id)||field.id!<0||field.id!>=4096||!scalar(source,field.from)||!scalar(source,field.to)||field.from>field.to||!Number.isSafeInteger(field.index)||field.index<0||field.index>1004096||field.parent!=null&&!Number.isSafeInteger(field.parent)))return null;
 const ids=new Map(fields.map(field=>[field.id!,field]));if(ids.size!==fields.length||fields.filter(field=>field.index===0).length!==1||fields.some(field=>field.index===0&&field.from!==field.to))return null;
 for(const field of fields){const seen=new Set<number>();let next:SnippetField|undefined=field;while(next){if(seen.has(next.id!))return null;seen.add(next.id!);if(next.parent==null)break;const parent=ids.get(next.parent!);if(!parent||parent.from>next.from||parent.to<next.to)return null;next=parent;}}
 const active=Math.min(...fields.map(field=>field.index).filter(index=>index>0));
 const first=fields.find(field=>field.index===active);return first?{source,fields:fields.map(field=>({...field,choices:[...field.choices],parent:field.parent??null})),active,current:first.id!}:null;
}
export function advanceSnippet(session:SnippetSession,source:string,from:number,to:number,direction:1|-1):{session:SnippetSession|null;selection:{from:number;to:number}}|null {
 if(source!==session.source||from>to||!session.fields.some(field=>field.index===session.active&&field.from<=from&&field.to>=to))return null;
 const order=[...new Set(session.fields.map(field=>field.index).filter(index=>index>0))].sort((a,b)=>a-b);order.push(0);
 const at=order.indexOf(session.active),next=at+direction;if(at<0||next<0||next>=order.length)return null;
 const active=order[next],field=session.fields.find(field=>field.index===active)!;
 return {session:active===0?null:{...session,active,current:field.id!},selection:{from:field.from,to:field.to}};
}
export function editSnippet(session:SnippetSession,source:string,from:number,to:number,text:string):SnippetInput|null {
 if(source!==session.source||from>to||!scalar(source,from)||!scalar(source,to)||session.active===0)return null;
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
    const enclosing=owners.map((id,index)=>({id,index})).filter(({id})=>field.index===0&&ancestors(id,field.id!));
    if(enclosing.length)return edit.from+delta+text.length*(enclosing.at(-1)!.index+1);
    return edit.from+delta+text.length*owners.filter(id=>ordinals.get(id)!<ordinals.get(field.id!)!).length;
   }
   const owns=edit.owners.has(field.id!)||[...edit.owners].some(id=>ancestors(field.id!,id));
   const exit=field.index===0&&[...edit.owners].some(id=>ancestors(id,field.id!));
   const right=owns?end:exit?true:position===edit.to;
   return edit.from+delta+(right?edit.text.length:0);
  }return position+delta;};
 const retained=fields.filter(field=>after===source||field.index===0||field.index===session.active || !occurrences.some(parent=>ancestors(parent.id!,field.id!)));
 const retainedIds=new Set(retained.map(field=>field.id!));
 const updated=retained.map(field=>{let parent=field.parent;while(parent!==null&&parent!==undefined&&!retainedIds.has(parent))parent=ids.get(parent)?.parent;return {...field,parent:parent??null,from:map(field,field.from,false),to:map(field,field.to,true)};});
 if(updated.some(field=>field.from>field.to||!scalar(after,field.from)||!scalar(after,field.to)))return null;
 const groups=new Map<number,string>();for(const field of updated){const value=after.slice(field.from,field.to),previous=groups.get(field.index);if(previous!==undefined&&previous!==value)return null;groups.set(field.index,value);}
 const caret=map(origin,from,false)+text.length;
 return {session:{source:after,fields:updated,active:session.active,current:origin.id!},edits:replacements.map(({from,to,text})=>({from,to,text})),selection:{from:caret,to:caret}};
}
