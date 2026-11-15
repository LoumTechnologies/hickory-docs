import {expect,it,vi} from 'vitest';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {history,undo,redo} from '@codemirror/commands';
import {acceptCompletion,completionStatus,setSelectedCompletion} from '@codemirror/autocomplete';
import {waitFor} from '@testing-library/react';
import {completions} from '../lsp/completion';
import {beginSnippet} from './SnippetSession';
import {sourceSnippets,setSourceSnippet,readSourceSnippet,moveSourceSnippet,openSourceSnippetChoices,dispatchSourceSnippetTransactions} from './SourceSnippets';
import type {SnippetField} from './SnippetExpansion';
import {acceptSourceCompletion} from './SourceCompletion';
import {readCompletionAcceptance} from '../lsp/completionAcceptance';
import type {CompletionItem} from '../lsp/client';
const field=(id:number,index:number,from:number,to:number,choices:string[]=[]):SnippetField=>({id,index,from,to,choices,parent:null});
function fixture(source='a + a',fields=[field(0,1,0,1),field(1,1,4,5),field(2,0,5,5)],options:{writable?:()=>boolean;provider?:()=>Promise<never[]>}={}){
 const parent=document.body.appendChild(document.createElement('div')),notices:string[]=[];
 const view=new EditorView({parent,dispatchTransactions:dispatchSourceSnippetTransactions,state:EditorState.create({doc:source,extensions:[history(),sourceSnippets({writable:options.writable??(()=>true),notice:value=>notices.push(value)}),completions(options.provider?{project:options.provider}:{})]})});
 view.dispatch({selection:{anchor:fields[0].from,head:fields[0].to},effects:setSourceSnippet.of(beginSnippet(source,fields))});view.focus();
 return {view,notices,close:()=>{view.destroy();parent.remove();}};
}
it('actual input mirrors all fields and restores field state with one undo/redo',()=>{
 const {view,close}=fixture();try{
  view.dispatch({changes:{from:0,to:1,insert:'long'},selection:{anchor:4},userEvent:'input.type'});
  expect(view.state.doc.toString()).toBe('long + long');expect(view.state.selection.main.head).toBe(4);expect(readSourceSnippet(view)?.source).toBe('long + long');
  expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('a + a');expect(readSourceSnippet(view)?.source).toBe('a + a');
  expect(redo(view)).toBe(true);expect(view.state.doc.toString()).toBe('long + long');expect(readSourceSnippet(view)?.source).toBe('long + long');expect(view.state.selection.main.anchor).toBe(4);expect(view.state.selection.main.head).toBe(4);
  expect(moveSourceSnippet(view,-1)).toBe(true);expect(readSourceSnippet(view)).not.toBeNull();
  expect(moveSourceSnippet(view,1)).toBe(true);expect(view.state.selection.main.head).toBe(11);expect(readSourceSnippet(view)).toBeNull();
 }finally{close();}
});
it('plain provider completion preserves all current mirrors and their history',async()=>{
 const {view,close}=fixture();try{const item:CompletionItem={label:'long',textEdit:{range:{start:{line:0,character:0},end:{line:0,character:1}},newText:'long'}};
  await acceptSourceCompletion(view,item,'a + a',view.state.selection,0,1,{uri:'hick:///a.py',resolve:async()=>item,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(view.state.doc.toString()).toBe('long + long');expect(readSourceSnippet(view)?.source).toBe('long + long');expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('a + a');expect(readSourceSnippet(view)?.source).toBe('a + a');
 }finally{close();}
});
it('nested completion fields mirror across outer occurrences and restore outer scope after undo',async()=>{
 const {view,close}=fixture();try{const syntax='${1:x}:$1$0',inner=[field(0,1,0,1),field(1,1,2,3),field(2,0,3,3)];
  const item:CompletionItem={label:'nested',insertTextFormat:2,textEdit:{range:{start:{line:0,character:0},end:{line:0,character:1}},newText:syntax},data:{hickory_snippet:{version:1,syntax,text:'x:x',fields:inner}},additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'# import\n'}]};
  await acceptSourceCompletion(view,item,'a + a',view.state.selection,0,1,{uri:'hick:///a.py',resolve:async()=>item,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(view.state.doc.toString()).toBe('# import\nx:x + x:x');expect(readSourceSnippet(view)?.scopes?.length).toBe(1);expect(view.state.selection.main.from).toBe(9);
  view.dispatch({changes:{from:9,to:10,insert:'long'},selection:{anchor:13},userEvent:'input.type'});expect(view.state.doc.toString()).toBe('# import\nlong:long + long:long');
  expect(undo(view)).toBe(true);expect(readSourceSnippet(view)?.scopes?.length).toBe(1);expect(redo(view)).toBe(true);expect(readSourceSnippet(view)?.scopes?.length).toBe(1);expect(view.state.selection.main.head).toBe(13);
  expect(moveSourceSnippet(view,1)).toBe(true);expect(view.state.selection.main.head).toBe(18);expect(readSourceSnippet(view)?.active).toBe(1);expect(moveSourceSnippet(view,1)).toBe(true);expect(view.state.selection.main.head).toBe(30);expect(readSourceSnippet(view)).toBeNull();
 }finally{close();}
});
it('source-equal completion retains template scope without a text history step',async()=>{
 const {view,close}=fixture();try{const item:CompletionItem={label:'a',insertText:'a'};
  await acceptSourceCompletion(view,item,'a + a',view.state.selection,0,1,{uri:'hick:///a.py',resolve:async()=>item,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(view.state.doc.toString()).toBe('a + a');expect(readCompletionAcceptance(view)).toMatchObject({phase:'applied',source_changed:false});expect(readSourceSnippet(view)?.active).toBe(1);expect(undo(view)).toBe(false);
 }finally{close();}
});
it('completion imports apply once while the primary replacement updates every mirror',async()=>{
 const {view,close}=fixture();try{const item:CompletionItem={label:'long',textEdit:{range:{start:{line:0,character:0},end:{line:0,character:1}},newText:'long'},additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'# import\n'},{range:{start:{line:0,character:5},end:{line:0,character:5}},newText:'\n# end'}]};
  await acceptSourceCompletion(view,item,'a + a',view.state.selection,0,1,{uri:'hick:///a.py',resolve:async()=>item,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(view.state.doc.toString()).toBe('# import\nlong + long\n# end');expect(view.state.selection.main.head).toBe(13);expect(readSourceSnippet(view)?.fields.map(field=>[field.from,field.to])).toEqual([[9,13],[16,20],[20,20]]);
  expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe('a + a');expect(readSourceSnippet(view)?.source).toBe('a + a');expect(redo(view)).toBe(true);expect(view.state.selection.main.head).toBe(13);expect(readSourceSnippet(view)?.source).toBe(view.state.doc.toString());
 }finally{close();}
});
it('additional edits inside the active template forest refuse as a whole',async()=>{
 const {view,close}=fixture();try{const item:CompletionItem={label:'long',textEdit:{range:{start:{line:0,character:0},end:{line:0,character:1}},newText:'long'},additionalTextEdits:[{range:{start:{line:0,character:2},end:{line:0,character:2}},newText:'\nimport other'}]};
  await acceptSourceCompletion(view,item,'a + a',view.state.selection,0,1,{uri:'hick:///a.py',resolve:async()=>item,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(view.state.doc.toString()).toBe('a + a');expect(readCompletionAcceptance(view)).toMatchObject({phase:'unavailable',applied:false,source_changed:false});expect(readSourceSnippet(view)?.source).toBe('a + a');
 }finally{close();}
});
it('equal-position field navigation retires a held completion intent',async()=>{
 const source='a',fields=[{...field(1,2,0,1),parent:0},field(0,1,0,1),field(2,0,1,1)],{view,close}=fixture(source,fields);
 let resolve!:(item:CompletionItem)=>void;const held=new Promise<CompletionItem>(done=>{resolve=done;});
 try{const selection=view.state.selection,item:CompletionItem={label:'long'},work=acceptSourceCompletion(view,item,source,selection,0,1,{uri:'hick:///a.py',resolve:()=>held,current:()=>true,writable:()=>true,notice:()=>{}});
  expect(moveSourceSnippet(view,1)).toBe(true);expect(view.state.selection.eq(selection)).toBe(true);expect(readSourceSnippet(view)?.active).toBe(2);resolve({label:'long',insertText:'long'});await work;
  expect(view.state.doc.toString()).toBe(source);expect(readCompletionAcceptance(view)?.phase).toBe('unavailable');
 }finally{close();}
});
it('smart-enter input retains its intended caret inside the linked replacement',()=>{
 const source='call() + call()',fields=[field(0,1,0,6),field(1,1,9,15),field(2,0,15,15)],{view,close}=fixture(source,fields);
 try{view.dispatch({selection:{anchor:5}});view.dispatch({changes:{from:5,insert:'\n  \n'},selection:{anchor:8},userEvent:'input.type'});expect(view.state.doc.toString()).toBe('call(\n  \n) + call(\n  \n)');expect(view.state.selection.main.head).toBe(8);expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe(source);}finally{close();}
});
it('source-local choices use the existing popup while providers are unavailable',async()=>{
 const provider=vi.fn(async()=>[] as never[]),source='first + first',fields=[field(0,1,0,5,['first','second']),field(1,1,8,13,['first','second']),field(2,0,13,13)],{view,close}=fixture(source,fields,{provider});
 try{expect(openSourceSnippetChoices(view)).toBe(true);await waitFor(()=>expect(completionStatus(view.state)).toBe('active'));expect(provider).not.toHaveBeenCalled();view.dispatch({effects:setSelectedCompletion(1)});await waitFor(()=>expect(acceptCompletion(view)).toBe(true));expect(view.state.doc.toString()).toBe('second + second');expect(view.state.selection.main.head).toBe(6);expect(readSourceSnippet(view)?.active).toBe(1);expect(undo(view)).toBe(true);expect(view.state.doc.toString()).toBe(source);expect(readSourceSnippet(view)?.source).toBe(source);}finally{close();}
});
it('unavailable mirroring retains ordinary input and retires its field context',()=>{
 for(const saving of[false,true]){let writable=!saving;const {view,close}=fixture(undefined,undefined,{writable:()=>writable});try{
  if(!saving)view.dispatch({selection:{anchor:2}});
  view.dispatch({changes:{from:saving?0:2,to:saving?1:2,insert:'x'},selection:{anchor:saving?1:3},userEvent:'input.type'});
  expect(view.state.doc.toString()).toBe(saving?'x + a':'a x+ a');expect(readSourceSnippet(view)).toBeNull();writable=true;
 }finally{close();}}
});
it('coincident empty mirrors preserve every occurrence in actual editor input',()=>{
 const fields=[field(0,1,0,0),field(1,2,0,0),field(2,1,0,0),field(3,0,0,0)],{view,close}=fixture('',fields);
 try{view.dispatch({changes:{from:0,insert:'x'},selection:{anchor:1},userEvent:'input.type'});expect(view.state.doc.toString()).toBe('xx');expect(view.state.selection.main.head).toBe(1);expect(moveSourceSnippet(view,1)).toBe(true);expect(view.state.selection.main.head).toBe(1);view.dispatch({changes:{from:1,insert:'y'},selection:{anchor:2},userEvent:'input.type'});expect(view.state.doc.toString()).toBe('xyx');}finally{close();}
});
