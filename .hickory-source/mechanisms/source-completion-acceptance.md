<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Accept a complete source completion through the existing popup

For maintainers changing source completion behavior independently of popup and editor rendering.

## Observe asynchronous acceptance independently of a popup

A popup acceptance can begin a source-bound resolver job. Keep its latest identity, cancellation signal and observed outcome separate from whether the native popup is open. Read returns a copy; a superseded job cannot update the current observation. Escape or the invoking agent signal cancels pending acceptance. A waiter resolves only when the acceptance is applied or unavailable, so an agent does not treat popup acceptance as a committed edit. Saved and verified remain false for this draft-only operation.

<ui:copy id="completion-acceptance-observation">
import type {EditorView} from '@codemirror/view';
export interface CompletionAcceptance {job:string;phase:'resolving'|'applying'|'applied'|'unavailable';message:string;applied:boolean;source_changed:boolean;saved:false;verified:false}
interface PendingAcceptance {value:CompletionAcceptance;cancelled:boolean;promise:Promise<CompletionAcceptance>;update:(change:Partial<CompletionAcceptance>)=>void;cancel:(message:string)=>boolean}
const latest=new WeakMap<EditorView,PendingAcceptance>(),acceptingSignals=new WeakMap<EditorView,AbortSignal>();
export function withCompletionAcceptanceSignal<T>(view:EditorView,signal:AbortSignal,accept:()=>T):T {acceptingSignals.set(view,signal);try{return accept();}finally{acceptingSignals.delete(view);}}
export function readCompletionAcceptance(view:EditorView):CompletionAcceptance|null {const value=latest.get(view)?.value;return value?{...value}:null;}
export function waitCompletionAcceptance(view:EditorView,job:string):Promise<CompletionAcceptance> {const current=latest.get(view);return current?.value.job===job?current.promise:Promise.reject(new Error('Completion acceptance was superseded.'));}
export function cancelCompletionAcceptance(view:EditorView):boolean {return latest.get(view)?.cancel('Completion cancelled.')??false;}
export function beginCompletionAcceptance(view:EditorView,notice:(message:string)=>void=()=>{}) {
 const previous=latest.get(view);if(previous&&previous.value.phase==='resolving'){previous.cancelled=true;previous.update({phase:'unavailable',message:'Superseded by a newer completion.'});}
 const signal=acceptingSignals.get(view),value:CompletionAcceptance={job:crypto.randomUUID(),phase:'resolving',message:'Resolving completion…',applied:false,source_changed:false,saved:false,verified:false};
 let settle!:(value:CompletionAcceptance)=>void;
 const promise=new Promise<CompletionAcceptance>(resolve=>{settle=resolve;});
 const onAbort=()=>{entry.cancel('Completion cancelled.');};
 const entry:PendingAcceptance={value,cancelled:false,promise,update:change=>{Object.assign(value,change);if(value.phase==='applied'||value.phase==='unavailable'){signal?.removeEventListener('abort',onAbort);settle({...value});}},cancel:message=>{if(value.phase!=='resolving')return false;entry.cancelled=true;entry.update({phase:'unavailable',message});if(latest.get(view)===entry)notice(message);return true;}};
 latest.set(view,entry);signal?.addEventListener('abort',onAbort,{once:true});if(signal?.aborted)onAbort();
 return {job:value.job,cancelled:()=>entry.cancelled,current:()=>latest.get(view)===entry,update:(change:Partial<CompletionAcceptance>)=>{if(latest.get(view)===entry||change.phase==='applied'||change.phase==='unavailable')entry.update(change);}};
}
</ui:copy>

<ui:file path="apps/web/src/lsp/completionAcceptance.ts"><ui:paste select="#completion-acceptance-observation"/></ui:file>

## Prepare and commit one complete browser transaction

An optional source-editor adapter retains the actual native choice and query selection, resolves through its producing client and rechecks attachment, source, selection, provider, permissions, saving and composition before planning. This bridge currently requires one caret; multiple-carets are retained and refused until independently captured proposals can be combined. The whole source/import plan passes the shared input guard and transaction preparation before one synchronous commit with isolated undo. Guarded no-op selection is supported; a late or replaced job cannot write source or overwrite a newer status.

