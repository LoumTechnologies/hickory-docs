<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Project files symbols and commands into one search

For maintainers changing workspace services independently of the editor renderer.

Search Everywhere is a projection of existing workspace facts and command descriptors. File targets use the full root index, including generated files that remain read-only. Symbols contribute only while their owning source snapshot still matches a retained buffer; unsupported, stale and unavailable reads cannot masquerade as indexed declarations. LSP symbol kinds distinguish known types and members, with unknown kinds retained as general symbols. Disabled commands remain discoverable with their reason. Ranking and result limits are pure algorithms shared in behavior across renderers; the total match count is independent of the displayed cap. Selecting a symbol must still pass its captured source through the existing authoritative reveal path.

<ui:copy id="simple-search-everywhere-browser">
import type {FileNode} from "../api/types";
import type {AnalysisKnowledge} from "./WorkspaceAnalysis";
import type {WorkspaceCommand} from "./WorkspaceCommands";
import {semanticPath,type SemanticLocation} from "./Intelligence";
export type SearchKind="file"|"type"|"member"|"symbol"|"action";
export interface SearchTarget {id:string;kind:SearchKind;label:string;detail:string;path?:string;location?:SemanticLocation;source?:string;commandID?:string;reason?:string|null}
export function symbolSearchKind(kind?:number):SearchKind {return [5,10,11,23,26].includes(kind??0)?"type":[6,7,8,9,12,13,14,22,25].includes(kind??0)?"member":"symbol";}
export function everywhereTargets(files:readonly FileNode[],knowledge:readonly AnalysisKnowledge[],current:(path:string)=>string|undefined,commands:readonly WorkspaceCommand[]):SearchTarget[] {
  const targets:SearchTarget[]=[],seen=new Set<string>();
  const visit=(files:readonly FileNode[])=>{for(const file of files){if(file.dir){visit(file.children??[]);continue;}if(seen.has(file.path))continue;seen.add(file.path);targets.push({id:`file:${file.path}`,kind:"file",label:file.name,detail:file.path+(file.generated_by?" · generated, read-only":""),path:file.path});}};visit(files);
  for(const item of knowledge){if(!seen.has(item.path)||item.state!=="ready"||item.source===undefined||current(item.path)!==item.source)continue;
    for(const location of item.locations){if(semanticPath(location.uri)!==item.path||!location.name)continue;targets.push({id:`symbol:${location.uri}:${location.range.start.line}:${location.range.start.character}`,kind:symbolSearchKind(location.kind),label:location.name,detail:`${item.path}:${location.range.start.line+1}`,path:item.path,location,source:item.source});}}
  for(const command of commands)targets.push({id:`action:${command.id}`,kind:"action",label:command.label,detail:command.group,commandID:command.id,reason:command.reason});
  return [...new Map(targets.map(item=>[item.id,item])).values()];
}
function score(value:string,query:string):number|null {
  value=value.toLowerCase();if(value===query)return 100;if(value.startsWith(query))return 80;if(value.includes(query))return 60;
  const scalars=Array.from(query);let cursor=0;for(const scalar of value){if(scalar===scalars[cursor])cursor++;if(cursor===scalars.length)return 20;}return null;
}
export function searchEverywhere(targets:readonly SearchTarget[],query:string,kind:SearchKind|"all"="all",limit=200) {
  const terms=query.toLowerCase().trim().split(/\s+/).filter(Boolean);
  const matches=targets.flatMap(target=>{if(kind!=="all"&&target.kind!==kind)return[];let total=0;for(const term of terms){const label=score(target.label,term),detail=score(target.detail,term);const best=Math.max(label??-1,detail===null?-1:detail/2);if(best<0)return[];total+=best;}return[{target,score:total}];});
  matches.sort((a,b)=>b.score-a.score||(a.target.label<b.target.label?-1:a.target.label>b.target.label?1:0)||(a.target.id<b.target.id?-1:a.target.id>b.target.id?1:0));
  return {items:matches.slice(0,Math.max(0,Math.min(1000,limit))).map(item=>item.target),total:matches.length};
}
</ui:copy>

<ui:file path="apps/web/src/simple/SearchEverywhere.ts"><ui:paste select="#simple-search-everywhere-browser"/></ui:file>
</ui:doc>
