import {it,expect} from 'vitest';
import {ConstructionReadingModel,constructionPointOffset,type ConstructionReading} from './ConstructionReading';
const value=(entry='project.md'):ConstructionReading=>({version:1,entry,focus:null,materialized:false,outputs:[],output_count:0,outputs_truncated:false,tests:[],test_count:0,tests_truncated:false,test_relationship:'declaration',evidence_status:'not_queried',notices:[],unused_sources:[]});
it('retires superseded and closed compiler readings and rejects foreign identities',async()=>{
 const pending:((value:ConstructionReading)=>void)[]=[];const model=new ConstructionReadingModel(()=>new Promise(resolve=>pending.push(resolve)));const input={entry:'project.md',focus:null,sources:{}};
 const old=model.read(input),latest=model.read(input);pending[1]({...value(),notices:['new']});await latest;pending[0]({...value(),notices:['old']});await old;expect(model.value?.notices).toEqual(['new']);
 const closed=model.read(input);model.close();pending[2](value());await closed;expect(model.value).toBeNull();
 const foreign=model.read(input);pending[3](value('other.md'));await foreign;expect(model.value).toBeNull();expect(model.notice).toContain('identity changed');
 const written=model.read(input);pending[4]({...value(),materialized:true});await written;expect(model.value).toBeNull();
});
it('captures request identity before callers change their request object',async()=>{
 let finish!:(value:ConstructionReading)=>void;const model=new ConstructionReadingModel(()=>new Promise(resolve=>finish=resolve));const input={entry:'project.md',focus:null,sources:{'a.md':'one'}};const read=model.read(input);input.entry='other.md';input.sources['a.md']='two';finish(value());await read;expect(model.value?.entry).toBe('project.md');
});

it('requires exact source hashes and scalar UTF-16 boundaries and supersedes older links',()=>{
 const source='😀e\u0301\né😀code',hash='a'.repeat(64),point={path:'part.md',line:2,character:3,hash};
 expect(constructionPointOffset(source,point,hash)).toBe(source.indexOf('code'));
 expect(constructionPointOffset(source,{...point,character:2},hash)).toBeNull();
 expect(constructionPointOffset(source,{...point,line:0},hash)).toBeNull();
 expect(constructionPointOffset(source,{...point,character:99},hash)).toBeNull();
 expect(constructionPointOffset(source,point,'b'.repeat(64))).toBeNull();
 const model=new ConstructionReadingModel(async()=>value());model.value=value();const old=model.sourceLink(model.value),latest=model.sourceLink(model.value);expect(old()).toBe(false);expect(latest()).toBe(true);model.close();expect(latest()).toBe(false);
});