<ui:copy id="completion-acceptance-browser">
import {isolateHistory} from '@codemirror/commands';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {closeCompletion,pickedCompletion,type Completion,type CompletionContext} from '@codemirror/autocomplete';
import type {CompletionItem} from '../lsp/client';
import {lspCompletion} from '../lsp/completion';
import {beginCompletionAcceptance} from '../lsp/completionAcceptance';
import {prepareEditorChanges} from '../editor/textInput';
import {planCompletion} from './CompletionPlan';
export interface SourceCompletionAdapter {
 uri:string;resolve:(item:CompletionItem)=>Promise<CompletionItem>;current:()=>boolean;writable:()=>boolean;notice:(text:string)=>void;
}
const querySnapshots=new WeakMap<EditorState,{source:string;selection:EditorState['selection']}>();
export function sourceCompletion(item:CompletionItem,context:CompletionContext,adapter:SourceCompletionAdapter):Completion {
 let snapshot=querySnapshots.get(context.state);
 if(!snapshot){snapshot={source:context.state.doc.toString(),selection:context.state.selection};querySnapshots.set(context.state,snapshot);}
 const {source,selection}=snapshot,original=structuredClone(item);
 const label=typeof item.filterText==='string'?item.filterText:item.label;
 return {...lspCompletion(item),label,...(label!==item.label?{displayLabel:item.label}:{}),apply:(view,completion,from,to)=>{closeCompletion(view);void acceptSourceCompletion(view,original,source,selection,from,to,adapter,completion);}};
}
export async function acceptSourceCompletion(view:EditorView,item:CompletionItem,source:string,selection:EditorState['selection'],from:number,to:number,adapter:SourceCompletionAdapter,completion?:Completion):Promise<void> {
 const request=beginCompletionAcceptance(view,adapter.notice);
 let expectedAfter:string|undefined;
 const valid=()=>!request.cancelled()&&request.current()&&adapter.current()&&view.dom.isConnected&&view.state.doc.toString()===source&&view.state.selection.eq(selection)&&adapter.writable()&&!view.composing&&!view.state.readOnly&&view.state.facet(EditorView.editable);
 const report=(text:string)=>{if(request.current())adapter.notice(text);};
 try {
  if(request.cancelled())return;
  if(selection.ranges.length!==1)throw new Error('Choose one caret before accepting this source completion.');
  if(view.composing)throw new Error('Finish composition before accepting this completion.');
  if(!valid())throw new Error('Completion is out of date or source is unavailable. Request completions again.');
  report('Resolving completion…');const resolved=await adapter.resolve(structuredClone(item));
  if(!request.current()||request.cancelled())return;
  if(view.composing)throw new Error('Finish composition, then request completions again.');
  if(!valid())throw new Error('Source, cursor or provider changed. Request completions again.');
  const draft=planCompletion(adapter.uri,resolved,{text:source,writable:adapter.writable()},from,to);
  const patch=draft.plan.patches[0],after=patch?.after??source,changes=(patch?.edits??[]).map(edit=>({from:edit.from,to:edit.to,insert:edit.text}));
  const input=prepareEditorChanges(source,changes);if('reason' in input)throw new Error(input.reason);
  if(!valid())throw new Error('Source, cursor or provider changed. Request completions again.');
  const transaction=view.state.update({changes,selection:{anchor:draft.selection.from,head:draft.selection.to},annotations:[isolateHistory.of('full'),...(completion?[pickedCompletion.of(completion)]:[])],userEvent:'input.complete',filter:false});
  if(transaction.newDoc.toString()!==after)throw new Error('Editor could not prepare the complete completion. Request again.');
  expectedAfter=after;request.update({phase:'applying'});view.update([transaction]);
  const applied=view.state.doc.toString()===after;
  request.update({phase:applied?'applied':'unavailable',applied,source_changed:applied&&transaction.docChanged,message:applied?'Completion applied to draft.':'Editor could not reflect the complete completion. Inspect the current draft.'});
  report(applied?(transaction.docChanged?'Completion applied · draft':'Completion selected'):'Editor could not reflect the complete completion. Inspect the current draft.');
 }catch(error){if((request.current()||expectedAfter!==undefined)&&!request.cancelled()){const applied=expectedAfter!==undefined&&view.state.doc.toString()===expectedAfter,message=error instanceof Error?error.message:String(error);request.update({phase:'unavailable',message,applied,source_changed:applied&&source!==expectedAfter});report(message);}}
}
</ui:copy>

