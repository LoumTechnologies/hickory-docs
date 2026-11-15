import {expect,it,vi} from 'vitest';
import {waitFor} from '@testing-library/react';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import type {LspClient,WorkspaceEdit} from './client';
import {codeActionController} from './codeActionController';
function fixture(){
 const host=document.createElement('div');document.body.append(host);const view=new EditorView({state:EditorState.create({doc:'bad'}),parent:host});
 const held:((value:unknown)=>void)[]=[],message=vi.fn(),apply=vi.fn((target:EditorView,edit:WorkspaceEdit|null)=>{const text=edit?.changes?.['hick:///file.py'][0].newText??'';target.dispatch({changes:{from:0,to:target.state.doc.length,insert:text}});return true;});
 let sequence=0;const client={codeActions:async()=>[{title:`Choice ${++sequence}`,data:{hickory_action:String(sequence)}}],request:vi.fn(()=>new Promise(resolve=>held.push(resolve)))} as unknown as LspClient;
 const controller=codeActionController({client,uri:'hick:///file.py',positionAt:p=>({line:0,character:p}),offsetAt:p=>p.character,onCodeActions:actions=>actions[0],onMessage:message},apply);
 const result=(text:string)=>({title:'Resolved',edit:{changes:{'hick:///file.py':[{range:{start:{line:0,character:0},end:{line:0,character:3}},newText:text}]}}});
 return {view,held,message,apply,controller,result,close:()=>{view.destroy();host.remove();}};
}
it('a late choice cannot edit or replace the message after a newer choice completes',async()=>{
 const value=fixture();try{
  value.controller.human(value.view);await waitFor(()=>expect(value.held).toHaveLength(1));value.controller.human(value.view);await waitFor(()=>expect(value.held).toHaveLength(2));
  value.held[1](value.result('second'));await waitFor(()=>expect(value.view.state.doc.toString()).toBe('second'));
  value.held[0](value.result('first'));await new Promise(resolve=>setTimeout(resolve,0));expect(value.apply).toHaveBeenCalledTimes(1);expect(value.message).not.toHaveBeenCalled();
 }finally{value.close();}
});
it('composition beginning during resolution refuses the current-editor mutation',async()=>{
 const value=fixture();try{
  value.controller.human(value.view);await waitFor(()=>expect(value.held).toHaveLength(1));Object.defineProperty(value.view,'composing',{get:()=>true,configurable:true});
  value.held[0](value.result('unsafe'));await waitFor(()=>expect(value.message).toHaveBeenCalled());expect(value.apply).not.toHaveBeenCalled();expect(value.view.state.doc.toString()).toBe('bad');
 }finally{value.close();}
});
