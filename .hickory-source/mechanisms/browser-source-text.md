<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep authored bytes while the browser editor displays normalized lines

For maintainers connecting UTF-16 provider coordinates to a retained browser editor.

The editor displays one line break per authored CRLF, LF or CR separator. Its source authority remains the exact original string. A state field follows edits against the immutable preimage, preserves untouched separators, and chooses the local separator for newly typed line breaks. Explicit authored changes preserve their inserted bytes. Neither capability arrival nor opening text creates an edit.

## Translate only at the editor boundary

Provider and host models use authored UTF-16 offsets. Display offsets collapse CRLF into one position. Refuse positions inside a CRLF pair or Unicode scalar rather than rounding a provider edit onto another character. Legacy test editors without this extension retain LF behavior; they cannot silently accept raw CR edits.

Reviewed resource publication supplies a complete after image. Diff its
display text against the mounted editor so selections and undo map through
the changed spans. Keep the exact authored after image in the same
transaction, including changes to line endings that do not alter the display.

<ui:copy id="browser-source-text-core">
import {EditorSelection,EditorState,Facet,StateEffect,StateField,type Extension,type Transaction,type TransactionSpec} from '@codemirror/state';
import {invertedEffects} from '@codemirror/commands';
import {computeEdits} from '../lib/diff';
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
export function authoredReplacement(state: EditorState, source: string): TransactionSpec {
 const displayed = normalizedSource(source);
 const original = state.field(field, false);
 if (!original && displayed !== source) throw Error('Byte-preserving source editing is unavailable on this editor.');
 const changes = computeEdits(state.doc.toString(), displayed)
  .map(({start, end, text}) => ({from: start, to: end, insert: text}));
 return {
  changes,
  ...(original ? {effects: [replace.of({origin: original, source, eols: endings(source)})]} : {}),
 };
}
export function authoredInsert(state:EditorState,text:string):TransactionSpec{
 const ranges=state.selection.ranges.map(range=>({from:authoredOffset(state,range.from),to:authoredOffset(state,range.to),insert:text})),spec=authoredChanges(state,ranges),change=state.changes(spec.changes);return {...spec,selection:EditorSelection.create(state.selection.ranges.map(range=>EditorSelection.cursor(change.mapPos(range.to,1))),state.selection.mainIndex),userEvent:'input.paste'};
}
</ui:copy>

## Preserve line-ending authority through native editor undo

CodeMirror owns text and selection undo. Its inverse effects retain only line-ending sequences, without duplicating complete source strings in every history entry. Undo and redo reconstruct the raw source from their restored display text and the matching retained separator ledger. An explicit edit is tied to the original state snapshot and cannot be reused after a different revision.

<ui:file path="apps/web/src/simple/SourceText.ts"><ui:paste select="#browser-source-text-core"/></ui:file>

## Check byte identity, offset refusal and real editor history

These focused cases use CodeMirror state and its real history commands. They distinguish opening from editing, preserve mixed separator sequences through grouped typing and provider changes, and refuse stale raw-coordinate transactions. The browser witness separately verifies the mounted editor and asynchronous capabilities. Per-revision CRLF indices make attached-editor coordinate reads logarithmic rather than scanning the complete document for every field decoration. Rich inline prose, folding and indentation intentionally read normalized display text; their positions are paint geometry and never provider edit authority.