<ui:file path="apps/web/src/simple/SourceCompletion.ts"><ui:paste select="#completion-acceptance-browser"/></ui:file>

## Check browser intent and atomic draft application

Held resolvers keep the source readable and unchanged until complete. Apply primary/import edits with one undo, map the caret, reject source changes and composition, retire older choices and settle cancellation before a late reply.

<ui:copy id="source-completion-browser-witness">
import {expect,it} from 'vitest';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {history,undo,redo} from '@codemirror/commands';
import {acceptSourceCompletion,type SourceCompletionAdapter} from './SourceCompletion';
import {readCompletionAcceptance,waitCompletionAcceptance,cancelCompletionAcceptance,withCompletionAcceptanceSignal} from '../lsp/completionAcceptance';
import type {CompletionItem} from '../lsp/client';
function held(){let resolve!:(value:CompletionItem)=>void;const promise=new Promise<CompletionItem>(done=>{resolve=done;});return {promise,resolve};}
function fixture(){const parent=document.body.appendChild(document.createElement('div'));const view=new EditorView({state:EditorState.create({doc:'va\n',selection:{anchor:2},extensions:history()}),parent});return {view,close:()=>{view.destroy();parent.remove();}};}
const raw={label:'value',data:{hickory_completion:'held'}};
function resolved(text='value'):CompletionItem {return {...raw,textEdit:{range:{start:{line:0,character:0},end:{line:0,character:2}},newText:text},additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'# import\n'}]};}
it('waits for the whole resolved import edit, commits one undo and maps the caret',async()=>{
 const {view,close}=fixture(),pending=held(),notices:string[]=[];
 try{const work=acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,{uri:'hick:///a.py',resolve:()=>pending.promise,current:()=>true,writable:()=>true,notice:value=>notices.push(value)});
  const before=readCompletionAcceptance(view)!;const finished=waitCompletionAcceptance(view,before.job);expect(before.phase).toBe('resolving');expect(view.state.doc.toString()).toBe('va\n');pending.resolve(resolved());await work;
  expect((await finished).phase).toBe('applied');expect(view.state.doc.toString()).toBe('# import\nvalue\n');expect(view.state.selection.main.head).toBe(14);expect(notices.at(-1)).toContain('draft');expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('va\n');expect(redo(view)).toBe(true);expect(view.state.doc.toString()).toBe('# import\nvalue\n');
 }finally{close();}
});
it('a newer choice owns the result and late older replies cannot overwrite it',async()=>{
 const {view,close}=fixture(),first=held(),second=held(),notices:string[]=[];let calls=0;
 const adapter:SourceCompletionAdapter={uri:'hick:///a.py',resolve:()=>++calls===1?first.promise:second.promise,current:()=>true,writable:()=>true,notice:value=>notices.push(value)};
 try{const selection=view.state.selection,a=acceptSourceCompletion(view,raw,'va\n',selection,0,2,adapter),old=waitCompletionAcceptance(view,readCompletionAcceptance(view)!.job),b=acceptSourceCompletion(view,raw,'va\n',selection,0,2,adapter);
  expect((await old).phase).toBe('unavailable');second.resolve(resolved('newer'));await b;const notice=notices.at(-1);first.resolve(resolved('older'));await a;expect(view.state.doc.toString()).toBe('# import\nnewer\n');expect(readCompletionAcceptance(view)?.phase).toBe('applied');expect(notices.at(-1)).toBe(notice);
 }finally{close();}
});
it('source edits and composition refuse both the primary and import portions',async()=>{
 const {view,close}=fixture(),pending=held();let calls=0;
 const adapter:SourceCompletionAdapter={uri:'hick:///a.py',resolve:()=>{calls++;return pending.promise;},current:()=>true,writable:()=>true,notice:()=>{}};
 try{const work=acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,adapter);view.dispatch({changes:{from:0,insert:'# user\n'}});pending.resolve(resolved());await work;expect(view.state.doc.toString()).toBe('# user\nva\n');expect(readCompletionAcceptance(view)?.phase).toBe('unavailable');
  Object.defineProperty(view,'composing',{value:true,configurable:true});const before=view.state.doc.toString();await acceptSourceCompletion(view,raw,before,view.state.selection,0,2,adapter);expect(calls).toBe(1);expect(view.state.doc.toString()).toBe(before);
 }finally{close();}
});
it('Escape cancellation and an agent abort settle observers before the late provider reply',async()=>{
 for(const agent of[false,true]){const {view,close}=fixture(),pending=held(),signal=new AbortController();try{
  const work=withCompletionAcceptanceSignal(view,signal.signal,()=>acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,{uri:'hick:///a.py',resolve:()=>pending.promise,current:()=>true,writable:()=>true,notice:()=>{}}));
  const finished=waitCompletionAcceptance(view,readCompletionAcceptance(view)!.job);if(agent)signal.abort();else expect(cancelCompletionAcceptance(view)).toBe(true);expect((await finished).phase).toBe('unavailable');pending.resolve(resolved());await work;expect(view.state.doc.toString()).toBe('va\n');
 }finally{close();}}
});
</ui:copy>

