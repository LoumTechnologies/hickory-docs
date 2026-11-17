import {expect,it} from 'vitest';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {history,undo,redo} from '@codemirror/commands';
import {acceptSourceCompletion,sourceCompletion,type SourceCompletionAdapter} from './SourceCompletion';
import {readCompletionAcceptance,waitCompletionAcceptance,cancelCompletionAcceptance,withCompletionAcceptanceSignal} from '../lsp/completionAcceptance';
import type {CompletionItem} from '../lsp/client';
import {CompletionContext} from '@codemirror/autocomplete';
import {sourceSnippets,dispatchSourceSnippetTransactions} from './SourceSnippets';
import snippetVectors from '../../../simple-contract/snippet-expansion-cases.json';
function held(){let resolve!:(value:CompletionItem)=>void;const promise=new Promise<CompletionItem>(done=>{resolve=done;});return {promise,resolve};}
function fixture(){const parent=document.body.appendChild(document.createElement('div'));const view=new EditorView({dispatchTransactions:dispatchSourceSnippetTransactions,state:EditorState.create({doc:'va\n',selection:{anchor:2},extensions:[history(),sourceSnippets({writable:()=>true,notice:()=>{}})]}),parent});return {view,close:()=>{view.destroy();parent.remove();}};}
const raw={label:'value',data:{hickory_completion:'held'}};
function resolved(text='value'):CompletionItem {return {...raw,textEdit:{range:{start:{line:0,character:0},end:{line:0,character:2}},newText:text},additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'# import\n'}]};}
it('waits for the whole resolved import edit, commits one undo and maps the caret',async()=>{
 const {view,close}=fixture(),pending=held(),notices:string[]=[];
 try{const work=acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,{uri:'hick:///a.py',resolve:()=>pending.promise,current:()=>true,writable:()=>true,notice:value=>notices.push(value)});
  const before=readCompletionAcceptance(view)!;const finished=waitCompletionAcceptance(view,before.job);expect(before.phase).toBe('resolving');expect(view.state.doc.toString()).toBe('va\n');pending.resolve(resolved());await work;
  expect((await finished).phase).toBe('applied');expect(view.state.doc.toString()).toBe('# import\nvalue\n');expect(view.state.selection.main.head).toBe(14);expect(notices.at(-1)).toContain('draft');expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('va\n');expect(redo(view)).toBe(true);expect(view.state.doc.toString()).toBe('# import\nvalue\n');
 }finally{close();}
});
it('restores snippet field selection with complete imports on redo',async()=>{
 const {view,close}=fixture();try{
  const resolved=snippetVectors.cases[0].item as CompletionItem;
  await acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,{uri:'hick:///a.py',resolve:async()=>resolved,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(view.state.doc.toString()).toBe('#\n😀 + 😀\n');expect(view.state.selection.main.anchor).toBe(2);expect(view.state.selection.main.head).toBe(4);
  expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('va\n');expect(redo(view)).toBe(true);
  expect(view.state.doc.toString()).toBe('#\n😀 + 😀\n');expect(view.state.selection.main.anchor).toBe(2);expect(view.state.selection.main.head).toBe(4);
 }finally{close();}
});
it('accepts a source-equal completion without a fake source edit',async()=>{
 const {view,close}=fixture();try{await acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,{uri:'hick:///a.py',resolve:async()=>({...resolved('va'),additionalTextEdits:[]}),current:()=>true,writable:()=>true,notice:()=>{}});expect(view.state.doc.toString()).toBe('va\n');expect(readCompletionAcceptance(view)).toMatchObject({phase:'applied',applied:true,source_changed:false});}finally{close();}
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

it('an offered query cannot revive after equal-byte undo or a cursor round trip',async()=>{
 for(const sourceABA of[true,false]){const {view,close}=fixture();let calls=0;try{const item=sourceCompletion(raw,new CompletionContext(view.state,2,true),{uri:'hick:///a.py',resolve:async()=>{calls++;return resolved()},current:()=>true,writable:()=>true,notice:()=>{}});if(sourceABA){view.dispatch({changes:{from:0,to:2,insert:'xy'},selection:{anchor:2},userEvent:'input.type'});expect(undo(view)).toBe(true)}else{view.dispatch({selection:{anchor:0}});view.dispatch({selection:{anchor:2}})}expect(view.state.doc.toString()).toBe('va\n');expect(view.state.selection.main.head).toBe(2);expect(typeof item.apply).toBe('function');(item.apply as Exclude<typeof item.apply,string|undefined>)(view,item,0,2);await waitCompletionAcceptance(view,readCompletionAcceptance(view)!.job);expect(calls).toBe(0);expect(view.state.doc.toString()).toBe('va\n');expect(readCompletionAcceptance(view)?.phase).toBe('unavailable')}finally{close()}}
});
it('a pending provider cannot revive after equal-byte undo or a cursor round trip',async()=>{
 for(const sourceABA of[true,false]){const {view,close}=fixture(),pending=held();try{const work=acceptSourceCompletion(view,raw,'va\n',view.state.selection,0,2,{uri:'hick:///a.py',resolve:()=>pending.promise,current:()=>true,writable:()=>true,notice:()=>{}});if(sourceABA){view.dispatch({changes:{from:0,to:2,insert:'xy'},selection:{anchor:2},userEvent:'input.type'});expect(undo(view)).toBe(true)}else{view.dispatch({selection:{anchor:0}});view.dispatch({selection:{anchor:2}})}pending.resolve(resolved());await work;expect(view.state.doc.toString()).toBe('va\n');expect(view.state.selection.main.head).toBe(2);expect(readCompletionAcceptance(view)?.phase).toBe('unavailable')}finally{close()}}
});
