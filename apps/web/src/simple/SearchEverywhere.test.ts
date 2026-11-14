import {it,expect} from "vitest";
import {everywhereTargets,searchEverywhere} from "./SearchEverywhere";
import {semanticLocations} from "./Intelligence";
it("projects only current symbols while retaining files and disabled actions",()=>{
  const file={name:"Widget.py",path:"src/Widget.py",dir:false},generated={name:"out.py",path:"out.py",dir:false,generated_by:"owner"};let source="source";
  const locations=semanticLocations([5,6,99].map((kind,i)=>({name:["Widget","render","unknown"][i],kind,range:{start:{line:0,character:i},end:{line:0,character:i}}})),"hick:///src/Widget.py");
  locations.push({uri:"hick:///foreign.py",range:{start:{line:0,character:0},end:{line:0,character:0}},name:"foreign",kind:5});
  const knowledge=[{path:file.path,source:"source",locations,state:"ready" as const,notice:""}];const commands=[{id:"symbol.rename",label:"Rename symbol",group:"Edit",reason:"Open source first.",run:()=>{}}];
  const targets=()=>everywhereTargets([file,file,generated],knowledge,()=>source,commands);
  expect(targets()).toHaveLength(6);expect(searchEverywhere(targets(),"wdgt","type").items[0]?.label).toBe("Widget");
  expect(searchEverywhere(targets(),"render","member").total).toBe(1);expect(searchEverywhere(targets(),"unknown","symbol").total).toBe(1);
  expect(searchEverywhere(targets(),"rename","action").items[0].reason).toBe("Open source first.");expect(searchEverywhere(targets(),"out.py").items[0].detail).toContain("read-only");
  expect(searchEverywhere(targets(),"","all",1).items).toHaveLength(1);expect(searchEverywhere(targets(),"","all",1).total).toBe(6);
  source="draft";expect(targets()).toHaveLength(3);expect(searchEverywhere(targets(),"","type").total).toBe(0);
});
it("fuzzy search compares whole Unicode scalars",()=>{expect(searchEverywhere([{id:"emoji",kind:"file",label:"😀x😀",detail:""}],"😀😀").total).toBe(1);});
