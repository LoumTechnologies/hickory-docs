import {describe,it,expect,vi} from 'vitest';
import {Buffer,Buffers} from './buffers';
import {api} from '../api/client';
import {EditorView} from '@codemirror/view';
const views=vi.hoisted(()=>new WeakMap());
vi.mock('./Document',()=>({editorForBuffer:(buffer:Buffer)=>views.get(buffer)}));
import {EditorState} from '@codemirror/state';
import {history,undoDepth} from '@codemirror/commands';
import {routeResourceEditors,adoptResourceReceipt,captureResourceEditors,type ResourceEditorPatch,type ResourceReceipt,type ResourceSaved} from './ResourceEditors';
// Contract: .hickory-source/frontends/contract.md (resource namespace preservation).
const source=(identity:string,text=identity)=>({identity,text});
describe('resource editor identity routing',()=>{
 it('adopts a confirmed move, deletion and restoration into live buffers with separate saved baselines',async()=>{
  const buffers=new Buffers(()=>{}),a=new Buffer({path:'old.py',name:'old.py',dir:false},'saved','old-hash',()=>{});a.edit('dirty');buffers.values.set('old.py',a);
  const state=EditorState.create({doc:'saved',extensions:[history()]}).update({changes:{from:0,to:5,insert:'dirty'},selection:{anchor:2}}).state;
  const view=new EditorView({state});views.set(a,{view});view.scrollDOM.scrollTop=73;
  const retained=buffers.retainedResources,saved:ResourceSaved={kind:'file',text:'dirty',sha256:'a'.repeat(64),readonly:false,executable:false,mode:null};
  function adopt(patches:ResourceEditorPatch[],baseline:ResourceSaved,restoring=new Map<string,Buffer>()){
   const expected={request:'request',workspace:'workspace',context:'context',saved:new Map(patches.map(p=>[p.path,p.after?baseline:{kind:'missing'} as ResourceSaved]))};
   const receipt:ResourceReceipt={version:1,request_id:'request',workspace:'workspace',context:'context',outcome:{complete:true,recovery:'recovery',applied:patches.map(p=>p.path)},saved_sources:Object.fromEntries(expected.saved),source_files:patches.filter(p=>p.after).map(p=>({file:{path:p.path,name:p.path,dir:false},saved:baseline,hash:'fresh'}))};
   return adoptResourceReceipt(buffers,patches,receipt,expected,captureResourceEditors(buffers,patches,restoring),restoring);
  }
  const original=view.state;
  adopt([{path:'old.py',before:source('a','dirty'),after:null},{path:'new.py',before:null,after:source('a','dirty')}],saved);
  expect(buffers.values.get('new.py')).toBe(a);expect(view.state).toBe(original);expect(a.base).toBe('dirty');expect(a.dirty).toBe(false);
  a.base='saved';
  adopt([{path:'new.py',before:source('a','dirty'),after:null}],saved);expect(retained.get('a')).toBe(a);expect(a.resourceDeleted).toBe(true);expect(buffers.hasUnsavedChanges).toBe(true);
  const write=vi.spyOn(api,'saveFile').mockRejectedValue(new Error('Deleted editors cannot write.'));await a.save();expect(write).not.toHaveBeenCalled();write.mockRestore();
  adopt([{path:'old.py',before:null,after:source('restored','dirty')}],{...saved,text:'saved'},new Map([['restored',a]]));
  expect(buffers.values.get('old.py')).toBe(a);expect(a.base).toBe('saved');expect(a.text).toBe('dirty');expect(a.dirty).toBe(true);expect(a.resourceDeleted).toBe(false);
  expect(view.state).toBe(original);expect(view.state.selection.main.anchor).toBe(2);expect(undoDepth(view.state)).toBe(1);expect(view.scrollDOM.scrollTop).toBe(73);view.destroy();
 });
 it('refuses partial receipts and changed local baselines without changing any live address',()=>{
  const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'a.py',name:'a.py',dir:false},'a','hash',()=>{});buffers.values.set('a.py',buffer);
  const patches=[{path:'a.py',before:source('a'),after:null}],expected={request:'r',workspace:'w',context:'c',saved:new Map<string,ResourceSaved>([['a.py',{kind:'missing'}]])};
  const receipt:ResourceReceipt={version:1,request_id:'r',workspace:'w',context:'c',outcome:{complete:false,recovery:'recovery',applied:[]},saved_sources:null,source_files:[]},retained=buffers.retainedResources;
  expect(()=>adoptResourceReceipt(buffers,patches,receipt,expected,captureResourceEditors(buffers,patches))).toThrow();
  const current=captureResourceEditors(buffers,patches);buffer.base='independent';
  expect(()=>adoptResourceReceipt(buffers,patches,{...receipt,outcome:{complete:true,recovery:'recovery',applied:['a.py']},saved_sources:{'a.py':{kind:'missing'}}},expected,current)).toThrow();
  expect(buffers.values.get('a.py')).toBe(buffer);expect(buffer.base).toBe('independent');expect(retained.size).toBe(0);
 });

 it('retains editor state through swaps, deletion and explicit restoration',()=>{
  const a={selection:[2,4],scroll:108,undo:['old'],draft:'dirty'},b={selection:[0,0],scroll:0,undo:[],draft:'B'};
  const live=new Map([['a',{editor:a,source:'dirty'}],['b',{editor:b,source:'B'}]]);
  const patches:ResourceEditorPatch[]=[{path:'a.py',before:source('a','dirty'),after:source('b','B')},{path:'b.py',before:source('b','B'),after:source('a','dirty')}];
  const swap=routeResourceEditors(patches,live);expect(swap.find(x=>x.to==='b.py')?.editor).toBe(a);expect(swap.find(x=>x.to==='a.py')?.editor).toBe(b);
  const deleted=routeResourceEditors([{path:'b.py',before:source('a','dirty'),after:null}],new Map([['a',live.get('a')!]]));expect(deleted[0].editor).toBe(a);expect(deleted[0].to).toBeNull();
  const restored=routeResourceEditors([{path:'b.py',before:null,after:source('restored','dirty')}],new Map(),new Map([['restored',live.get('a')!]]));expect(restored[0].editor).toBe(a);
  const created=routeResourceEditors([{path:'new.py',before:null,after:source('new','')}],new Map());expect(created[0].editor).toBeNull();expect(created[0].after).toBe('');
  expect(a).toEqual({selection:[2,4],scroll:108,undo:['old'],draft:'dirty'});
 });
 it('retains actual CodeMirror selection and undo history through a move and inverse',()=>{
  let state=EditorState.create({doc:'saved',extensions:[history()]});state=state.update({changes:{from:5,insert:' dirty'},selection:{anchor:3}}).state;
  const editor={state,scrollTop:201},capture={editor,source:state.doc.toString()},a=source('a',capture.source);
  const moved=routeResourceEditors([{path:'old.py',before:a,after:null},{path:'new.py',before:null,after:a}],new Map([['a',capture]]));
  const inverse=routeResourceEditors([{path:'new.py',before:a,after:null},{path:'old.py',before:null,after:a}],new Map([['a',capture]]));
  expect(moved[0].editor).toBe(editor);expect(inverse[0].editor?.state).toBe(state);expect(editor.state.selection.main.anchor).toBe(3);expect(undoDepth(editor.state)).toBe(1);expect(editor.scrollTop).toBe(201);
 });
 it('refuses duplicate, stale, missing and aliased captures before mutation',()=>{
  const editor={undo:['retained']},capture={editor,source:'a'},live=new Map([['a',capture]]),patch={path:'a.py',before:source('a'),after:source('a','changed')};
  expect(()=>routeResourceEditors([patch,patch],live)).toThrow();expect(()=>routeResourceEditors([patch],new Map())).toThrow();expect(()=>routeResourceEditors([patch],new Map([['a',{editor,source:'later'}]]))).toThrow();
  expect(()=>routeResourceEditors([patch],live,new Map([['a',capture]]))).toThrow();expect(()=>routeResourceEditors([patch,{path:'b.py',before:source('b'),after:null}],new Map([...live,['b',{editor,source:'b'}]]))).toThrow();
  expect(editor.undo).toEqual(['retained']);
  const exact=routeResourceEditors([{path:'é.py',before:null,after:source('é')},{path:'e\u0301.py',before:null,after:source('e\u0301')}],new Map());expect(exact).toHaveLength(2);
 });
});
