<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Resolve actions before admitting their complete edits

For maintainers changing provider interaction without replacing editor presentation.

A held action resolves before application, including eager values with engine lookup tokens. Legacy complete edits remain usable without an unsupported resolver. The origin text-document field lets the wire synchronize that source before this otherwise URI-less request. Returned commands remain commands: this helper neither executes them nor applies an edit portion. Each host still checks its own retained choice, source, selection and provider before and after the await.

Keep resolution independent of source mutation. A missing resolved value refuses instead of substituting an incomplete edit.

<ui:copy id="browser-provider-action-resolution">
import type {CodeAction} from './client';
export interface CodeActionResolver {request:(method:string,params?:unknown)=>Promise<unknown>}
export function actionNeedsResolution(action:CodeAction):boolean {
 const data=action.data,held=!!data&&typeof data==='object'&&typeof (data as {hickory_action?:unknown}).hickory_action==='string';
 return held||(!action.edit&&data!==undefined&&data!==null);
}
/** Resolve retained provider data without granting command or edit authority. */
export async function resolveCodeAction(client:CodeActionResolver,action:CodeAction,uri:string):Promise<CodeAction>{
 if(!actionNeedsResolution(action))return action;
 const value=await client.request('codeAction/resolve',{...action,textDocument:{uri}});
 if(!value||typeof value!=='object'||Array.isArray(value)||typeof (value as CodeAction).title!=='string')throw new Error('Provider did not resolve this action. Prepare actions again.');
 return value as CodeAction;
}
</ui:copy>

<ui:file path="apps/web/src/lsp/codeActionResolution.ts"><ui:paste select="#browser-provider-action-resolution"/></ui:file>

The focused witness checks legacy eager fallback, complete returned command/edit values, origin synchronization context and explicit malformed/refused resolution. It does not apply source or simulate provider semantics.

<ui:copy id="browser-provider-action-resolution-witness">
import {expect,it,vi} from 'vitest';
import {resolveCodeAction} from './codeActionResolution';
it('eager legacy actions remain usable without an unsupported resolver',async()=>{
 const action={title:'Eager',edit:{changes:{}}},client={request:vi.fn()};expect(await resolveCodeAction(client,action,'hick:///a.py')).toBe(action);expect(client.request).not.toHaveBeenCalled();
});
it('held actions resolve even with an edit and retain the complete returned command boundary',async()=>{
 const action={title:'Held',data:{hickory_action:'token'},edit:{changes:{}}},resolved={title:'Resolved',edit:{changes:{}},command:{title:'Provider command',command:'provider.effect',arguments:['opaque']}};
 const client={request:vi.fn(async()=>resolved)};expect(await resolveCodeAction(client,action,'hick:///a.py')).toBe(resolved);
 expect(client.request).toHaveBeenCalledWith('codeAction/resolve',{...action,textDocument:{uri:'hick:///a.py'}});
});
it('unresolved opaque actions propagate refusal and never substitute an eager partial value',async()=>{
 const action={title:'Lazy',data:{opaque:[1,2]}};await expect(resolveCodeAction({request:async()=>null},action,'hick:///a.py')).rejects.toThrow('did not resolve');
 await expect(resolveCodeAction({request:async()=>{throw new Error('source changed');}},action,'hick:///a.py')).rejects.toThrow('source changed');
});
</ui:copy>

<ui:file path="apps/web/src/lsp/codeActionResolution.test.ts"><ui:paste select="#browser-provider-action-resolution-witness"/></ui:file>

## Gate current-editor application after resolution

Hold actual controller resolver completions against a real CodeMirror view. A newer choice wins without a late older status message, and controlled composition state prevents application. This is model/editor gating evidence, not device IME or external-provider acceptance. The separate engine witness proves provider routing.

<ui:copy id="browser-resolved-action-editor-witness">
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
</ui:copy>

<ui:file path="apps/web/src/lsp/codeActionResolutionEditor.test.ts"><ui:paste select="#browser-resolved-action-editor-witness"/></ui:file>
</ui:doc>
