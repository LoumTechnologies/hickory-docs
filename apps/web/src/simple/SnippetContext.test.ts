import {describe,it,expect,beforeAll,vi} from 'vitest';
import {resolveSnippetCompletion,captureSnippetClipboard} from './SnippetContext';
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
const source='λ😀\r\nword';
const hash='sha256:b780b85cc61fcebbfa18870c45ac77760d59272f59d6b40db68250fb081f8452';
const proof={source_hash:hash,source_revision:'a'.repeat(64)};
const initial=():{label:string;data:Record<string,unknown>}=>({label:'field',data:{hickory_completion:'original-token',hickory_context:proof,opaque:{provider:['untouched']}}});
const requested=(token='resolved-token',p=proof)=>({label:'field',data:{hickory_completion:token,hickory_context:p,hickory_context_request:{version:1,variables:['CLIPBOARD']}}});
describe('source-bound completion variable handshake',()=>{
 it('bounds clipboard validation before encoding or walking an excessive value',async()=>{const original=globalThis.TextEncoder;try{vi.stubGlobal('TextEncoder',class{encode(){throw Error('Clipboard must not allocate an encoded copy');}});for(const value of['x'.repeat(1048577),'😀'.repeat(300000),'\uD800']){vi.stubGlobal('navigator',{clipboard:{readText:async()=>value}});expect(await captureSnippetClipboard()).toEqual({available:false,text:null});}vi.stubGlobal('navigator',{clipboard:{readText:async()=>'x'.repeat(1048576)}});expect((await captureSnippetClipboard()).available).toBe(true);}finally{vi.unstubAllGlobals();globalThis.TextEncoder=original;}});
 it('reads clipboard once on demand, keeps clock and seed, and leaves the original opaque item unchanged',async()=>{
  const original=initial(),before=JSON.stringify(original),calls:any[]=[];let reads=0;
  const final=await resolveSnippetCompletion(original,{source,from:5,to:9,current:()=>true,clipboard:async()=>{reads++;return {available:true,text:'clipboard text'};},request:async item=>{calls.push(item);return (calls.length===1?requested():{label:'done',data:{hickory_context:proof}}) as typeof item;}});
  expect(final.label).toBe('done');expect(reads).toBe(1);expect(calls).toHaveLength(2);expect(JSON.stringify(original)).toBe(before);
  const first=calls[0].data.hickory_context,second=calls[1].data.hickory_context;
  expect(first.expected).toEqual(proof);expect(first.selection).toEqual({from:5,to:9});expect(first.clipboard).toBe(null);expect(second.clipboard).toEqual({available:true,text:'clipboard text'});
  expect(second.clock).toEqual(first.clock);expect(second.random_hex).toBe(first.random_hex);expect(calls[1].data.hickory_completion).toBe('resolved-token');
 });
 it('does not request or read unavailable context when no clipboard variable was requested',async()=>{
  let reads=0,calls=0;await resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>true,clipboard:async()=>{reads++;return {available:false,text:null};},request:async item=>{calls++;return {label:'done',data:{hickory_context:proof}} as typeof item;}});expect(calls).toBe(1);expect(reads).toBe(0);
 });
 it('preserves explicit clipboard denial for the second resolve',async()=>{
  const calls:any[]=[];await resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>true,clipboard:async()=>({available:false,text:null}),request:async item=>{calls.push(item);return (calls.length===1?requested():{label:'done',data:{hickory_context:proof}}) as typeof item;}});expect(calls[1].data.hickory_context.clipboard).toEqual({available:false,text:null});
 });
 it('rejects foreign source proofs before privacy-sensitive clipboard access',async()=>{
  for(const p of [{...proof,source_hash:'sha256:'+'b'.repeat(64)},{...proof,source_revision:'b'.repeat(64)}]){let reads=0;await expect(resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>true,clipboard:async()=>{reads++;return {available:true,text:'secret'};},request:async()=>requested('foreign',p) as any})).rejects.toThrow('source context changed');expect(reads).toBe(0);}
 });
 it('rejects malformed or repeated requests without another clipboard read',async()=>{
  for(const request of [{version:2,variables:['CLIPBOARD']},{version:1,variables:['PATH']},{version:1,variables:['CLIPBOARD'],extra:true}]){let reads=0;await expect(resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>true,clipboard:async()=>{reads++;return {available:true,text:'secret'};},request:async()=>({...requested(),data:{...requested().data,hickory_context_request:request}}) as any})).rejects.toThrow('Unsupported');expect(reads).toBe(0);}
  let reads=0,calls=0;await expect(resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>true,clipboard:async()=>{reads++;return {available:true,text:'secret'};},request:async()=>{calls++;return requested() as any;}})).rejects.toThrow('repeated');expect(reads).toBe(1);expect(calls).toBe(2);
 });
 it('retires before another request when source or cursor changes during clipboard access',async()=>{
  let current=true,calls=0;await expect(resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>current,clipboard:async()=>{current=false;return {available:true,text:'secret'};},request:async()=>{calls++;return requested() as any;}})).rejects.toThrow('Source, cursor');expect(calls).toBe(1);
 });
 it('retires source changes while the provider is resolving before clipboard access',async()=>{
  let current=true,reads=0;await expect(resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>current,clipboard:async()=>{reads++;return {available:true,text:'secret'};},request:async()=>{current=false;return requested() as any;}})).rejects.toThrow('Source, cursor');expect(reads).toBe(0);
 });
});