<ui:copy id="browser-source-text-tests">
import {describe,it,expect} from 'vitest';
import {EditorState,type StateCommand,type TransactionSpec} from '@codemirror/state';
import {history,undo,redo,isolateHistory} from '@codemirror/commands';
import {sourceText,authoredSource,authoredOffset,editorOffset,authoredOffsetIn,editorOffsetIn,authoredChanges,authoredReplacement,authoredInsert} from './SourceText';
function fixture(source:string){let state=EditorState.create({doc:source,extensions:[history(),sourceText(source)]});return {get state(){return state;},apply(spec:TransactionSpec){state=state.update(spec).state;},command(command:StateCommand){return command({state,dispatch:tr=>{state=tr.state;}});}};}
describe('authored browser source text',()=>{
 it('opens mixed separators without changing bytes and maps authored UTF16 positions',()=>{const source='one\r\ntwo\n😀 three\r\nfour',f=fixture(source);expect(f.state.doc.toString()).toBe('one\ntwo\n😀 three\nfour');expect(authoredSource(f.state)).toBe(source);for(let offset=0;offset<=source.length;offset++){if(offset>0&&source[offset-1]==='\r'&&source[offset]==='\n'||offset>0&&/[\uD800-\uDBFF]/.test(source[offset-1])&&/[\uDC00-\uDFFF]/.test(source[offset]??''))continue;expect(authoredOffset(f.state,editorOffset(f.state,offset))).toBe(offset);}});
 it('refuses CRLF interior, split scalars and out-of-range coordinates',()=>{for(const offset of [2,5,-1,99])expect(()=>editorOffsetIn('a\r\nb😀',offset)).toThrow();expect(()=>authoredOffsetIn('a\r\nb😀',4)).toThrow();});
 it('typing retains untouched mixed endings and new Enter uses its local line ending',()=>{const f=fixture('a\r\nb\nc\r\n');f.apply({changes:{from:1,insert:'!'}});expect(authoredSource(f.state)).toBe('a!\r\nb\nc\r\n');f.apply({changes:{from:1,insert:'\n'}});expect(authoredSource(f.state)).toBe('a\r\n!\r\nb\nc\r\n');});
 it('explicit provider edits preserve exact inserted mixed separators and offset preimages',()=>{const f=fixture('a\r\nb\nc');f.apply(authoredChanges(f.state,[{from:3,to:4,insert:'B\nextra\r\n'}]));expect(authoredSource(f.state)).toBe('a\r\nB\nextra\r\n\nc');expect(f.state.doc.toString()).toBe('a\nB\nextra\n\nc');});
 it('native Undo and Redo restore mixed bytes even when history joins newline edits',()=>{const original='a\r\nb\nc\r\n',f=fixture(original);f.apply({changes:{from:1,insert:'\n'},userEvent:'input.type'});f.apply({changes:{from:2,insert:'x'},userEvent:'input.type'});const changed=authoredSource(f.state);expect(changed).not.toBe(original);while(f.command(undo)){}expect(authoredSource(f.state)).toBe(original);while(f.command(redo)){}expect(authoredSource(f.state)).toBe(changed);});
 it('provider insertion undo restores old EOL ledger and redo restores exact new raw bytes',()=>{const original='a\r\nb\nc',f=fixture(original);f.apply({...authoredChanges(f.state,[{from:0,to:3,insert:'A\nx\r\n'}]),annotations:isolateHistory.of('full')});const changed=authoredSource(f.state);expect(f.command(undo)).toBe(true);expect(authoredSource(f.state)).toBe(original);expect(f.command(redo)).toBe(true);expect(authoredSource(f.state)).toBe(changed);});
 it('keeps a selection on the surviving source across reviewed replacements', () => {
  const before = 'heading 😀\r\nlet selected = 1;\r\nkeep this\nfooter';
  const after = 'heading 😀\nimport \"./new\";\r\nkeep this\r\nfooter';
  const f = fixture(before);
  const start = f.state.doc.toString().indexOf('keep this');
  f.apply({selection: {anchor: start + 9, head: start}});
  for (const source of [after, before, after]) {
   f.apply({...authoredReplacement(f.state, source), annotations: isolateHistory.of('full')});
   const range = f.state.selection.main;
   expect(f.state.doc.sliceString(range.from, range.to)).toBe('keep this');
   expect(range.anchor).toBe(range.to);
   expect(authoredSource(f.state)).toBe(source);
  }
  expect(f.command(undo)).toBe(true);
  expect(authoredSource(f.state)).toBe(before);
 });
 it('refuses applying a prepared authored edit after an intervening revision',()=>{const f=fixture('a\r\nb'),held=authoredChanges(f.state,[{from:3,to:4,insert:'B'}]);f.apply({changes:{from:0,insert:'x'}});expect(()=>f.apply(held)).toThrow(/obsolete/);expect(authoredSource(f.state)).toBe('xa\r\nb');});
 it('raw multi-line paste preserves clipboard bytes, replaces selection and undoes exactly',()=>{const f=fixture('a\r\nb\nc');f.apply({selection:{anchor:2,head:3}});f.apply(authoredInsert(f.state,'B\nC\r\n'));expect(authoredSource(f.state)).toBe('a\r\nB\nC\r\n\nc');expect(f.state.selection.main.empty).toBe(true);expect(f.command(undo)).toBe(true);expect(authoredSource(f.state)).toBe('a\r\nb\nc');});
 it('capability-only configuration preserves raw text, selection and undo authority',()=>{const f=fixture('a\r\nb\nc');f.apply({changes:{from:1,insert:'!'},selection:{anchor:4}});const source=authoredSource(f.state),selection=f.state.selection;f.apply({});expect(authoredSource(f.state)).toBe(source);expect(f.state.selection.eq(selection)).toBe(true);expect(f.command(undo)).toBe(true);expect(authoredSource(f.state)).toBe('a\r\nb\nc');});
});
</ui:copy>
<ui:file path="apps/web/src/simple/SourceText.test.ts"><ui:paste select="#browser-source-text-tests"/></ui:file>

</ui:doc>
