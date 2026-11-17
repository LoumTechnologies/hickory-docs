import {expect,it} from 'vitest';
import {sha256} from '../lib/reviews';
import {providerSourceProofPaths,providerSourceProofs,validateProviderSourceProofs} from './ProviderSourceProofs';
const hash='sha256:'+'0'.repeat(64);
const action=(rows:unknown)=>({data:{hickory_action:'opaque:1',hickory_sources:rows}});
const row=(uri='hick:///a',source_hash=hash)=>({uri,source_hash});
it('discovers sorted proof context without granting edit authority',()=>{expect(providerSourceProofPaths(action([row(),row('hick:///b')]))).toEqual(['a','b']);});
it('refuses missing stamps and malformed proof schema, hashes, order and identities',()=>{
 for(const value of [{},action([]),action([row('file:///a')]),action([row('hick:///a?x')]),action([row('hick:///../a')]),action([row('hick:///a','SHA256:'+'0'.repeat(64))]),action([row('hick:///a','sha256:'+'A'.repeat(64))]),action([{...row(),extra:true}]),action([row(),row()]),action([row('hick:///b'),row()]),action([row('hick:///%61'),row()])])expect(()=>providerSourceProofs(value)).toThrow();
});
it('validates readonly context and edited preimages against frozen UTF8 bytes',async()=>{
 const original={text:'é😀',writable:false},proof=action([row('hick:///a','sha256:'+await sha256(original.text))]);await validateProviderSourceProofs(proof,new Map([['a',original]]),new Set(['a']));
 await expect(validateProviderSourceProofs(proof,new Map([['a',{text:'e\u0301😀',writable:false}]]),new Set())).rejects.toThrow('changed');
 await expect(validateProviderSourceProofs(proof,new Map(),new Set())).rejects.toThrow('Acquire');
 await expect(validateProviderSourceProofs(proof,new Map([['a',original]]),new Set(['created-only-if-misclassified']))).rejects.toThrow('lacks');
});
it('does not require proof for explicitly excluded created destinations',async()=>{const proof=action([row('hick:///a','sha256:'+await sha256('before'))]);await validateProviderSourceProofs(proof,new Map([['a',{text:'before',writable:true}]]),new Set(['a']));});
