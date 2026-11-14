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
