import {EditorSelection,EditorState,Facet,StateEffect,StateField,type Extension,type Transaction,type TransactionSpec} from '@codemirror/state';
import {invertedEffects} from '@codemirror/commands';
export interface AuthoredEdit {from:number;to:number;insert:string}
interface SourceSnapshot {source:string;eols:readonly string[];crlf:readonly number[]}
const seed=Facet.define<string,string>({combine:values=>values[0]??''});
const replace=StateEffect.define<{origin:SourceSnapshot|null;source?:string;eols:readonly string[]}>();
export function normalizedSource(source:string):string{return source.replace(/\r\n?|\n/g,'\n');}
function endings(source:string):string[]{return source.match(/\r\n?|\n/g)??[];}
function scalar(source:string,at:number){if(!Number.isSafeInteger(at)||at<0||at>source.length||at>0&&at<source.length&&source.charCodeAt(at-1)>=0xd800&&source.charCodeAt(at-1)<=0xdbff&&source.charCodeAt(at)>=0xdc00&&source.charCodeAt(at)<=0xdfff)throw Error('Source position splits a Unicode scalar or is outside the current source.');}
function crlfOffsets(source:string):number[]{const result:number[]=[];for(let offset=source.indexOf('\r\n');offset>=0;offset=source.indexOf('\r\n',offset+2))result.push(offset);return result;}
function lowerBound(values:readonly number[],predicate:(value:number,index:number)=>boolean):number{let low=0,high=values.length;while(low<high){const middle=(low+high)>>>1;if(predicate(values[middle],middle))low=middle+1;else high=middle;}return low;}
function toAuthored(source:string,crlf:readonly number[],offset:number):number{if(!Number.isSafeInteger(offset)||offset<0||offset>source.length-crlf.length)throw Error('Editor position is outside the current source.');const raw=offset+lowerBound(crlf,(at,index)=>at-index<offset);scalar(source,raw);return raw;}
function toEditor(source:string,crlf:readonly number[],offset:number):number{scalar(source,offset);if(offset>0&&source[offset-1]==='\r'&&source[offset]==='\n')throw Error('Source position is inside a CRLF separator.');return offset-lowerBound(crlf,at=>at+1<offset);}
export function authoredOffsetIn(source:string,offset:number):number{return toAuthored(source,crlfOffsets(source),offset);}
export function editorOffsetIn(source:string,offset:number):number{return toEditor(source,crlfOffsets(source),offset);}

function restore(normalized:string,eols:readonly string[]):string{let index=0;const source=normalized.replace(/\n/g,()=>{const value=eols[index++];if(!['\n','\r','\r\n'].includes(value))throw Error('Authored line-ending history is unavailable.');return value;});if(index!==eols.length)throw Error('Authored line-ending history does not match the editor.');return source;}
function snapshot(source:string,previous?:SourceSnapshot):SourceSnapshot{const next=endings(source),eols=previous&&previous.eols.length===next.length&&next.every((value,index)=>value===previous.eols[index])?previous.eols:next;return {source,eols,crlf:crlfOffsets(source)};}
function preferred(source:string,at:number):string{const after=source.slice(at).match(/\r\n?|\n/);return after?.[0]??endings(source.slice(0,at)).at(-1)??'\n';}
function advanceSource(previous:SourceSnapshot,tr:Transaction):SourceSnapshot {const explicit=tr.effects.filter(effect=>effect.is(replace)).at(-1)?.value;
  if(explicit){if(explicit.origin&&explicit.origin!==previous)throw Error('Authored edit belongs to an obsolete editor revision.');const source=explicit.source??restore(tr.newDoc.toString(),explicit.eols);if(normalizedSource(source)!==tr.newDoc.toString())throw Error('Authored edit does not match the editor transaction.');return snapshot(source,previous);}
  if(!tr.docChanged)return previous;let source='',last=0;
  tr.changes.iterChanges((from,to,_fromNew,_toNew,insert)=>{const start=toAuthored(previous.source,previous.crlf,from),end=toAuthored(previous.source,previous.crlf,to);source+=previous.source.slice(last,start)+insert.toString().replace(/\n/g,preferred(previous.source,start));last=end;});source+=previous.source.slice(last);return snapshot(source,previous);
}
const field=StateField.define<SourceSnapshot>({
 create(state){const source=state.facet(seed);if(normalizedSource(source)!==state.doc.toString())throw Error('Authored text does not match the initial editor.');return snapshot(source);},
 update:advanceSource
});
export function sourceText(source:string):Extension{return [seed.of(source),field,invertedEffects.of(tr=>{const before=tr.startState.field(field,false);return before&&(tr.docChanged||authoredTransactionSource(tr)!==before.source)?[replace.of({origin:null,eols:before.eols})]:[];})];}
export function authoredTransactionSource(tr:Transaction):string {const previous=tr.startState.field(field,false);return previous?advanceSource(previous,tr).source:tr.newDoc.toString();}
export function authoredSource(state:EditorState):string{return state.field(field,false)?.source??state.doc.toString();}
export function authoredOffset(state:EditorState,offset:number):number{const value=state.field(field,false);return value?toAuthored(value.source,value.crlf,offset):authoredOffsetIn(state.doc.toString(),offset);}
export function editorOffset(state:EditorState,offset:number):number{const value=state.field(field,false);return value?toEditor(value.source,value.crlf,offset):editorOffsetIn(state.doc.toString(),offset);}
export function authoredSelection(state:EditorState){return state.selection.ranges.map(range=>({anchor:authoredOffset(state,range.anchor),head:authoredOffset(state,range.head)}));}
export function authoredChanges(state:EditorState,edits:readonly AuthoredEdit[]):TransactionSpec{
 const source=authoredSource(state),sorted=[...edits].sort((a,b)=>a.from-b.from||a.to-b.to);let last=0,next='';const changes=[];
 for(const edit of sorted){scalar(source,edit.from);scalar(source,edit.to);if(edit.to<edit.from||edit.from<last||typeof edit.insert!=='string')throw Error('Authored edits overlap or are malformed.');const from=editorOffset(state,edit.from),to=editorOffset(state,edit.to);changes.push({from,to,insert:normalizedSource(edit.insert)});next+=source.slice(last,edit.from)+edit.insert;last=edit.to;}next+=source.slice(last);
 const original=state.field(field,false);if(!original&&normalizedSource(next)!==next)throw Error('Byte-preserving source editing is unavailable on this editor.');return {changes,...(original?{effects:[replace.of({origin:original,source:next,eols:endings(next)})]}:{})};
}
export function authoredInsert(state:EditorState,text:string):TransactionSpec{
 const ranges=state.selection.ranges.map(range=>({from:authoredOffset(state,range.from),to:authoredOffset(state,range.to),insert:text})),spec=authoredChanges(state,ranges),change=state.changes(spec.changes);return {...spec,selection:EditorSelection.create(state.selection.ranges.map(range=>EditorSelection.cursor(change.mapPos(range.to,1))),state.selection.mainIndex),userEvent:'input.paste'};
}
