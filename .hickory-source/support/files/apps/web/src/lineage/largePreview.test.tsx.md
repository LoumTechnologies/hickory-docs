<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lineage/largePreview.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `da28f5a956099b995bd8e7e7d468f9a3d1cd82d5fd4e66f4fd4aaaa037f02470`.

<book:entry path="apps/web/src/lineage/largePreview.test.tsx">
import { cleanup, render, screen, fireEvent } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { buildModel, LINEAGE_PREVIEW_CHARACTERS } from "./build";
import { LineageColumns } from "./LineageColumns";
import { structuralAddition } from "./structure";
import type { SourceDocument } from "./build";
import type { OutputFile } from "../api/types";
import * as parser from "../editor/hickDoc";
const { navigate } = vi.hoisted(()=>({navigate:vi.fn()}));
vi.mock("../router",()=>({navigate}));
afterEach(()=>{cleanup();vi.restoreAllMocks();});
it("bounds a28MB encoded source in model and DOM without parsing it or inventing links",()=>{
 const parse=vi.spyOn(parser,"parseStructure");
 const source='<hick:copy id="hidden">'+'x'.repeat(28_000_000)+'</hick:copy>';
 const doc:SourceDocument={id:'registered-large',path:'sessions/large.md',source,outputs:[{path:'result.txt',language:'',content:'x',provenance:[{start:0,end:1,origin:{kind:'literal',doc_path:'sessions/large.md',span:[22,23]}}]}]};
 const model=buildModel([doc]);expect(parse).not.toHaveBeenCalled();
 expect(doc.source).toBe(source);expect(model.stages).toHaveLength(1);
 expect(model.files.get(doc.path)?.preview?.totalCharacters).toBe(source.length);
 expect(model.files.get(doc.path)!.lines.join('\n').length).toBeLessThanOrEqual(LINEAGE_PREVIEW_CHARACTERS);
 expect(model.links).toEqual([]);expect(model.nodes.size).toBe(0);
 const view=render(<LineageColumns model={model}/>);
 expect(view.container.textContent!.length).toBeLessThan(30_000);
 expect(screen.getByText(/Block and provenance links involving this file are omitted/)).toBeTruthy();
 fireEvent.click(screen.getByRole('button',{name:'Open full document'}));
 expect(navigate).toHaveBeenCalledWith('/docs/registered-large');
});
it("does not iterate oversized output provenance or add structural links into a limited preview",()=>{
 const provenance=new Proxy([],{get(target,key){if(key===Symbol.iterator)throw Error('Oversized provenance traversed');return Reflect.get(target,key);}}) as OutputFile['provenance'];
 const model=buildModel([{id:'d',path:'small.md',source:'# small',outputs:[{path:'large.py',content:'y'.repeat(28_000_000),language:'python',provenance}]}]);
 expect(model.files.get('large.py')?.preview).toBeTruthy();expect(model.links).toEqual([]);
 expect(structuralAddition(model,{files:[{path:'large.py',language:'python',definitions:[{name:'hidden',kind:'function',start_line:0,end_line:0}],references:[]}],links:[{from_path:'small.md',from_line:0,to_path:'large.py',to_line:0,name:'hidden',candidates:1}]}).links).toEqual([]);
});
</book:entry>

</book:doc>
