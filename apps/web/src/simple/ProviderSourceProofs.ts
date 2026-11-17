import type {SourceSnapshot} from './ChangePlan';
import {semanticPath} from './Intelligence';
import {sha256} from '../lib/reviews';
export interface ProviderSourceProof {uri:string;path:string;sourceHash:string}
const record=(value:unknown):value is Record<string,unknown>=>!!value&&typeof value==='object'&&!Array.isArray(value);
export function providerSourceProofs(action:unknown):ProviderSourceProof[]{
 const data=record(action)&&record(action.data)?action.data:null;
 if(!data||typeof data.hickory_action!=='string'||! /^[\x21-\x7e]{1,1024}$/.test(data.hickory_action)||!Array.isArray(data.hickory_sources)||!data.hickory_sources.length||data.hickory_sources.length>10000)throw new Error('Missing or malformed engine source proof. Prepare the provider action again.');
 const seen=new Set<string>();let previous='';return data.hickory_sources.map(raw=>{
  if(!record(raw)||Object.keys(raw).sort().join(',')!=='source_hash,uri'||typeof raw.uri!=='string'||! /^[\x21-\x7e]+$/.test(raw.uri)||typeof raw.source_hash!=='string'||! /^sha256:[a-f0-9]{64}$/.test(raw.source_hash))throw new Error('Malformed engine source proof row.');
  const path=semanticPath(raw.uri);if(!path||raw.uri<=previous||seen.has(path))throw new Error('Engine source proof identities are unsupported, duplicated or unsorted.');
  previous=raw.uri;seen.add(path);return {uri:raw.uri,path,sourceHash:raw.source_hash};
 });
}
export function providerSourceProofPaths(action:unknown):string[]{return providerSourceProofs(action).map(row=>row.path);}
export async function validateProviderSourceProofs(action:unknown,sources:ReadonlyMap<string,SourceSnapshot>,requiredExisting:ReadonlySet<string>):Promise<void>{
 const rows=providerSourceProofs(action),covered=new Set(rows.map(row=>row.path));
 for(const path of requiredExisting)if(!covered.has(path))throw new Error(`Edited source lacks engine coordinate proof: ${path}. Prepare again.`);
 for(const row of rows){const source=sources.get(row.path);if(!source)throw new Error(`Acquire and synchronize provider proof source before querying again: ${row.path}`);if('sha256:'+await sha256(source.text)!==row.sourceHash)throw new Error(`Provider source proof changed: ${row.path}. Prepare again.`);}
}
