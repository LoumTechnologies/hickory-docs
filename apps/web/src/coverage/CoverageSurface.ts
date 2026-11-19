import type {CoverageDiscovery,CoverageRun} from "./types";
export interface CoverageSourceNavigation {path:string;line:number|null;sourceHash:string|null;sourceState:"current"|"historical"|"unknown";runId:string|null}
export interface CoverageSurfaceClient {
  call:<T>(body:Record<string,unknown>)=>Promise<T>;
  select:(run:CoverageRun|null)=>void;
  navigate:(location:CoverageSourceNavigation,current:()=>boolean)=>boolean|void|Promise<boolean|void>;
  retireNavigation?:()=>void;
}
export function coverageLanguage(language:string){return ["javascript","javascriptreact","typescriptreact","jsx","tsx"].includes(language)?"typescript":language;}
/** A reported line needs a captured owner hash and no observed execution-source change. */
export function coverageSourceNavigation(run:CoverageRun|null,path:string,line:number|null,discovery:CoverageDiscovery|null=null):CoverageSourceNavigation {
  const mapping=line===null?undefined:run?.mappings[path]?.[line];
  const owner=mapping?.path??path,position=mapping?.line??line;
  const listing=discovery?.version===1?discovery:null;
  const hashes=run?.identity.source_hashes??listing?.source_hashes??{};
  const stable=run?run.source_changed_during_run===false:!!listing&&listing.source_changed_during_discovery===false;
  const observed=stable?hashes[owner]??null:null;
  const hash=observed&&/^[0-9a-f]{64}$/i.test(observed)?observed.toLowerCase():null;
  const precise=!!hash&&position!==null&&Number.isInteger(position)&&position>0;
  return {path:owner,line:precise?position:null,sourceHash:hash,sourceState:run?.source_state??listing?.source_state??"unknown",runId:run?.id??null};
}