<ui:file path="apps/web/src/simple/SourceCompletion.test.ts"><ui:paste select="#source-completion-browser-witness"/></ui:file>

## Drive the real popup through the agent adapter

The agent opens and selects an actual CodeMirror popup choice. Its acceptance must remain pending while resolution is held and report completion only after the whole source/import draft is committed. No fabricated apply range or second completion query substitutes for the popup.

<ui:copy id="source-completion-agent-witness">
import {act,cleanup,render,waitFor} from '@testing-library/react';
import {expect,it} from 'vitest';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {completionStatus} from '@codemirror/autocomplete';
import {completions} from '../lsp/completion';
import {readCompletionAcceptance} from '../lsp/completionAcceptance';
import {sourceCompletion} from './SourceCompletion';
import type {CompletionItem} from '../lsp/client';
import {UiActions} from '../lib/uiActions';
import {UiActionContext} from '../lib/uiActionBridge';
import {useEditorActions} from '../editor/uiEditorActions';
it('agent native-popup acceptance completes only after resolution and the whole draft commit',async()=>{
 const registry=new UiActions({workspace:'p',window:'w'},()=>true,async()=>{}),parent=document.body.appendChild(document.createElement('div'));
 let resolve!:(item:CompletionItem)=>void;const held=new Promise<CompletionItem>(done=>{resolve=done;});
 const item={label:'value',data:{hickory_completion:'held'}};
 const view=new EditorView({state:EditorState.create({doc:'va\n',selection:{anchor:2},extensions:completions({lsp:{client:{completion:async()=>[item]} as never,uri:'hick:///a.py',positionAt:offset=>({line:0,character:offset})},lspItem:(item,context)=>sourceCompletion(item,context,{uri:'hick:///a.py',resolve:()=>held,current:()=>true,writable:()=>true,notice:()=>{}})})}),parent});
 function Harness(){useEditorActions(view,'source');return null;}
 try{
  render(<UiActionContext.Provider value={registry}><Harness/></UiActionContext.Provider>);
  const actions=()=>registry.groups().groups.flatMap(group=>registry.observe(group).actions);let id=0;
  const run=async(label:string,args={})=>{let reply:unknown;await act(async()=>{reply=await registry.act(registry.scope,{id:'python:turn',kind:'agent'},{handle:actions().find(item=>item.label===label)!.handle,arguments:args,request_id:String(++id)});});return reply as {outcome:{value:{snapshot:string}}};};
  await waitFor(()=>expect(actions().some(item=>item.label==='Open editor completions')).toBe(true));await run('Open editor completions');await waitFor(()=>expect(completionStatus(view.state)).toBe('active'));
  const observed=await run('Observe editor completion popup'),selected=await run('Select editor completion',{snapshot:observed.outcome.value.snapshot,index:0});await new Promise(done=>setTimeout(done,90));
  let completed=false;
  const accepting=registry.act(registry.scope,{id:'python:turn',kind:'agent'},{handle:actions().find(item=>item.label==='Accept selected editor completion')!.handle,arguments:{snapshot:selected.outcome.value.snapshot},request_id:String(++id)}).then(value=>{completed=true;return value;});
  await waitFor(()=>expect(readCompletionAcceptance(view)?.phase).toBe('resolving'));expect(completed).toBe(false);expect(view.state.doc.toString()).toBe('va\n');
  let reply:unknown;await act(async()=>{resolve({...item,textEdit:{range:{start:{line:0,character:0},end:{line:0,character:2}},newText:'value'},additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'# import\n'}]});reply=await accepting;});
  expect(reply).toMatchObject({status:'result',outcome:{status:'completed',value:{accepted:true,source_changed:true,saved:false,verified:false,acceptance:{phase:'applied'}}}});expect(view.state.doc.toString()).toBe('# import\nvalue\n');expect(view.state.selection.main.head).toBe(14);
 }finally{cleanup();view.destroy();parent.remove();}
});
</ui:copy>

