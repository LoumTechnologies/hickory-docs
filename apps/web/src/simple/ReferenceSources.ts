import type {SourceSnapshot} from './ChangePlan';
/** Discovery never authorizes edits. Newly found targets synchronize before a fresh query. */
export async function acquireSourceProposal<T>(initial:ReadonlyMap<string,SourceSnapshot>,query:()=>Promise<{paths:string[];value:T}>,acquire:(path:string)=>Promise<SourceSnapshot>,current:()=>boolean){
 const sources=new Map(initial);let references:string[]=[];
 const check=()=>{if(!current())throw new Error('Source, selection or provider changed. Prepare again.');};
 for(let round=0;round<4;round++){
  check();const proposal=await query();references=[...new Set(proposal.paths)];check();if(references.length>10000)throw new Error('Reference discovery exceeds 10,000 source targets. Narrow the change.');
  const missing=references.filter(path=>!sources.has(path));if(!missing.length)return {sources,paths:references,value:proposal.value};
  let cursor=0,failure:unknown=null;
  const worker=async()=>{while(failure===null&&cursor<missing.length){if(!current()){failure=new Error('Source, selection or provider changed. Prepare again.');break;}const path=missing[cursor++];
   try{const value=await acquire(path);check();if(!value.writable)throw new Error(`Source is generated, read-only or saving: ${path}`);sources.set(path,value);}catch(error){failure=error;}
  }};
  await Promise.all([worker(),worker()]);if(failure!==null)throw failure;check();
 }
 throw new Error('Reference targets kept expanding. Refresh project analysis and prepare again. No edits were applied.');
}
export async function acquireReferenceSources(initial:ReadonlyMap<string,SourceSnapshot>,query:()=>Promise<string[]>,acquire:(path:string)=>Promise<SourceSnapshot>,current:()=>boolean){
 const result=await acquireSourceProposal(initial,async()=>({paths:await query(),value:null}),acquire,current);
 return {sources:result.sources,references:result.paths};
}
