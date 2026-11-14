<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Index test declarations without holding the editor on browser

For maintainers replacing source discovery without coupling it to a gutter renderer.

Keep one observed source value per file. Repeated requests for the same active or successful observation do not multiply reads. A changed draft retires precise access immediately through the source equality check. Debounce pending work while retaining only its latest text.

<ui:copy id="source-test-index-browser-observations">
import type {SourceTestScan} from "../api/testSources";
export interface IndexedSourceTests {path:string;source:string;facts:SourceTestScan|null;notice:string;rows?:{line:number;candidate:SourceTestScan["candidates"][number]}[]}
type Job={path:string;source:string;due:number;current:()=>string|null};
/** Two independent reads, one pending value per path, no runner execution. */
export class SourceTestIndex {
  readonly records=new Map<string,IndexedSourceTests>();
  private pending=new Map<string,Job>();private active=new Map<string,Job>();private closed=false;
  private timer:ReturnType<typeof setTimeout>|null=null;private revision=0;private listeners=new Set<()=>void>();
  constructor(private read:(path:string,source:string)=>Promise<SourceTestScan>,private debounce=250){}
  subscribe=(listener:()=>void)=>{this.listeners.add(listener);return()=>{this.listeners.delete(listener);};};snapshot=()=>this.revision;
  private publish(){this.revision++;this.listeners.forEach(listener=>listener());}
  get inFlight(){return this.active.size;}get queued(){return this.pending.size;}
  current(path:string,source:string){const record=this.records.get(path);return record?.source===source&&record.facts?record:null;}
  enqueue(path:string,source:string,current:()=>string|null){
    if(this.closed)return;
    if(this.current(path,source)||this.active.get(path)?.source===source){this.pending.delete(path);this.pump();return;}
    if(this.pending.get(path)?.source===source)return;
    this.pending.set(path,{path,source,current,due:Date.now()+this.debounce});this.publish();this.pump();
  }
  stop(){this.closed=true;if(this.timer)clearTimeout(this.timer);this.timer=null;this.pending.clear();}
</ui:copy>

Admit at most two reads and never overlap reads of the same path. Only ready jobs consume workers; another file can progress while one fails. The supplied current-source callback and absence of a newer queued value decide whether a reply can be installed. Stop refuses new work and late replies without claiming the underlying provider is immediately terminated.

<ui:copy id="source-test-index-browser-workers">
  private pump(){
    if(this.timer)clearTimeout(this.timer);this.timer=null;if(this.closed)return;
    const now=Date.now();
    for(const job of this.pending.values()){
      if(this.active.size>=2)break;
      if(this.active.has(job.path)||job.due>now)continue;
      this.pending.delete(job.path);this.active.set(job.path,job);
      void this.perform(job);
    }
    if(this.active.size<2){const due=[...this.pending.values()].filter(job=>!this.active.has(job.path)).reduce((next,job)=>Math.min(next,job.due),Infinity);
      if(Number.isFinite(due))this.timer=setTimeout(()=>this.pump(),Math.max(0,due-Date.now()));}
  }
  private async perform(job:Job){
    let facts:SourceTestScan|null=null,notice="";
    try {facts=await this.read(job.path,job.source);if(facts.version!==1||facts.input_path!==job.path)throw new Error("Unsupported source test observation. Refresh the app and engine together.");}
    catch(error){facts=null;notice=String(error);}
    if(!this.closed&&job.current()===job.source&&!this.pending.has(job.path)){const record:IndexedSourceTests={path:job.path,source:job.source,facts,notice};record.rows=indexedTestRows(record);this.records.set(job.path,record);this.publish();}
    this.active.delete(job.path);this.pump();
  }
}
</ui:copy>

Use compiler-owned authored locations. An unmapped constructed runtime line cannot become a source gutter mark, and a foreign authored owner belongs to its own file. Plain files may use their runtime location because that is the supplied source. Validate line and UTF-16 column bounds once, and cache rows on the installed observation so each rendered gutter line does not split the whole source again. This primary-source index supplies inspection hints; opening a hint must refresh the full source-test view before offering execution against current contributors.

<ui:copy id="source-test-index-browser-locations">
/** Compiler-owned locations only. A constructed runtime line is not an authored line. */
export function indexedTestRows(record:IndexedSourceTests|null|undefined){
  if(record?.rows)return record.rows;
  if(!record?.facts)return [];
  const lines=record.source.split("\n");
  return record.facts.candidates.flatMap(candidate=>{
    const point=candidate.source??(record.facts!.scope==="plain"?candidate.target:null);
    return point?.path===record.path&&Number.isInteger(point.line)&&point.line>0&&point.line<=lines.length&&Number.isInteger(point.character)&&point.character>=0&&point.character<=lines[point.line-1].length?[{line:point.line-1,candidate}]:[];
  });
}
</ui:copy>

