import {expect,it} from 'vitest';
import {validateSourcePreparation} from './WorkspaceLanguages';
const hash='sha256:'+'a'.repeat(64);
const ack={processed:true,current:true,source_files_modified:false,source_hash:hash,source_revision:'b'.repeat(64),parsed:false};
it('admits confirmed plain text and requires a parsed literate graph',()=>{expect(()=>validateSourcePreparation(ack,hash,false)).not.toThrow();expect(()=>validateSourcePreparation(ack,hash,true)).toThrow('exact source');expect(()=>validateSourcePreparation({...ack,parsed:true},hash,true)).not.toThrow();});
it('refuses absent, forged, stale and effectful preparation receipts',()=>{for(const value of [null,{},...['processed','current'].flatMap(key=>[false,1,'true'].map(value=>({...ack,[key]:value}))),...['source_files_modified'].flatMap(key=>[true,0,'false'].map(value=>({...ack,[key]:value}))),{...ack,source_hash:'sha256:'+'c'.repeat(64)},{...ack,source_revision:''},{...ack,source_revision:'B'.repeat(64)}])expect(()=>validateSourcePreparation(value,hash,false)).toThrow('exact source');});
