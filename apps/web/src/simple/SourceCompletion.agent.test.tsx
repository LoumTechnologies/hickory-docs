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
