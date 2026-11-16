<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep a retained editor's viewport without replaying obsolete input

For maintainers changing simple browser tab or resource routing. EditorState owns text, selection and undo; the existing cached EditorView owns DOM and scroll. Detaching that DOM can clamp scroll offsets, so record the viewport before detachment and restore it after the exact retained view is mounted again.

## Admit one mount lease, not an old source spelling

Restoration is a display action, not source authority. Capture the exact buffer, editor record, parent, mount lease, authored text, CodeMirror document and selection, metadata identity and interaction epoch before scheduling. Typing, selection, scrolling, resource relocation, replacement of the cached record or a later mount retires the callback. A stale cleanup cannot detach a newer mount. Retain CodeMirror’s own scroll snapshot with the exact document so its next measurement also honors the viewport, rather than anchoring a transient detached geometry. Numeric offsets remain a fallback when the document changed while hidden. Rapid tab switching keeps an unapplied saved viewport rather than overwriting it with a transient detached zero.

<ui:copy id="simple-editor-mounts-browser">
import type {EditorView} from '@codemirror/view';
import type {Buffer} from './buffers';
export interface RetainedEditorMount {view:EditorView;interaction:number;viewport?:{top:number;left:number;snapshot?:ReturnType<EditorView["scrollSnapshot"]>;document?:EditorView["state"]["doc"]};mount?:symbol}
/** Capture while attached, before a resource route can detach its styled host.
 * A detached view has no reliable layout; keep a matching attached snapshot. */
export function captureRetainedEditorViewport(record:RetainedEditorMount){const view=record.view,top=view.scrollDOM.scrollTop,left=view.scrollDOM.scrollLeft,document=view.state.doc,previous=record.viewport;record.viewport={top,left,document,snapshot:!view.dom.isConnected&&previous?.top===top&&previous.left===left&&previous.document===document?previous.snapshot:view.scrollSnapshot()};}
export function mountRetainedEditor(buffer:Buffer,record:RetainedEditorMount,parent:HTMLElement,current:()=>boolean,
 schedule:(callback:()=>void)=>number=callback=>requestAnimationFrame(callback),cancel:(ticket:number)=>void=ticket=>cancelAnimationFrame(ticket)):()=>void {
 const view=record.view,lease=Symbol('editor mount'),saved=record.viewport,interaction=record.interaction,text=buffer.text,file=buffer.file,path=buffer.file.path,deleted=buffer.resourceDeleted,doc=view.state.doc,selection=view.state.selection;
 let mounted=true,restored=!saved;record.mount=lease;parent.append(view.dom);
 const ticket=saved?schedule(()=>{
  if(!mounted||!current()||record.mount!==lease||view.dom.parentElement!==parent||record.interaction!==interaction||buffer.file!==file||buffer.file.path!==path||buffer.resourceDeleted!==deleted||buffer.text!==text||view.state.doc!==doc||!view.state.selection.eq(selection))return;
  view.scrollDOM.scrollTop=saved.top;view.scrollDOM.scrollLeft=saved.left;if(saved.snapshot&&saved.document===view.state.doc)view.dispatch({effects:saved.snapshot});restored=true;
 }):null;
 return ()=>{if(!mounted)return;mounted=false;if(ticket!==null)cancel(ticket);if(record.mount!==lease||!current())return;
  if(restored||record.interaction!==interaction||buffer.text!==text||view.state.doc!==doc||!view.state.selection.eq(selection))captureRetainedEditorViewport(record);
  delete record.mount;if(view.dom.parentElement===parent)view.dom.remove();
 };
}
</ui:copy>
<ui:file path="apps/web/src/simple/EditorMounts.ts"><ui:paste select="#simple-editor-mounts-browser"/></ui:file>

## Hold real editor callbacks across route and input changes

These focused witnesses use an actual CodeMirror EditorView with a held animation scheduler. They prove the lease and source/input refusal paths independently of browser layout; the real resource/debug browser witness separately exercises measured scroll, retained DOM, selection and geometry through move, swap, deletion and undo.

