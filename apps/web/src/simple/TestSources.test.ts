import {describe,it,expect} from "vitest";
import {testSourceCurrent,sourceTestRun,type TestSourceSnapshot} from "./TestSources";
import type {SourceTestScan} from "../api/testSources";
const before:TestSourceSnapshot={workspace:"w1",request:{path:"cases.md",source:"draft",project_entry:"project.md",sources:{"helper.md":"helper","unused.md":"old"}}};
const point={path:"test.py",line:1,character:4,hash:"target"};
const facts:SourceTestScan={version:1,workspace:"root",workspace_id:"w1",input_hash:"draft-hash",scope:"project",project_entry:"project.md",syntax_complete:true,notices:[],unsupported_languages:[],unclassified_outputs:[],scanned_files:1,unused_sources:["unused.md"],project_context:null,input_path:"cases.md",fingerprint:"scan",source_hashes:{"cases.md":"draft-hash","helper.md":"helper-hash"},candidates:[{id:"case",name:"test_case",language:"python",run_scope:"file",name_filter:"test_case",confidence:"syntax",source:null,target:point}]};
describe("source test observations",()=>{
  it("retires changed contributors and workspace identity while ignoring unrelated drafts",()=>{
    expect(testSourceCurrent(before,facts,before)).toBe(true);
    expect(testSourceCurrent(before,facts,{...before,workspace:"w2"})).toBe(false);
    expect(testSourceCurrent(before,facts,{...before,request:{...before.request,source:"changed"}})).toBe(false);
    expect(testSourceCurrent(before,facts,{...before,request:{...before.request,sources:{"helper.md":"changed"}}})).toBe(false);
    expect(testSourceCurrent(before,facts,{...before,request:{...before.request,sources:{"helper.md":"helper","unused.md":"new"}}})).toBe(true);
    expect(testSourceCurrent(before,{...facts,version:2} as unknown as SourceTestScan,before)).toBe(false);
  });
  it("requires explicit supported selection and keeps saved execution bound to its facts",()=>{
    const request=sourceTestRun(before,facts,before,"case","name_filter");
    expect(request).toMatchObject({input:{kind:"saved"},fingerprint:"scan",candidate_id:"case",analysis:before.request});
    const dynamic={...facts,candidates:[{...facts.candidates[0],name_filter:null}]};
    expect(()=>sourceTestRun(before,dynamic,before,"case","name_filter")).toThrow("discovery");
    expect(sourceTestRun(before,dynamic,before,"case","containing_scope").selection.kind).toBe("containing_scope");
    expect(()=>sourceTestRun(before,facts,{...before,workspace:"changed"},"case","containing_scope")).toThrow("changed");
    expect(()=>sourceTestRun(before,facts,before,"missing","name_filter")).toThrow("unavailable");
  });
});
