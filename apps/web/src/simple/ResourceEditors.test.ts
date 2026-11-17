import {describe,it,expect,vi} from 'vitest';
import {Buffer,Buffers} from './buffers';
import {api} from '../api/client';
import {EditorView} from '@codemirror/view';
const views=vi.hoisted(()=>new WeakMap());
vi.mock('./Document',()=>({editorForBuffer:(buffer:Buffer)=>views.get(buffer)}));
import {EditorState} from '@codemirror/state';
import {history,undoDepth} from '@codemirror/commands';
import {WorkspaceChangeHistory} from './ChangePlan';
import {captureResourceRoster,captureResourceInputs,prepareResourceSave,prepareResourceRestore,publishResourceRestore,RetainedResourceTabs,retireCopiedBuffer,resourceLayout,routeResourceEditors,adoptResourceReceipt,captureResourceEditors,type ResourceEditorPatch,type ResourceReceipt,type ResourceSaved} from './ResourceEditors';
// Contract: .hickory-source/frontends/contract.md (resource namespace preservation).
const source=(identity:string,text=identity)=>({identity,text});
describe('resource editor identity routing',()=>{
 it('captures distinct named and unaddressed retained owners without filename or prototype aliasing',()=>{
  const buffers=new Buffers(()=>{});buffers.workspace='/owned';buffers.workspaceId='a'.repeat(32);
  const live=new Buffer({path:'__proto__',name:'live',dir:false},'independent named text','',()=>{},false,buffers.workspace),deleted=new Buffer({path:'__proto__',name:'deleted',dir:false},'retained deleted text','',()=>{},false,buffers.workspace),loose=new Buffer({path:'Untitled-other',name:'loose',dir:false},'unnamed notes','',()=>{});deleted.resourceDeleted=true;loose.isUntitledDraft=true;buffers.values.set('__proto__',live);buffers.values.set('Untitled-other',loose);buffers.retainedResources.set('deleted',deleted);buffers.retainedResources.set('duplicate-handle',deleted);
  const capture=captureResourceRoster(buffers),again=captureResourceRoster(buffers);expect(capture.captures.length).toBe(3);expect(Object.entries(capture.ownership_sources)).toEqual([['__proto__','independent named text']]);expect(Object.values(capture.unaddressed_sources).sort()).toEqual(['retained deleted text','unnamed notes']);expect(again.unaddressed_sources).toEqual(capture.unaddressed_sources);expect(Object.keys(capture.unaddressed_sources).every(id=>/^[a-f0-9]{32}$/.test(id))).toBe(true);expect(capture.current()).toBe(true);deleted.edit('new deleted text');expect(capture.current()).toBe(false);
 });
 it('retires held resource acquisition when a new retained author, token or file identity appears',async()=>{
  for(const change of [(buffers:Buffers)=>buffers.retainedResources.set('new',new Buffer({path:'deleted.md',name:'deleted',dir:false},'retained','',()=>{})),(_buffers:Buffers,buffer:Buffer)=>{buffer.saveGuard='new-token';},(_buffers:Buffers,buffer:Buffer)=>{buffer.file.doc_id='different-document';}]){
   const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'a.py',name:'a.py',dir:false},'before','base',()=>{});buffer.saveGuard='loaded';buffers.values.set('a.py',buffer);let resolve!:(value:unknown)=>void,body:any;const load=<T>(_path:string,value:unknown)=>{body=value;return new Promise<T>(done=>{resolve=done as (value:unknown)=>void;});};const pending=captureResourceInputs(buffers,['a.py'],'capture',{changes:{}},load,()=>true,async()=> 'b'.repeat(64));await Promise.resolve();await Promise.resolve();change(buffers,buffer);resolve({context:'capture',workspace:'scope',source_files_modified:false,refusals:{},saved_guards:{},sources:{},saved_sources:{}});await expect(pending).rejects.toThrow('changed');expect(body.unaddressed_sources).toEqual({});expect(buffer.text).toBe('before');
  }
 });

 it('admits only the explicitly acquired target handle while preserving every original owner',()=>{const buffers=new Buffers(()=>{}),source=new Buffer({path:'a.py',name:'a.py',dir:false},'before','base',()=>{});buffers.values.set('a.py',source);const roster=captureResourceRoster(buffers),target=new Buffer({path:'b.py',name:'b.py',dir:false},'target','base',()=>{});buffers.values.set('b.py',target);expect(roster.current()).toBe(false);expect(roster.current(new Map([['b.py',target]]))).toBe(true);buffers.retainedResources.set('late',new Buffer({path:'Untitled-late',name:'notes',dir:false},'new construction','',()=>{}));expect(roster.current(new Map([['b.py',target]]))).toBe(false);});
 it('requires the mandatory physical guard map and forwards only originally loaded identities',async()=>{for(const guards of [undefined,null,[],{}, {'a.py':''}]){const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'a.py',name:'a.py',dir:false},'before','hash',()=>{});buffer.saveGuard='original-loaded';buffers.values.set('a.py',buffer);let body:any;const load=async<T>(_path:string,value:unknown)=>{body=value;return {context:'capture',workspace:'workspace',source_files_modified:false,refusals:{},sources:{'a.py':{kind:'file',identity:'source',text:'before'}},saved_sources:{},saved_guards:guards} as T;};await expect(captureResourceInputs(buffers,['a.py'],'capture',{changes:{}},load,()=>true,async()=> 'a'.repeat(64))).rejects.toThrow('acquisition');expect(body.drafts['a.py'].save_guard).toBe('original-loaded');expect(body.saved_guards['a.py']).toBe('original-loaded');expect(buffer.saveGuard).toBe('original-loaded');}});
 it('keeps historical complete outcomes readable but refuses v1, missing or stale adoption authority',()=>{const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'a.py',name:'a.py',dir:false},'a','hash',()=>{});buffers.values.set('a.py',buffer);const patches=[{path:'a.py',before:source('a'),after:null}],expected={request:'request',workspace:'workspace',context:'context',saved:new Map<string,ResourceSaved>([['a.py',{kind:'missing'}]])};const base={version:2,adoption_current:true,request_id:'request',workspace:'workspace',context:'context',outcome:{complete:true,recovery:'retained-history',applied:['a.py']},saved_sources:{'a.py':{kind:'missing'}},source_files:[]};for(const changed of [{version:1},{adoption_current:false,adoption_reason:'Same-byte independent source now owns the path.'},{adoption_current:undefined}]){const receipt={...base,...changed} as ResourceReceipt;expect(()=>adoptResourceReceipt(buffers,patches,receipt,expected,captureResourceEditors(buffers,patches))).toThrow();expect(receipt.outcome.complete).toBe(true);expect(buffers.values.get('a.py')).toBe(buffer);expect(buffers.retainedResources.size).toBe(0);}});
 it('captures prototype-looking source names literally and refuses mismatching restored source before publication',async()=>{
  const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'__proto__',name:'__proto__',dir:false},'base','',()=>{});buffer.saveGuard='loaded';buffer.edit('dirty');buffers.values.set('__proto__',buffer);let context='',reads=0;
  const record={id:'a'.repeat(32),workspace:'scope',fingerprint:'b'.repeat(64),paths:[{path:'__proto__',saved_after:{kind:'file'}}]};
  const load=async<T>(route:string,body:unknown):Promise<T>=>{reads++;const request=body as Record<string,any>;
   if(route.endsWith('resource-inputs')){context=request.context;expect(Object.hasOwn(request.drafts,'__proto__')).toBe(true);expect(Object.getPrototypeOf(request.drafts)).toBeNull();expect(request.drafts['__proto__'].text).toBe('dirty');return {workspace:'scope',context,source_files_modified:false,refusals:{},saved_guards:Object.fromEntries([['__proto__','loaded']]),sources:Object.fromEntries([['__proto__',{kind:'file',identity:'source',text:'dirty'}]]),saved_sources:{}} as T;}
   expect(route).toBe('/api/changes/prepare-restoration');return {version:1,restoration:{workspace:'scope',context,recovery_fingerprint:record.fingerprint,saved_before:{},saved_after:{},plan:{version:1,context,title:'Refuse stale restoration',patches:[{path:'__proto__',before:{kind:'file',identity:'source',text:'base'},after:{kind:'file',identity:'source',text:'older'}}]}},consequences:{version:1,workspace:'scope',context,fingerprint:'c'.repeat(64),protected_targets:[],saved:{entries:[],removed_entries:[],output_paths:[]},draft:{entries:[],removed_entries:[],output_paths:[]}}} as T;
  };
  await expect(prepareResourceRestore(buffers,record,async()=>buffer,load,()=>true,async()=> 'd'.repeat(64))).rejects.toThrow('does not match');expect(reads).toBe(2);expect(buffer.text).toBe('dirty');expect(buffer.base).toBe('base');
 });

 it('previews whole resource publication, rejects expanded namespaces and consumes uncertain requests without retry',async()=>{
  const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'old.txt',name:'old.txt',dir:false},'source','old',()=>{});buffer.saveGuard='loaded';buffers.values.set('old.txt',buffer);
  const saved:ResourceSaved={kind:'file',text:'source',sha256:'a'.repeat(64),readonly:false,executable:false,mode:0o600},edit={documentChanges:[{kind:'rename',oldUri:'hick:///old.txt',newUri:'hick:///new.txt'}]};
  let context='',writes=0;const workspace='scope';
  const load=async<T>(route:string,body:unknown):Promise<T>=>{const request=body as Record<string,any>;context=request.context??context;
   if(route.endsWith('resource-targets'))return {paths:['old.txt','new.txt']} as T;
   if(route.endsWith('resource-inputs'))return {workspace,context,source_files_modified:false,refusals:{},saved_guards:{'old.txt':'loaded'},sources:{'old.txt':{kind:'file',identity:'source',text:'source'},'new.txt':{kind:'missing'}},saved_sources:{'old.txt':saved,'new.txt':{kind:'missing'}}} as T;
   if(route.endsWith('plan-resources'))return {plan:{version:1,context,title:'Move source',patches:[{path:'old.txt',before:{kind:'file',identity:'source',text:'source'},after:{kind:'missing'}},{path:'new.txt',before:{kind:'missing'},after:{kind:'file',identity:'source',text:'source'}}]}} as T;
   if(route.endsWith('resource-consequences'))return {version:1,workspace,context,saved:{entries:[],removed_entries:[],output_paths:[]},draft:{entries:[],removed_entries:[],output_paths:[]},fingerprint:'c'.repeat(64),protected_targets:[],saved_sources:{'old.txt':{kind:'missing'},'new.txt':saved}} as T;
   writes++;throw new Error('Transport result unavailable');
  };
  const prepare=()=>prepareResourceSave(buffers,'Move source',edit,async()=>buffer,load,()=>true,async()=> 'a'.repeat(64));
  const changed=await prepare();const extra=new Buffer({path:'extra.md',name:'extra.md',dir:false},'new ownership','',()=>{});buffers.values.set('extra.md',extra);expect(changed.current()).toBe(false);buffers.values.delete('extra.md');buffer.file.generated_by='owner';expect(changed.current()).toBe(false);delete buffer.file.generated_by;
  const review=await prepare();expect(review.operation).toBe('save');expect(review.saved['new.txt']).toEqual(saved);expect(review.savedBefore['old.txt']).toEqual(saved);
  await expect(publishResourceRestore(buffers,review,load)).rejects.toThrow('not confirmed');await expect(publishResourceRestore(buffers,review,load)).rejects.toThrow('already has publication request');expect(writes).toBe(1);expect(buffers.values.get('old.txt')).toBe(buffer);expect(buffers.values.has('new.txt')).toBe(false);
 });
 it('coordinates draft and resource undo with a bounded history and retires interleaved review tokens',()=>{
  const history=new WorkspaceChangeHistory(),draft={title:'Source',patches:[]};history.record({kind:'draft',plan:draft});history.record({kind:'resource',recovery:'record'});
  const old=history.preview('undo')!;history.record({kind:'draft',plan:draft});expect(history.current(old)).toBe(false);expect(()=>history.commit(old,{kind:'resource',recovery:'wrong'})).toThrow('history changed');
  const latest=history.preview('undo')!;history.commit(latest,{kind:'draft',plan:draft});expect(history.preview('redo')?.entry.kind).toBe('draft');const resource=history.preview('undo')!;expect(resource.entry).toEqual({kind:'resource',recovery:'record'});history.commit(resource,{kind:'resource',recovery:'inverse'});expect(history.preview('redo')?.entry).toEqual({kind:'resource',recovery:'inverse'});
  for(let i=0;i<40;i++)history.record({kind:'resource',recovery:String(i)});let count=0;for(let next=history.preview('undo');next;next=history.preview('undo')){history.commit(next,{kind:'resource',recovery:'redo'});count++;}expect(count).toBe(32);
 });

 it('prepares and publishes explicit restoration with separate baselines, identity-bound receipts and no stale retry',async()=>{
  const buffers=new Buffers(()=>{}),buffer=new Buffer({path:'b.txt',name:'b.txt',dir:false},'dirty','old',()=>{});buffer.saveGuard='loaded';buffers.values.set('b.txt',buffer);
  const saved:ResourceSaved={kind:'file',text:'saved',sha256:'a'.repeat(64),readonly:false,executable:false,mode:null};
  const calls:string[]=[];let context='',workspace='workspace';
  const load=async<T>(route:string,body:unknown):Promise<T>=>{calls.push(route);const request=body as Record<string,any>;
   if(route.endsWith('resource-inputs')){context=request.context;expect(request.drafts['b.txt'].text).toBe('dirty');return {workspace,context,source_files_modified:false,refusals:{},saved_guards:{'b.txt':'loaded'},sources:{'b.txt':{kind:'file',identity:'source',text:'dirty'},'a.txt':{kind:'missing'}},saved_sources:{}} as T;}
   if(route.endsWith('prepare-restoration'))return {version:1,restoration:{workspace,context,recovery_fingerprint:'c'.repeat(64),saved_before:{'b.txt':{...saved,text:'dirty'},'a.txt':{kind:'missing'}},saved_after:{'a.txt':saved,'b.txt':{kind:'missing'}},plan:{version:1,context,title:'Restore',patches:[{path:'b.txt',before:{kind:'file',identity:'source',text:'dirty'},after:{kind:'missing'}},{path:'a.txt',before:{kind:'missing'},after:{kind:'file',identity:'source',text:'dirty'}}]}},consequences:{version:1,workspace,context,saved:{entries:[],removed_entries:[],output_paths:[]},draft:{entries:[],removed_entries:[],output_paths:[]},fingerprint:'d'.repeat(64),protected_targets:[]}} as T;
   expect(route).toBe('/api/changes/restore-resources');return {version:2,adoption_current:true,request_id:request.request_id,workspace,context,outcome:{complete:true,recovery:'.hick-cache/resource-changes/'+ 'e'.repeat(32)+'/manifest.json',applied:['a.txt','b.txt']},saved_sources:{'a.txt':saved,'b.txt':{kind:'missing'}},source_files:[{file:{path:'a.txt',name:'a.txt',dir:false},saved,hash:'new'}]} as T;
  };
  const record={id:'b'.repeat(32),workspace,fingerprint:'c'.repeat(64),paths:[{path:'a.txt',saved_after:{kind:'missing'}},{path:'b.txt',saved_after:{kind:'file'}}]};
  const prepare=()=>prepareResourceRestore(buffers,record,async()=>buffer,load,()=>true,async()=> 'a'.repeat(64));
  const stale=await prepare();buffer.edit('independent');await expect(publishResourceRestore(buffers,stale,load)).rejects.toThrow('Source changed');expect(calls.filter(route=>route.endsWith('restore-resources'))).toHaveLength(0);
  buffer.edit('dirty');const review=await prepare(),result=await publishResourceRestore(buffers,review,load);expect(result.recovery).toBe('e'.repeat(32));expect(buffers.values.get('a.txt')).toBe(buffer);expect(buffers.values.has('b.txt')).toBe(false);expect(buffer.text).toBe('dirty');expect(buffer.base).toBe('saved');expect(buffer.dirty).toBe(true);
  await expect(publishResourceRestore(buffers,review,load)).rejects.toThrow('already has publication request');expect(calls.filter(route=>route.endsWith('restore-resources'))).toHaveLength(1);
 });
 it('routes live tab swaps without losing pane identity and explicitly restores deleted placement',()=>{
  const a=new Buffer({path:'a',name:'a',dir:false},'a','',()=>{}),b=new Buffer({path:'b',name:'b',dir:false},'b','',()=>{}),placements=new Map<Buffer,{group:string;index:number;active:boolean}>();
  const layout={id:'pane',tabs:['a','b'],active:'a'},route=(editor:Buffer,from:string|null,to:string|null)=>({identity:editor.text,from,to,before:editor.text,after:editor.text,editor});
  const swapped=resourceLayout(layout,[route(a,'a','b'),route(b,'b','a')],placements);expect(swapped).toEqual({id:'pane',tabs:['b','a'],active:'b'});expect(layout.tabs).toEqual(['a','b']);
  const deleted=resourceLayout(swapped,[route(a,'b',null)],placements);expect(deleted).toEqual({id:'pane',tabs:['a'],active:'a'});expect(placements.get(a)).toEqual({group:'pane',index:0,active:true});
  const restored=resourceLayout(deleted,[route(a,null,'restored')],placements);expect(restored).toEqual({id:'pane',tabs:['restored','a'],active:'restored'});expect(placements.size).toBe(0);
 });

 it('adopts a confirmed move, deletion and restoration into live buffers with separate saved baselines',async()=>{
  const buffers=new Buffers(()=>{}),a=new Buffer({path:'old.py',name:'old.py',dir:false},'saved','old-hash',()=>{});a.edit('dirty');buffers.values.set('old.py',a);
  const state=EditorState.create({doc:'saved',extensions:[history()]}).update({changes:{from:0,to:5,insert:'dirty'},selection:{anchor:2}}).state;
  const view=new EditorView({state});views.set(a,{view});view.scrollDOM.scrollTop=73;
  const retained=buffers.retainedResources,saved:ResourceSaved={kind:'file',text:'dirty',sha256:'a'.repeat(64),readonly:false,executable:false,mode:null};
  function adopt(patches:ResourceEditorPatch[],baseline:ResourceSaved,restoring=new Map<string,Buffer>()){
   const expected={request:'request',workspace:'workspace',context:'context',saved:new Map(patches.map(p=>[p.path,p.after?baseline:{kind:'missing'} as ResourceSaved]))};
   const receipt:ResourceReceipt={version:2,adoption_current:true,request_id:'request',workspace:'workspace',context:'context',outcome:{complete:true,recovery:'recovery',applied:patches.map(p=>p.path)},saved_sources:Object.fromEntries(expected.saved),source_files:patches.filter(p=>p.after).map(p=>({file:{path:p.path,name:p.path,dir:false},saved:baseline,hash:'fresh'}))};
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
  const receipt:ResourceReceipt={version:2,adoption_current:true,request_id:'r',workspace:'w',context:'c',outcome:{complete:false,recovery:'recovery',applied:[]},saved_sources:null,source_files:[]},retained=buffers.retainedResources;
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
 it('retains active deleted editor tabs, restores the same route and retires only an exact copied handle',()=>{
  const buffers=new Buffers(()=>{}),original=new Buffer({path:'old.py',name:'old.py',dir:false},'retained','old',()=>{}),independent=new Buffer({path:'old.py',name:'old.py',dir:false},'independent','new',()=>{}),tabs=new RetainedResourceTabs();
  const layout={id:'one',tabs:['old.py'],active:'old.py'};const deletion={identity:'source',from:'old.py',to:null,editor:original,before:'retained',after:null};original.resourceDeleted=true;buffers.retainedResources.set('source',original);buffers.values.set('old.py',independent);
  const held=resourceLayout(layout,tabs.routes([deletion],buffer=>buffer===original),new Map());const alias=(held as typeof layout).active!;expect(alias.startsWith('\0')).toBe(true);expect(tabs.buffers.get(alias)).toBe(original);expect(buffers.values.get('old.py')).toBe(independent);
  const restored=resourceLayout(held,tabs.routes([{...deletion,from:null,to:'restored.py',after:'retained'}],()=>false),new Map());expect((restored as typeof layout).tabs).toEqual(['restored.py']);expect(tabs.buffers.size).toBe(0);
  expect(retireCopiedBuffer(buffers,original,'stale')).toBe(false);expect(buffers.retainedResources.get('source')).toBe(original);tabs.noteCopy(original,original.text);expect(tabs.copied(original)).toBe(true);original.edit('new unsaved');expect(tabs.copied(original)).toBe(false);
  expect(retireCopiedBuffer(buffers,original,original.text)).toBe(true);expect(buffers.retainedResources.size).toBe(0);expect(buffers.values.get('old.py')).toBe(independent);
 });
});
