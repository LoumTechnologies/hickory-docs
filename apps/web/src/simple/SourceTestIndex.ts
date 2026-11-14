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
