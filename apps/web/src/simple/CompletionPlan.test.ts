import {expect,it} from 'vitest';
import cases from '../../../simple-contract/completion-plan-cases.json';
import {planCompletion,type CompletionExpansion} from './CompletionPlan';
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
