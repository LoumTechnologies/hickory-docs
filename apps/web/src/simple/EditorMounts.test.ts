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