<ui:copy id="simple-editor-mounts-browser-tests">
import {afterEach,expect,it} from 'vitest';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {Buffer} from './buffers';
import {captureRetainedEditorViewport,mountRetainedEditor,type RetainedEditorMount} from './EditorMounts';
const views:EditorView[]=[];afterEach(()=>{for(const view of views.splice(0))view.destroy();document.body.replaceChildren();});
function setup(){const buffer=new Buffer({path:'source.py',name:'source.py',dir:false},'value = 1\n','hash',()=>{}),view=new EditorView({state:EditorState.create({doc:buffer.text})});views.push(view);const parent=document.createElement('div');document.body.append(parent);const record:RetainedEditorMount={view,interaction:0,viewport:{top:80,left:9}},callbacks:(()=>void)[]=[];let owner=record;const mount=()=>mountRetainedEditor(buffer,record,parent,()=>owner===record,callback=>callbacks.push(callback),()=>{});return {buffer,view,parent,record,callbacks,mount,replace:()=>owner={...record}};}
it('restores only the exact retained editor and leaves its document/selection untouched',()=>{const {view,record,callbacks,mount}=setup(),state=view.state;const release=mount();callbacks[0]();expect(view.scrollDOM.scrollTop).toBe(80);expect(view.scrollDOM.scrollLeft).toBe(9);expect(view.state).toBe(state);view.scrollDOM.scrollTop=123;release();expect(record.viewport?.top).toBe(123);});
it('refuses obsolete input, authored revisions, route identity and cached-owner replies',()=>{
 for(const change of ['selection','text','scroll','route','route-in-place','delete','owner'] as const){const value=setup(),release=value.mount();
  if(change==='selection')value.view.dispatch({selection:{anchor:3}});
  if(change==='text'){value.view.dispatch({changes:{from:0,insert:'#'}});value.buffer.edit(value.view.state.doc.toString());}
  if(change==='scroll'){value.record.interaction++;value.view.scrollDOM.scrollTop=17;}
  if(change==='route')value.buffer.file={...value.buffer.file,path:'moved.py'};
  if(change==='route-in-place')value.buffer.file.path='moved.py';
  if(change==='delete')value.buffer.resourceDeleted=true;
  if(change==='owner')value.replace();
  value.callbacks[0]();expect(value.view.scrollDOM.scrollTop).toBe(change==='scroll'?17:0);release();
 }
});
it('leaves a replacement cache owner mounted when the old record is cleaned up',()=>{const value=setup(),old=value.mount(),replacement=value.replace();const next=mountRetainedEditor(value.buffer,replacement,value.parent,()=>true,callback=>value.callbacks.push(callback),()=>{});old();value.callbacks[0]();expect(value.view.dom.parentElement).toBe(value.parent);value.callbacks[1]();expect(value.view.scrollDOM.scrollTop).toBe(80);next();});
it('retains the attached document snapshot when cleanup runs after host detachment',()=>{const {view,record,mount,callbacks}=setup(),release=mount();callbacks[0]();view.scrollDOM.scrollTop=100;captureRetainedEditorViewport(record);const snapshot=record.viewport?.snapshot;view.dom.parentElement?.remove();release();expect(record.viewport?.top).toBe(100);expect(record.viewport?.snapshot).toBe(snapshot);});
it('never lets an old callback or cleanup detach the next mount, including rapid remounts',()=>{const {view,parent,record,callbacks,mount}=setup(),first=mount(),second=mount();callbacks[0]();expect(view.scrollDOM.scrollTop).toBe(0);first();expect(view.dom.parentElement).toBe(parent);callbacks[1]();expect(view.scrollDOM.scrollTop).toBe(80);second();const third=mount();third();callbacks[2]();expect(record.viewport?.top).toBe(80);expect(view.dom.isConnected).toBe(false);});
</ui:copy>
<ui:file path="apps/web/src/simple/EditorMounts.test.ts"><ui:paste select="#simple-editor-mounts-browser-tests"/></ui:file>
## Focused resource identity receipt — 2026-11-16

`HICKORY_SIMPLE_TEST_ENGINE="$PWD/target/source-edit-focus/debug/examples/engine_client" python3 scripts/test-action-chooser-browser.py --debug-resources` passed against the actual provider and protocol-3 engine. A paused source moved, swapped, was deleted, restored through workspace Undo/Redo and had its old spelling independently recreated. The same cached editor, caret/selection, horizontal/vertical scroll and measured geometry remained intact; one immutable debugger launch retained watch and Stop controls. Failed Save As attempts preserved the draft and independent old-path occupant; successful Save As after confirmed Stop retired only the original retained handle. A stopping/unconfirmed session remains live, so its source is not retired merely because Stop was requested.

The five focused EditorMounts cases also passed, including held callbacks after typing, selection/scroll changes, route changes, replacement mounts and detached cleanup. Resource publication Undo/Redo was exercised through the UI; transport uncertainty and stale-input refusal have controller/model evidence rather than an induced real network interruption. Physical input and accessibility acceptance remain unverified. No broad regression run was performed for this receipt.

</ui:doc>
