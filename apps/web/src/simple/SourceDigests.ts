/** Source hashing is optional background work: one worker, one latest job per path. */
export class SourceDigests {
  private records=new Map<string,{source:string;hash:string}>();private pending=new Map<string,{source:string;current:()=>string|null}>();private active=false;private closed=false;
  private revision=0;private listeners=new Set<()=>void>();
  constructor(private hash:(source:string)=>Promise<string>=async source=>{const digest=await crypto.subtle.digest('SHA-256',new TextEncoder().encode(source));return [...new Uint8Array(digest)].map(byte=>byte.toString(16).padStart(2,'0')).join('');}){}
  subscribe=(listener:()=>void)=>{this.listeners.add(listener);return()=>{this.listeners.delete(listener);};};snapshot=()=>this.revision;
  current(path:string,source:string){const record=this.records.get(path);return record?.source===source?record.hash:null;}
  enqueue(path:string,source:string,current:()=>string|null){if(this.closed||this.current(path,source)!==null)return;this.pending.set(path,{source,current});void this.pump();}
  stop(){this.closed=true;this.pending.clear();}
  private async pump(){if(this.closed||this.active)return;const next=this.pending.entries().next().value;if(!next)return;const [path,job]=next;this.pending.delete(path);this.active=true;
    try {const hash=await this.hash(job.source);if(!this.closed&&job.current()===job.source&&!this.pending.has(path)){this.records.set(path,{source:job.source,hash});this.revision++;this.listeners.forEach(listener=>listener());}}
    catch{/* A failed optional digest grants no positional authority. */}
    finally{this.active=false;void this.pump();}
  }
}