These focused witnesses hold real asynchronous reads to prove the two-worker bound, duplicate suppression, edit coalescing, stale refusal, independent failure and stop behavior. Location witnesses distinguish primary, foreign and missing compiler ownership. They do not execute a language runner or establish native pointer acceptance.

<ui:copy id="source-test-index-browser-witness">
import {describe,it,expect} from "vitest";
import {SourceTestIndex,indexedTestRows} from "./SourceTestIndex";
import type {SourceTestScan} from "../api/testSources";
function facts(path:string):SourceTestScan {return {version:1,workspace:"root",workspace_id:"root",input_path:path,input_hash:"hash",scope:"plain",project_entry:null,fingerprint:"scan",source_hashes:{[path]:"hash"},candidates:[],syntax_complete:true,notices:[],unsupported_languages:[],unclassified_outputs:[],scanned_files:1,unused_sources:[],project_context:null};}
const turn=async()=>{for(let i=0;i<8;i++)await Promise.resolve();};
describe("bounded source test index",()=>{
  it("coalesces edits, rejects obsolete reads and drains independently after a failure",async()=>{
    const text=new Map([["a","first"],["b","source"],["c","source"]]);
    const reads:{path:string;source:string;resolve:(value:SourceTestScan)=>void;reject:(error:Error)=>void}[]=[];
    const index=new SourceTestIndex((path,source)=>new Promise((resolve,reject)=>reads.push({path,source,resolve,reject})),0);
    const enqueue=(path:string)=>index.enqueue(path,text.get(path)!,()=>text.get(path)??null);
    enqueue("a");enqueue("a");enqueue("b");expect(index.inFlight).toBe(2);expect(reads).toHaveLength(2);
    text.set("a","middle");enqueue("a");text.set("a","latest");enqueue("a");enqueue("c");expect(index.queued).toBe(2);
    reads[0].resolve(facts("a"));await turn();expect(index.current("a","first")).toBeNull();
    expect(reads[2].source).toBe("latest");expect(index.inFlight).toBe(2);
    reads[1].reject(new Error("unavailable"));await turn();expect(index.records.get("b")?.notice).toContain("unavailable");expect(reads[3].path).toBe("c");
    reads[2].resolve(facts("a"));reads[3].resolve(facts("c"));await turn();expect(index.current("a","latest")).not.toBeNull();expect(index.current("a","changed")).toBeNull();expect(index.current("c","source")).not.toBeNull();expect(index.inFlight).toBe(0);
    enqueue("a");expect(reads).toHaveLength(4);index.stop();
  });
  it("stops admission without accepting late facts and never guesses a document's runtime line",async()=>{
    const done:((value:SourceTestScan)=>void)[]=[];const index=new SourceTestIndex(()=>new Promise(resolve=>{done.push(resolve);}),0);
    index.enqueue("a","source",()=>"source");index.enqueue("b","source",()=>"source");index.enqueue("c","source",()=>"source");index.stop();done[0](facts("a"));done[1](facts("b"));await turn();expect(index.inFlight).toBe(0);expect(index.records.size).toBe(0);expect(index.queued).toBe(0);
    const scan=facts("note.md"),point={path:"generated.py",line:2,character:4,hash:"hash"};scan.scope="document";scan.candidates=[{id:"case",language:"python",name:"test_case",run_scope:"file",name_filter:"test_case",confidence:"syntax",source:null,target:point}];
    expect(indexedTestRows({path:"note.md",source:"\nsource",facts:scan,notice:""})).toEqual([]);
    scan.candidates[0].source={...point,path:"other.md"};expect(indexedTestRows({path:"note.md",source:"\nsource",facts:scan,notice:""})).toEqual([]);
    scan.candidates[0].source={...point,path:"note.md",line:100};expect(indexedTestRows({path:"note.md",source:"\nsource",facts:scan,notice:""})).toEqual([]);
    scan.candidates[0].source={...point,path:"note.md"};expect(indexedTestRows({path:"note.md",source:"\nsource",facts:scan,notice:""})[0].line).toBe(1);
  });
});
</ui:copy>

<ui:file path="apps/web/src/simple/SourceTestIndex.ts"><ui:paste select="#source-test-index-browser-observations"/><ui:paste select="#source-test-index-browser-workers"/><ui:paste select="#source-test-index-browser-locations"/></ui:file>
<ui:file path="apps/web/src/simple/SourceTestIndex.test.ts"><ui:paste select="#source-test-index-browser-witness"/></ui:file>
</ui:doc>
