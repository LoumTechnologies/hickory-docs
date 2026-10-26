<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lineage/largePreview.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-040be5112cc9"></a>
## `apps/web/src/lineage/largePreview.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-040be5112cc9-navigate"></a>
### `{ navigate }`

<storycode:copy id="f-040be5112cc9-navigate">
import { cleanup, render, screen, fireEvent } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { buildModel, LINEAGE_PREVIEW_CHARACTERS } from "./build";
import { LineageColumns } from "./LineageColumns";
import { structuralAddition } from "./structure";
import type { SourceDocument } from "./build";
import type { OutputFile } from "../api/types";
import * as parser from "../editor/hickDoc";
const { navigate } = vi.hoisted(()=>({navigate:vi.fn()}));
</storycode:copy>
<a id="f-040be5112cc9-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-040be5112cc9-expression_statement">
vi.mock("../router",()=>({navigate}));
</storycode:copy>
<a id="f-040be5112cc9-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-040be5112cc9-expression_statement-2">
afterEach(()=>{cleanup();vi.restoreAllMocks();});
</storycode:copy>
<a id="f-040be5112cc9-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-040be5112cc9-expression_statement-3">
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
</storycode:copy>
<a id="f-040be5112cc9-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-040be5112cc9-expression_statement-4">
it("does not iterate oversized output provenance or add structural links into a limited preview",()=>{
 const provenance=new Proxy([],{get(target,key){if(key===Symbol.iterator)throw Error('Oversized provenance traversed');return Reflect.get(target,key);}}) as OutputFile['provenance'];
 const model=buildModel([{id:'d',path:'small.md',source:'# small',outputs:[{path:'large.py',content:'y'.repeat(28_000_000),language:'python',provenance}]}]);
 expect(model.files.get('large.py')?.preview).toBeTruthy();expect(model.links).toEqual([]);
 expect(structuralAddition(model,{files:[{path:'large.py',language:'python',definitions:[{name:'hidden',kind:'function',start_line:0,end_line:0}],references:[]}],links:[{from_path:'small.md',from_line:0,to_path:'large.py',to_line:0,name:'hidden',candidates:1}]}).links).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/src/lineage/largePreview.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lineage/largePreview.test.tsx"><storycode:paste select="#f-040be5112cc9-navigate"/><storycode:paste select="#f-040be5112cc9-expression_statement"/><storycode:paste select="#f-040be5112cc9-expression_statement-2"/><storycode:paste select="#f-040be5112cc9-expression_statement-3"/><storycode:paste select="#f-040be5112cc9-expression_statement-4"/></storycode:file>

</storycode:doc>
