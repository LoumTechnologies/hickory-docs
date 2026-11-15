import {planWorkspaceEdit,sourceChangeOffset,type ChangePlan,type SourceReplacement,type SourceSnapshot} from './ChangePlan';
import {semanticPath} from './Intelligence';
import type {SnippetField} from './SnippetExpansion';
import type {SnippetSession} from './SnippetSession';
import {utf16ToPosition} from '../lsp/positions';
export interface CompletionExpansion {text:string;selection?:{from:number;to:number};fields?:SnippetField[]}
export interface CompletionDraft {path:string;before:string;plan:ChangePlan;insertionFrom:number;selection:{from:number;to:number};fields:SnippetField[];continuing?:SnippetSession;primary?:SourceReplacement;additional?:SourceReplacement[]}
export function completionInsertion(item:unknown):string {
 const value=item as Record<string,unknown>|null;if(!value||typeof value!=='object'||typeof value.label!=='string')throw new Error('Malformed completion');
 const edit=value.textEdit as Record<string,unknown>|undefined;
 if(edit!==undefined&&(!edit||typeof edit!=="object"||Array.isArray(edit)))throw new Error("Malformed completion edit");
 const text=edit?edit.newText:value.insertText??value.label;
 if(typeof text!=='string')throw new Error('Missing completion insertion text');return text;
}
export function planCompletion(uri:string,item:unknown,snapshot:SourceSnapshot,from:number,to:number,expansion?:CompletionExpansion):CompletionDraft {
 const source=snapshot.text;if(!snapshot.writable)throw new Error("Completion source is generated, read-only or saving");
 const value=item as Record<string,unknown>|null,path=semanticPath(uri);if(!path||!value||typeof value!=='object')throw new Error('Unsupported completion source identity');
 if(value.command!==undefined&&value.command!==null)throw new Error('This completion requires a provider command; no edits were applied.');
 const format=value.insertTextFormat===undefined?1:value.insertTextFormat;if(format!==1&&format!==2)throw new Error('Unsupported completion insertion format');
 const rawText=completionInsertion(value);if(format===2&&!expansion)throw new Error('Snippet expansion unavailable; no edits were applied.');
 const inserted=format===2?expansion!:{text:rawText};
 const validateOffset=(text:string,offset:number)=>{if(!Number.isSafeInteger(offset)||offset<0||offset>text.length)throw new Error('Invalid completion selection');sourceChangeOffset(text,utf16ToPosition(text,offset));};
 validateOffset(source,from);validateOffset(source,to);if(from>to)throw new Error('Completion fallback runs backwards');
 const edit=value.textEdit as Record<string,unknown>|undefined;
 if(edit){if(edit.range!==undefined&&(edit.insert!==undefined||edit.replace!==undefined))throw new Error("Ambiguous completion edit");const range=(edit.range??edit.replace) as {start?:unknown;end?:unknown}|undefined;if(!range)throw new Error('Missing completion range');from=sourceChangeOffset(source,range.start);to=sourceChangeOffset(source,range.end);if(from>to)throw new Error('Completion range runs backwards');}
 let additions:SourceReplacement[]=[];
 if(value.additionalTextEdits!==undefined){if(!Array.isArray(value.additionalTextEdits))throw new Error('Malformed additional completion edits');
  additions=value.additionalTextEdits.map(raw=>{const edit=raw as {range?:{start?:unknown;end?:unknown};newText?:unknown}|null;if(!edit?.range||typeof edit.newText!=='string')throw new Error('Malformed additional completion edit');const change={from:sourceChangeOffset(source,edit.range.start),to:sourceChangeOffset(source,edit.range.end),text:edit.newText};if(change.from>change.to)throw new Error('Additional completion edit runs backwards');return change;});
 }
 const unique=additions.filter((edit,index)=>additions.findIndex(other=>other.from===edit.from&&other.to===edit.to&&other.text===edit.text)===index);
 // A completion names its primary edit explicitly. Imports inserted at its
 // start precede that insertion, in the provider's declared additional order.
 const prefix=unique.filter(edit=>edit.from===from&&edit.to===from).map(edit=>edit.text).join('');
 const others=unique.filter(edit=>edit.from!==from||edit.to!==from),primary={from,to,text:prefix+inserted.text};
 const rawEdits=[primary,...others].map(edit=>({range:{start:utf16ToPosition(source,edit.from),end:utf16ToPosition(source,edit.to)},newText:edit.text}));
 const plan=planWorkspaceEdit(`Complete ${String(value.label)}`,{changes:{[uri]:rawEdits}},new Map([[path,snapshot]]),true);
 const delta=others.filter(edit=>edit.from<from).reduce((sum,edit)=>sum+edit.text.length-(edit.to-edit.from),0),insertionFrom=from+delta+prefix.length;
 const selected=inserted.selection??{from:inserted.text.length,to:inserted.text.length};validateOffset(inserted.text,selected.from);validateOffset(inserted.text,selected.to);if(selected.from>selected.to)throw new Error('Completion selection runs backwards');
 const fields=(inserted.fields??[]).map(field=>{validateOffset(inserted.text,field.from);validateOffset(inserted.text,field.to);if(field.from>field.to)throw new Error('Invalid snippet field');return {...field,from:insertionFrom+field.from,to:insertionFrom+field.to};});
 return {path,before:source,plan,insertionFrom,fields,primary:{from,to,text:inserted.text},additional:unique,selection:{from:insertionFrom+selected.from,to:insertionFrom+selected.to}};
}
