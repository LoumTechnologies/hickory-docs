import type {SourceSnapshot} from './ChangePlan';
/** Discovery never authorizes edits. Every discovered target synchronizes before a fresh query. */
export async function acquireSourceProposal<T>(initial:ReadonlyMap<string,SourceSnapshot>,query:()=>Promise<{paths:string[];value:T}>,acquire:(path:string)=>Promise<SourceSnapshot>,current:()=>boolean,requireWritable:(path:string)=>boolean=()=>true){
 const sources=new Map(initial),synchronized=new Set<string>();let references:string[]=[];
 const check=()=>{if(!current())throw new Error('Source, selection or provider changed. Prepare again.');};
 for(let round=0;round<4;round++){
  check();const proposal=await query();references=[...new Set(proposal.paths)];check();if(references.length>10000)throw new Error('Reference discovery exceeds 10,000 source targets. Narrow the change.');
  const pending=references.filter(path=>!synchronized.has(path));if(!pending.length)return {sources,paths:references,value:proposal.value};
  for(const path of pending){
   check();const value=await acquire(path);check();const original=sources.get(path);
   if(original&&(original.text!==value.text||original.writable!==value.writable))throw new Error(`Source changed while synchronizing: ${path}. Prepare again.`);
   if(requireWritable(path)&&!value.writable)throw new Error(`Source is generated, read-only or saving: ${path}`);
   if(!original)sources.set(path,value);synchronized.add(path);
  }
  check();
 }
 throw new Error('Reference targets kept expanding. Refresh project analysis and prepare again. No edits were applied.');
}
export async function acquireReferenceSources(initial:ReadonlyMap<string,SourceSnapshot>,query:()=>Promise<string[]>,acquire:(path:string)=>Promise<SourceSnapshot>,current:()=>boolean,requireWritable:(path:string)=>boolean=()=>true){
 const result=await acquireSourceProposal(initial,async()=>({paths:await query(),value:null}),acquire,current,requireWritable);
 return {sources:result.sources,references:result.paths};
}
