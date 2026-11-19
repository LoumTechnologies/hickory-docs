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