<ui:file path="apps/web/src/simple/SourceCompletion.agent.test.tsx"><ui:paste select="#source-completion-agent-witness"/></ui:file>

## Check native batching, undo and obsolete intents

The actual AppKit editor applies a compound completion with the intended caret and one undo/redo group. Independent controller witnesses hold competing replies and dismiss a pending choice; older work must not write source or replace the newer notice. These checks do not establish native popover pointer or IME acceptance.

<ui:copy id="source-completion-apple-witness">
import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class CompletionAcceptanceTests:XCTestCase {
    func settle(_ condition:@escaping ()->Bool)async{let end=Date().addingTimeInterval(2);while !condition()&&Date()<end{try? await Task.sleep(for:.milliseconds(1))}}
    func edit(_ line:Int,_ from:Int,_ to:Int,_ text:String)->[String:Any]{["range":["start":["line":line,"character":from],"end":["line":line,"character":to]],"newText":text]}
    func testActualMacEditorAcceptsImportsAndCaretInOneUndo()async throws {
        #if os(macOS)
        let source="header\nva\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base")),service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll
        defer{service.close();window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:9,length:0));buffer.selection=editor.text.selectedRange();let original:[String:Any]=["label":"value","data":["hickory_completion":"held"]]
        let item=NativeCompletion(id:1,label:"value",detail:"",range:NSRange(location:7,length:2),insert:"value",source:source,selection:buffer.selection,original:original)
        let accepted=await service.accept(item,resolution:{_ in ["label":"value","textEdit":self.edit(1,0,2,"value"),"additionalTextEdits":[self.edit(0,0,6,"longheader")]]})
        XCTAssertTrue(accepted);XCTAssertEqual(buffer.text,"longheader\nvalue\n");XCTAssertEqual(editor.text.selectedRange().location,16);XCTAssertEqual(buffer.base,source);XCTAssertEqual(buffer.hash,"base")
        buffer.saving=true;editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);buffer.saving=false
        editor.text.undoManager?.redo();XCTAssertEqual(buffer.text,"longheader\nvalue\n");XCTAssertEqual(editor.text.selectedRange().location,16)
        buffer.text="external";editor.text.string="external";editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,"external");XCTAssertEqual(editor.text.string,"external");XCTAssertTrue(service.completionNotice.contains("history retired"));XCTAssertFalse(editor.text.undoManager?.canUndo ?? false)
        #endif
    }
    func testNewerChoiceAndDismissalRetireHeldResolutionWithoutOverwritingSource()async throws {
        let source="va\n",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"va\n",hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service;buffer.selection=NSRange(location:2,length:0);defer{service.close()}
        var commits=0
        buffer.applyCompletionDraft={draft,initial in
            let edits=(draft.plan.patches.first?.edits ?? []).map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},before=draft.before,selected=NSRange(location:draft.selection.from,length:draft.selection.to-draft.selection.from)
            guard exactUTF8(buffer.text,before),buffer.selection==initial else{return false};commits+=1
            let text=NSMutableString(string:before);for edit in edits.sorted(by:{$0.range.location>$1.range.location}){text.replaceCharacters(in:edit.range,with:edit.text)};buffer.text=text as String;buffer.selection=selected;return true
        }
        var held:[String:CheckedContinuation<[String:Any],Error>]=[:]
        let resolve:([String:Any]) async throws->[String:Any]={item in try await withCheckedThrowingContinuation{held[item["label"] as! String]=$0}}
        func item(_ label:String)->NativeCompletion{NativeCompletion(id:1,label:label,detail:"",range:NSRange(location:0,length:2),insert:label,source:source,selection:NSRange(location:2,length:0),original:["label":label,"data":["hickory_completion":label]])}
        func result(_ label:String)->[String:Any]{["label":label,"textEdit":edit(0,0,2,label),"additionalTextEdits":[edit(0,0,0,"# import\n")]]}
        let first=Task{await service.accept(item("old"),resolution:resolve)};await settle{held["old"] != nil}
        let second=Task{await service.accept(item("new"),resolution:resolve)};await settle{held["new"] != nil}
        try XCTUnwrap(held["new"]).resume(returning:result("new"));let secondAccepted=await second.value;XCTAssertTrue(secondAccepted);let notice=service.completionNotice
        try XCTUnwrap(held["old"]).resume(returning:result("old"));let firstAccepted=await first.value;XCTAssertFalse(firstAccepted);XCTAssertEqual(commits,1);XCTAssertEqual(buffer.text,"# import\nnew\n");XCTAssertEqual(service.completionNotice,notice)
        buffer.text=source;buffer.selection=NSRange(location:2,length:0)
        let pending=Task{await service.accept(item("dismissed"),resolution:resolve)};await settle{held["dismissed"] != nil};service.cancelCompletion()
        try XCTUnwrap(held["dismissed"]).resume(returning:result("dismissed"));let pendingAccepted=await pending.value;XCTAssertFalse(pendingAccepted);XCTAssertEqual(buffer.text,source);XCTAssertEqual(commits,1)
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/CompletionAcceptanceTests.swift"><ui:paste select="#source-completion-apple-witness"/></ui:file>

