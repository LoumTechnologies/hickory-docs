import {expect,it} from 'vitest';
import vectors from '../../../simple-contract/snippet-expansion-cases.json';
import {completionExpansion} from './SnippetExpansion';
import {planCompletion} from './CompletionPlan';
it('validates shared snippet payloads and maps fields through imports',()=>{
 for(const value of vectors.cases){
  const expand=()=>completionExpansion(value.item);
  if('error' in value){expect(expand,value.name).toThrow();continue;}
  if('plain' in value){expect(expand(),value.name).toBeUndefined();continue;}
  const expansion=expand()!,expected=value.expected!;expect(expansion.text,value.name).toBe(expected.text);expect(expansion.selection,value.name).toEqual(expected.selection);
  const draft=planCompletion('hick:///a.py',value.item,{text:'va',writable:true},0,2,expansion);
  expect(draft.plan.patches[0].after,value.name).toBe(expected.after);for(const [index,field] of expected.fields.entries()){if('id' in field&&'parent' in field){expect(draft.fields[index].id,value.name).toBe(field.id);expect(draft.fields[index].parent,value.name).toBe(field.parent);}}expect(draft.fields.map(field=>({index:field.index,from:field.from,to:field.to})),value.name).toEqual(expected.fields.map(field=>({index:field.index,from:field.sourceFrom,to:field.sourceTo})));
 }
});
