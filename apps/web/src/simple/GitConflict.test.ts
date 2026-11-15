import {it,expect} from 'vitest';
import {GitConflictModel,validateConflict,type ConflictReading} from './GitConflict';
const reading=(path:string,text='captured'):ConflictReading=>({version:1,path,fingerprint:'a'.repeat(64),stages:[{stage:2,mode:'100644',oid:'b'.repeat(40),text,unavailable:null}],index_modified:false,source_files_modified:false});
it('retires older, closed and foreign conflict replies',async()=>{
 const held:((value:ConflictReading)=>void)[]=[];const load=()=>new Promise<ConflictReading>(resolve=>held.push(resolve));const model=new GitConflictModel();
 const first=model.read('a.txt',load),second=model.read('b.txt',load);held[1](reading('b.txt','new'));await second;held[0](reading('a.txt','old'));await first;expect(model.value?.path).toBe('b.txt');
 const closed=model.read('a.txt',load);model.close();held[2](reading('a.txt'));await closed;expect(model.value).toBeNull();
 await model.read('é.txt',async()=>reading('e\u0301.txt'));expect(model.value).toBeNull();expect(model.notice).toContain('identity changed');
});
it('requires complete immutable identities and unambiguous text availability',()=>{
 const value=reading('a.txt');expect(()=>validateConflict(value,'a.txt')).not.toThrow();
 for(const malformed of [{...value,fingerprint:'a'},{...value,index_modified:true},{...value,stages:[value.stages[0],value.stages[0]]},{...value,stages:[{...value.stages[0],oid:'main'}]},{...value,stages:[{...value.stages[0],mode:'120000'}]},{...value,stages:[{...value.stages[0],text:null}]}])expect(()=>validateConflict(malformed as ConflictReading,'a.txt')).toThrow();
 const binary=reading('a.txt');binary.stages[0]={...binary.stages[0],text:null,unavailable:'Binary'};expect(()=>validateConflict(binary,'a.txt')).not.toThrow();
});