## Check Compose source state and obsolete intents

The real TextFieldState batch applies primary/import edits and selection in one undo/redo operation. Held replies prove latest-choice ownership and dismissal refusal independently of rendering. Device pointer/IME acceptance remains a separate check.

<ui:copy id="source-completion-android-witness">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray
import androidx.compose.ui.text.TextRange
class CompletionAcceptanceTest {
 private fun edit(line:Int,from:Int,to:Int,text:String)=JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",line).put("character",from)).put("end",JSONObject().put("line",line).put("character",to))).put("newText",text)
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun completeImportAndCaretAreOneUndo()=runBlocking {
  val source="header\nva\n";val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(9)}
  try{val item=NativeLanguageCompletion("value","",NativeSourceEdit(7,9,"value"),source,buffer.state.selection,JSONObject().put("label","value").put("data",JSONObject().put("hickory_completion","held")))
   assertTrue(service.accept(item){JSONObject().put("label","value").put("textEdit",edit(1,0,2,"value")).put("additionalTextEdits",JSONArray().put(edit(0,0,6,"longheader")))})
   assertEquals("longheader\nvalue\n",buffer.text);assertEquals(16,buffer.state.selection.start);assertEquals(source,buffer.base);assertEquals("base",buffer.hash)
   buffer.state.undoState.undo();assertEquals(source,buffer.text);buffer.state.undoState.redo();assertEquals("longheader\nvalue\n",buffer.text);assertEquals(16,buffer.state.selection.start)
  }finally{service.close()}
 }
 @Test fun newerChoiceAndDismissalRetireHeldResolution()=runBlocking {
  val source="va\n";val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText(source,"base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(2)}
  val held=mutableMapOf<String,CompletableDeferred<JSONObject>>()
  val resolve:suspend (JSONObject)->JSONObject={item->val promise=CompletableDeferred<JSONObject>();held[item.getString("label")]=promise;promise.await()}
  fun item(label:String)=NativeLanguageCompletion(label,"",NativeSourceEdit(0,2,label),source,TextRange(2),JSONObject().put("label",label).put("data",JSONObject().put("hickory_completion",label)))
  fun result(label:String)=JSONObject().put("label",label).put("textEdit",edit(0,0,2,label)).put("additionalTextEdits",JSONArray().put(edit(0,0,0,"# import\n")))
  suspend fun settle(condition:()->Boolean){withTimeout(2000){while(!condition())yield()}}
  try{val first=async{service.accept(item("old"),resolve)};settle{held["old"]!=null};val second=async{service.accept(item("new"),resolve)};settle{held["new"]!=null}
   held.getValue("new").complete(result("new"));assertTrue(second.await());val notice=service.completionNotice;held.getValue("old").complete(result("old"));assertFalse(first.await());assertEquals("# import\nnew\n",buffer.text);assertEquals(notice,service.completionNotice)
   buffer.state.edit{replace(0,length,source);selection=TextRange(2)};val pending=async{service.accept(item("dismissed"),resolve)};settle{held["dismissed"]!=null};service.cancelCompletion();held.getValue("dismissed").complete(result("dismissed"));assertFalse(pending.await());assertEquals(source,buffer.text)
  }finally{service.close()}
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/CompletionAcceptanceTest.kt"><ui:paste select="#source-completion-android-witness"/></ui:file>

## Resolve through the actual engine and existing popup

A controlled lazy child supplies standards-expanded completions and an import edit. The browser opens the real native popup, chooses its rendered item, checks the complete draft and exact caret, then uses one ordinary undo/redo. Retained editor identity, geometry and unchanged disk distinguish the interaction from a reconstructed editor or a hidden save. This is a protocol/UI witness, not a claim about the child's language analysis.

<ui:copy id="source-completion-browser-engine-witness">
"""Existing completion popup, actual engine and controlled lazy/import provider."""
import importlib.util
import json
import os
from pathlib import Path
import socket
import sys
import tempfile
import time
ROOT=Path(__file__).resolve().parents[1]
def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-source-completion-") as temporary:
        folder=Path(temporary);root=folder/"workspace";root.mkdir();path="a.py";source="value = 1\n";(root/path).write_text(source);events=folder/"events.jsonl";gate=folder/"release"
        (root/".hick-lsp.json").write_text(json.dumps({"servers":{"python":{"command":[sys.executable,str(ROOT/"scripts/controlled-action-provider.py"),"python",str(events),str(gate)]}}}))
        environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path);deadline=time.monotonic()+20
            while True:
                options=driver.act("completion-open",path=path,offset=3)
                if any("python completion" in value for value in options):break
                assert time.monotonic()<deadline,options;time.sleep(.1)
            assert driver.act("editor-size",path=path)==size;assert driver.act("search-source",path=path)==source
            notice=driver.act("completion-choose",title="python completion");assert "Completion applied" in notice,notice
            after="python_completed = 1\n# imported\n";assert driver.act("search-source",path=path)==after;assert driver.act("caret-offset",path=path)==16
            assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size;assert (root/path).read_text()==source
            driver.act("undo",path=path);assert driver.act("search-source",path=path)==source
            driver.act("redo",path=path);assert driver.act("search-source",path=path)==after;assert driver.act("caret-offset",path=path)==16;assert (root/path).read_text()==source
            driver.act("undo",path=path);assert driver.act("search-source",path=path)==source
            options=driver.act("completion-open",path=path,offset=2);assert any("python completion" in value for value in options),options
            driver.act("completion-pick",title="python completion");deadline=time.monotonic()+3
            while True:
                observed=[json.loads(line) for line in events.read_text().splitlines()]
                if any(item.get("slow_completion") for item in observed):break
                assert time.monotonic()<deadline,observed;time.sleep(.01)
            driver.act("completion-cancel",path=path);gate.write_text("release");deadline=time.monotonic()+3
            while True:
                observed=[json.loads(line) for line in events.read_text().splitlines()]
                if any("completion_released" in item for item in observed):break
                assert time.monotonic()<deadline,observed;time.sleep(.01)
            driver.act("completion-drain");assert driver.act("search-source",path=path)==source;assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size;assert (root/path).read_text()==source
            observed=[json.loads(line) for line in events.read_text().splitlines()];assert [item["completion_resolved"] for item in observed if "completion_resolved" in item]==["python","python"],observed
            print("PASS: native completion popup, actual lazy resolution, primary and import edits, precise caret, one undo/redo, Escape refusal of a held late reply, retained editor geometry and unchanged disk")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-source-completion-browser.py"><ui:paste select="#source-completion-browser-engine-witness"/></ui:file>

## Replay a native source draft and its selection as one operation

Cocoa's default undo records each replacement's caret independently. An import applied after the primary completion therefore placed redo's caret at the import. This platform bridge derives forward/inverse edits from the shared change plan, suppresses nested native registrations and records one source-and-selection step. Replay checks exact source and current editor permissions before applying; a mismatch retires incompatible native history while retaining the current draft. Other editing paths keep their existing history behavior. Source snapshots are retained by native undo entries rather than discarded on parser or provider attachment.

<ui:copy id="native-source-draft-history">
import Foundation
@MainActor final class SourceDraftHistory {
    private var retirementObservers:[ObjectIdentifier:NSObjectProtocol]=[:]
    private func retire(_ manager:UndoManager) {
        guard manager.isUndoing || manager.isRedoing else{manager.removeAllActions();return}
        let key=ObjectIdentifier(manager)
        guard retirementObservers[key]==nil else{return}
        let event=manager.isUndoing ? NSNotification.Name.NSUndoManagerDidUndoChange : NSNotification.Name.NSUndoManagerDidRedoChange
        retirementObservers[key]=NotificationCenter.default.addObserver(forName:event,object:manager,queue:nil){[weak self,weak manager] _ in
            MainActor.assumeIsolated {
                guard let self else{return}
                if let token=self.retirementObservers.removeValue(forKey:key){NotificationCenter.default.removeObserver(token)}
                manager?.removeAllActions()
            }
        }
    }
    private struct Step {
        var before:String;var after:String;var edits:[NativeSourceEdit];var inverse:[NativeSourceEdit]
        var selectionBefore:NSRange;var selectionAfter:NSRange;var title:String
        var reversed:Step {Step(before:after,after:before,edits:inverse,inverse:edits,selectionBefore:selectionAfter,selectionAfter:selectionBefore,title:title)}
    }
    typealias Apply=([NativeSourceEdit],String,NSRange)->Bool
    func commit(_ plan:ChangePlan,path:String,before:String,selectionBefore:NSRange,selectionAfter:NSRange,manager:UndoManager?,read:@escaping ()->String,apply:@escaping Apply,notice:@escaping (String)->Void)->Bool {
        guard plan.patches.count<=1,exactUTF8(read(),before) else{return false}
        let patch=plan.patches.first
        if let patch{guard exactUTF8(patch.path,path),exactUTF8(patch.before,before) else{return false}}
        let step=Step(before:before,after:patch?.after ?? before,edits:(patch?.edits ?? []).map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},inverse:(invertChangePlan(plan).patches.first?.edits ?? []).map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},selectionBefore:selectionBefore,selectionAfter:selectionAfter,title:plan.title)
        guard perform(step,manager:manager,read:read,apply:apply) else{return false}
        if !exactUTF8(step.before,step.after),let manager{register(step.reversed,manager:manager,read:read,apply:apply,notice:notice)}
        return true
    }
    private func perform(_ step:Step,manager:UndoManager?,read:()->String,apply:Apply)->Bool {
        guard exactUTF8(read(),step.before) else{return false}
        manager?.disableUndoRegistration();defer{manager?.enableUndoRegistration()}
        return apply(step.edits,step.before,step.selectionAfter) && exactUTF8(read(),step.after)
    }
    private func register(_ step:Step,manager:UndoManager,read:@escaping ()->String,apply:@escaping Apply,notice:@escaping (String)->Void) {
        let group = !manager.isUndoing && !manager.isRedoing
        if group{manager.beginUndoGrouping()}
        manager.registerUndo(withTarget:self){[weak manager] history in
            guard let manager else{return}
            guard history.perform(step,manager:manager,read:read,apply:apply) else{
                history.retire(manager);notice("Undo history retired because source or permissions changed. Current draft is retained.");return
            }
            history.register(step.reversed,manager:manager,read:read,apply:apply,notice:notice)
        }
        manager.setActionName(step.title)
        if group{manager.endUndoGrouping()}
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SourceDraftHistory.swift"><ui:paste select="#native-source-draft-history"/></ui:file>

</ui:doc>
