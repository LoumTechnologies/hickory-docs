import {describe,it,expect} from 'vitest';
import cases from '../../../simple-contract/snippet-transform-cases.json';
import {readSnippetTransform,applySnippetTransform} from './SnippetTransform';
describe('shared bounded snippet transforms',()=>{
 for(const row of cases)it(row.name,()=>{const program=readSnippetTransform(row.program);if('reject' in row&&row.reject)expect(()=>applySnippetTransform(program,row.input)).toThrow();else expect(applySnippetTransform(program,row.input)).toBe('expected' in row?row.expected:undefined);});
 it('refuses malformed descriptors and shares an action work budget',()=>{for(const raw of [{...cases[0].program,version:1},{...cases[0].program,case_map:"other"},{...cases[0].program,groups:true},{...cases[0].program,flags:'gg'},{...cases[0].program,regex:['class',[[5,1]],false]},{...cases[0].program,regex:['capture',true,['dot']]}])expect(()=>readSnippetTransform(raw)).toThrow();const program=readSnippetTransform(cases[0].program);expect(()=>applySnippetTransform(program,'foo_bar',{work:99999})).toThrow();expect(Object.isFrozen(program.regex)).toBe(true);for(const input of ['\ud800','\udfff'])expect(()=>applySnippetTransform(program,input)).toThrow();expect(()=>readSnippetTransform({...cases[0].program,format:['\ud800']})).toThrow();});
});
