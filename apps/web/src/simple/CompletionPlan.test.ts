import {expect,it} from 'vitest';
import cases from '../../../simple-contract/completion-plan-cases.json';
import {planCompletion,type CompletionExpansion} from './CompletionPlan';
import derived from '../../../simple-contract/snippet-derived-cases.json';
import {readSnippetTransform} from './SnippetTransform';
import {beginSnippet,editSnippet} from './SnippetSession';
import {invertChangePlan,validateChangePlan} from './ChangePlan';
it('prepares source, import order, UTF16 selection and refusal from shared completion vectors',()=>{
 for(const item of cases.cases){
  const prepare=()=>planCompletion('uri' in item?item.uri!:'hick:///a.py',item.item,{text:item.source,writable:'writable' in item?item.writable!==false:true},item.from,item.to,'expansion' in item?item.expansion as CompletionExpansion:undefined);
  if('error' in item&&item.error){expect(prepare,item.name).toThrow();continue;}
  const value=prepare();expect(value.before,item.name).toBe(item.source);expect(value.plan.patches[0]?.after??item.source,item.name).toBe('after' in item?item.after:undefined);expect(value.selection,item.name).toEqual('selection' in item?item.selection:undefined);expect(value.insertionFrom,item.name).toBe('insertionFrom' in item?item.insertionFrom:undefined);
  if('patchCount' in item)expect(value.plan.patches).toHaveLength(item.patchCount!);
  if(value.plan.patches.length){const inverse=invertChangePlan(value.plan);expect(inverse.patches[0].after).toBe(item.source);expect(validateChangePlan(inverse,new Map([['a.py',{text:value.plan.patches[0].after,writable:true}]]))).toBeNull();}
 }
});
it('preserves rebased computed programs through import edits and usable linked input',()=>{
 const transform=readSnippetTransform(derived.programs.upper),fields=[{index:1,from:0,to:3,choices:[],id:0,parent:null},{index:1,from:4,to:7,choices:[],id:1,parent:null,transform},{index:0,from:7,to:7,choices:[],id:2,parent:null}],draft=planCompletion('hick:///a.py',{label:'derived',insertTextFormat:2,insertText:'${1:foo} ${1/(.*)/${1:/upcase}/}$0',additionalTextEdits:[{range:{start:{line:0,character:0},end:{line:0,character:0}},newText:'import\n'}]},{text:'😀 old\n',writable:true},3,6,{text:'foo FOO',selection:{from:0,to:3},fields}),after=draft.plan.patches[0].after;
 expect(after).toBe('import\n😀 foo FOO\n');expect(draft.fields![1].from).toBe(14);expect(draft.fields![1].transform).toBe(transform);const session=beginSnippet(after,draft.fields!)!;expect(session).not.toBeNull();expect(editSnippet(session,after,draft.selection.from,draft.selection.to,'ßa')?.session.source).toBe('import\n😀 ßa SSA\n');
});
