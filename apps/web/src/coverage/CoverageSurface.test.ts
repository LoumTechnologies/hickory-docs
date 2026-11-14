import {expect,it} from "vitest";
import {coverageSourceNavigation,coverageLanguage} from "./CoverageSurface";
import type {CoverageDiscovery,CoverageRun} from "./types";
function run():CoverageRun {return {id:"record",mode:"automated",status:"completed",started_ms:0,project:".",language:"python",labels:[],group:null,source_state:"historical",identity:{collector:"provider",version:"1",configuration:"config",scope:".",source_hashes:{"app.py":"a".repeat(64),"note.md":"b".repeat(64)}},argv:[],output:"",error:null,exit_code:0,tests:[],evidence:null,metadata:{groups:{},manual:{},tests:{}},source_changed_during_run:false,manual_verdict:null,notes:"",checkpoints:[],parent:null,mappings:{"app.py":{1:{path:"note.md",line:4}}}};}
it("normalizes one collector family without pretending unsupported languages have a collector",()=>{expect(coverageLanguage("typescriptreact")).toBe("typescript");expect(coverageLanguage("go")).toBe("go");});
it("binds mapped owner locations to their own captured bytes, independently of global history",()=>{
  const record=run();expect(coverageSourceNavigation(record,"app.py",1)).toMatchObject({path:"note.md",line:4,sourceHash:"b".repeat(64),sourceState:"historical"});
  delete record.identity.source_hashes["note.md"];expect(coverageSourceNavigation(record,"app.py",1)).toMatchObject({path:"note.md",line:null,sourceHash:null});
  expect(coverageSourceNavigation(record,"app.py",null).line).toBeNull();
  record.identity.source_hashes["app.py"]="malformed";expect(coverageSourceNavigation(record,"app.py",2).line).toBeNull();
  record.source_changed_during_run=true;expect(coverageSourceNavigation(record,"app.py",2).line).toBeNull();
});
it("keeps changed or unversioned runner listings readable without positional authority",()=>{
  const discovery:CoverageDiscovery={version:1,tests:[],output:"",metadata:{groups:{},manual:{},tests:{}},project:".",language:"python",discovery:"runner",source_hashes:{"test.py":"c".repeat(64)},source_state:"current",source_changed_during_discovery:false,source_frozen:false,workspace_id:"workspace",fingerprint:"listing"};
  expect(coverageSourceNavigation(null,"test.py",2,discovery)).toMatchObject({line:2,sourceHash:"c".repeat(64)});
  discovery.source_changed_during_discovery=true;expect(coverageSourceNavigation(null,"test.py",2,discovery).line).toBeNull();
  expect(coverageSourceNavigation(null,"test.py",2,{...discovery,version:2} as unknown as CoverageDiscovery).sourceState).toBe("unknown");
  expect(coverageSourceNavigation(null,"test.py",-1,{...discovery,source_changed_during_discovery:false}).line).toBeNull();
});
