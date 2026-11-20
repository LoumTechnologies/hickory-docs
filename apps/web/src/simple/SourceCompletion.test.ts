import {expect,it} from 'vitest';
import {EditorSelection,EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {history,undo,redo} from '@codemirror/commands';
import {acceptSourceCompletion,sourceCompletion,type SourceCompletionAdapter} from './SourceCompletion';
import {readCompletionAcceptance,waitCompletionAcceptance,cancelCompletionAcceptance,withCompletionAcceptanceSignal} from '../lsp/completionAcceptance';
import type {CompletionItem} from '../lsp/client';
import {sourceDigest} from '../lsp/diagnosticRevision';
import {CompletionContext} from '@codemirror/autocomplete';
import {sourceSnippets,dispatchSourceSnippetTransactions,applyParallelSourceSnippet,readParallelSourceSnippet,readSourceEditorIntent} from './SourceSnippets';
import {sourceText,authoredSource,editorOffsetIn} from './SourceText';
import snippetVectors from '../../../simple-contract/snippet-expansion-cases.json';
function held(){let resolve!:(value:CompletionItem)=>void;const promise=new Promise<CompletionItem>(done=>{resolve=done;});return {promise,resolve};}
function fixture(source='va\n',caret=2){const parent=document.body.appendChild(document.createElement('div'));const view=new EditorView({dispatchTransactions:dispatchSourceSnippetTransactions,state:EditorState.create({doc:source,selection:{anchor:editorOffsetIn(source,caret)},extensions:[EditorState.allowMultipleSelections.of(true),history(),sourceText(source),sourceSnippets({writable:()=>true,notice:()=>{}})]}),parent});return {view,close:()=>{view.destroy();parent.remove();}};}
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
it('retires a held peer completion after changing and restoring the secondary caret',async()=>{
 const source='va\nva\n',{view,close}=fixture(source),pending=held();
 try {
  const hash=await sourceDigest(source),context={source_hash:hash,source_revision:'a'.repeat(64)};
  const offer=(token:string):CompletionItem=>({label:'value',data:{hickory_completion:token,hickory_context:context}});
  const first=offer('first'),peer=offer('peer');
  const selections=(end:number)=>EditorSelection.create([EditorSelection.cursor(2),EditorSelection.cursor(end)]);
  const original=selections(5);
  view.dispatch({selection:original});
  let beginPeer!:()=>void;
  const resolvingPeer=new Promise<void>(resolve=>{beginPeer=resolve;});
  const work=acceptSourceCompletion(view,first,source,original,0,2,{
   uri:'hick:///a.py',current:()=>true,writable:()=>true,notice:()=>{},
   resolve:async()=>{throw new Error('Single-caret resolution was unexpected');},
   group:{query:async offset=>{expect(offset).toBe(5);return [peer];},resolve:async item=>{
    if((item.data as {hickory_completion:string}).hickory_completion==='first') return first;
    beginPeer();
    return pending.promise;
   }},
  });
  await resolvingPeer;
  view.dispatch({selection:selections(4)});
  view.dispatch({selection:original});
  pending.resolve(peer);
  await work;
  expect(authoredSource(view.state)).toBe(source);
  expect(view.state.selection.eq(original)).toBe(true);
  expect(readCompletionAcceptance(view)?.phase).toBe('unavailable');
  expect(undo(view)).toBe(false);
 } finally {close();}
});
it('reads the clipboard once after validating both grouped demands and shared edits',async()=>{
 const source='va\nva\n';
 const descriptor=Object.getOwnPropertyDescriptor(navigator,'clipboard');
 const phases:string[]=[];
 Object.defineProperty(navigator,'clipboard',{configurable:true,value:{readText:async()=>{phases.push('clipboard');return 'shared';}}});
 try {
  const hash=await sourceDigest(source),context={source_hash:hash,source_revision:'a'.repeat(64)};
  const offer=(token:string):CompletionItem=>({label:'value',data:{hickory_completion:token,hickory_context:context}});
  const syntax='${1:${CLIPBOARD}}$0';
  for(const conflict of [true,false]){
   const {view,close}=fixture(source);
   try {
    phases.length=0;
    const selection=EditorSelection.create([EditorSelection.cursor(2),EditorSelection.cursor(5)],1);
    view.dispatch({selection});
    await acceptSourceCompletion(view,offer('second'),source,selection,3,5,{
     uri:'hick:///a.py',current:()=>true,writable:()=>true,notice:()=>{},
     resolve:async()=>{throw new Error('Unexpected single resolution');},
     group:{query:async()=>[offer('first')],resolve:async item=>{
      const data=item.data as {hickory_completion:string;hickory_context:{selection:{from:number;to:number};carets:{index:number;count:number};clipboard:unknown}};
      const {index,count}=data.hickory_context.carets;
      expect(count).toBe(2);
      expect(data.hickory_context.selection).toEqual(index===0?{from:2,to:2}:{from:5,to:5});
      const retry=data.hickory_completion.endsWith('-retry');
      phases.push(`${retry?'retry':'first'}-${index}`);
      const text=index===0?'one':'two';
      const resultData={hickory_completion:retry?data.hickory_completion:data.hickory_completion+'-retry',hickory_context:context};
      if(retry) expect(data.hickory_context.clipboard).toEqual({available:true,text:'shared'});
      else expect(data.hickory_context.clipboard).toBeNull();
      return {
       label:item.label,insertTextFormat:2,
       textEdit:{range:{start:{line:index,character:0},end:{line:index,character:2}},newText:syntax},
       additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:conflict&&index?'# other\n':'# import\n'}],
       data:retry?{...resultData,hickory_snippet:{version:2,syntax,text,fields:[
        {index:1,from:0,to:3,choices:[],id:0,parent:null},{index:0,from:3,to:3,choices:[],id:1,parent:null},
       ]}}:{...resultData,hickory_context_request:{version:1,variables:['CLIPBOARD']}},
      };
     }},
    });
    expect(phases).toEqual(conflict?['first-1','first-0']:['first-1','first-0','clipboard','retry-1','retry-0']);
    expect(authoredSource(view.state)).toBe(conflict?source:'# import\none\ntwo\n');
    expect(readParallelSourceSnippet(view.state)?.peers.length ?? 0).toBe(conflict?0:2);
   } finally {close();}
  }
 } finally {
  if(descriptor)Object.defineProperty(navigator,'clipboard',descriptor);
  else Reflect.deleteProperty(navigator,'clipboard');
 }
});
it('retains the active template group when an ordinary completion is chosen',async()=>{
 const source='va\nva\n',{view,close}=fixture(source);
 try {
  const ranges=[{from:0,to:2},{from:3,to:5}];
  view.dispatch({selection:EditorSelection.create(ranges.map(range=>EditorSelection.range(range.from,range.to)))});
  const fields=[
   {index:1,from:0,to:5,choices:[],id:0,parent:null},
   {index:0,from:5,to:5,choices:[],id:1,parent:null},
  ];
  expect(applyParallelSourceSnippet(view,source,ranges.map(range=>({...range,text:'value',fields})),0,readSourceEditorIntent(view.state)!,()=>true)).toBe(true);
  const parallel=readParallelSourceSnippet(view.state),draft=authoredSource(view.state),selection=view.state.selection;
  let calls=0;
  const resolve=async()=>{calls++;return resolved();};
  await acceptSourceCompletion(view,raw,draft,selection,0,5,{
   uri:'hick:///a.py',current:()=>true,writable:()=>true,notice:()=>{},resolve,
   group:{query:async()=>{calls++;return [raw];},resolve},
  });
  expect(calls).toBe(0);
  expect(authoredSource(view.state)).toBe(draft);
  expect(readParallelSourceSnippet(view.state)).toBe(parallel);
  expect(view.state.selection.eq(selection)).toBe(true);
  expect(readCompletionAcceptance(view)?.message).toContain('Finish editing the template');
 } finally {close();}
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

it('applies complete raw CRLF edits through a normalized editor and restores exact bytes with Undo and Redo',async()=>{
 const source='head\r\nva\r\nend\n',after='# imported\r\nhead\r\nλ😀\nnext\r\nend\n',{view,close}=fixture(source,8);
 try{const item=sourceCompletion(raw,new CompletionContext(view.state,7,true),{uri:'hick:///a.py',resolve:async()=>({...raw,textEdit:{range:{start:{line:1,character:0},end:{line:1,character:2}},newText:'λ😀\nnext'},additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'# imported\r\n'}]}),current:()=>true,writable:()=>true,notice:()=>{}});
  (item.apply as Exclude<typeof item.apply,string|undefined>)(view,item,5,7);await waitCompletionAcceptance(view,readCompletionAcceptance(view)!.job);
  expect(readCompletionAcceptance(view)?.phase).toBe('applied');expect(authoredSource(view.state)).toBe(after);expect(view.state.doc.toString()).toBe(after.replace(/\r\n/g,'\n'));const caret=view.state.selection.main.head;
  expect(undo(view)).toBe(true);expect(authoredSource(view.state)).toBe(source);expect(view.state.selection.main.head).toBe(7);expect(redo(view)).toBe(true);expect(authoredSource(view.state)).toBe(after);expect(view.state.selection.main.head).toBe(caret);
 }finally{close()}
});
