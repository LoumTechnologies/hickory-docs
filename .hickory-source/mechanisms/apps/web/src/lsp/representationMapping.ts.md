<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Translate navigation and edits through provenance

For engineers changing this behavior and its focused safety checks.

Generated-file positions and Markdown positions are different coordinate spaces. Map only ranges justified by provenance, and keep navigation permission separate from editing permission: an ingested byte has a location but is not authored source. Remaining fragment-level explanation gaps are recorded in the review ledger.

<a id="output-237646581124"></a>
## `apps/web/src/lsp/representationMapping.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A comparison keeps current code editable](../../../../../guarantees/editing/a-comparison-keeps-current-code-editable/contract.md).

<a id="f-237646581124-RepresentedFile"></a>
### `RepresentedFile`

<storycode:copy id="f-237646581124-RepresentedFile">
import type { Representation } from "../api/representations";
import { byteToChar, charToByte } from "../lib/offsets";
import { positionToUtf16, utf16ToPosition, type LspPosition } from "./positions";

export type RepresentedFile = Representation["files"][number];
</storycode:copy>
<a id="f-237646581124-filePosition"></a>
### `filePosition`

Only exact literal/paste mappings are writable. Never guess across a gap.

<storycode:copy id="f-237646581124-filePosition">
// Only exact literal/paste mappings are writable. Never guess across a gap.
export function filePosition(source: string, file: RepresentedFile, offset: number): LspPosition | null {
  const at = charToByte(source, offset);
  const matches = file.provenance.filter(p => ["literal", "paste", "ingested"].includes(p.origin.kind) && "span" in p.origin && p.origin.span && (!file.source_path || ("doc_path" in p.origin && p.origin.doc_path === file.source_path)) && p.end - p.start === p.origin.span[1] - p.origin.span[0] && at >= p.origin.span[0] && at < p.origin.span[1]);
  if (!matches.length) matches.push(...file.provenance.filter(p => ["literal", "paste", "ingested"].includes(p.origin.kind) && "span" in p.origin && p.origin.span && (!file.source_path || ("doc_path" in p.origin && p.origin.doc_path === file.source_path)) && at === p.origin.span[1] && p.end - p.start === p.origin.span[1] - p.origin.span[0]));
  if (matches.length !== 1) return null;
  const p = matches[0];
  if (!("span" in p.origin) || !p.origin.span) return null;
  const delta = at - p.origin.span[0];
  if (delta > p.end - p.start) return null;
  return utf16ToPosition(file.content, byteToChar(file.content, p.start + delta));
}
</storycode:copy>
<a id="f-237646581124-representationOffset"></a>
### `representationOffset`

The tutorial builds this part in [the construction step](../../../../../04-editing.md#f-237646581124-representationOffset).

<a id="f-237646581124-representationRange"></a>
### `representationRange`

An edit can span fragments only when their source bytes are contiguous.

The tutorial builds this part in [the construction step](../../../../../04-editing.md#f-237646581124-representationRange).


### Assemble `apps/web/src/lsp/representationMapping.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/representationMapping.ts"><storycode:paste select="#f-237646581124-RepresentedFile"/><storycode:paste select="#f-237646581124-filePosition"/><storycode:paste select="#f-237646581124-representationOffset"/><storycode:paste select="#f-237646581124-representationRange"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-f3e0210e3ed7"></a>
## `apps/web/src/lsp/representationMapping.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A comparison keeps current code editable](../../../../../guarantees/editing/a-comparison-keeps-current-code-editable/contract.md).

<a id="f-f3e0210e3ed7-bytes"></a>
### `bytes`

<storycode:copy id="f-f3e0210e3ed7-bytes">
// Guarantee: editing/a-comparison-keeps-current-code-editable.md
import { expect, it } from "vitest";
import { filePosition, representationOffset, representationRange, type RepresentedFile } from "./representationMapping";
const bytes=(s:string)=>new TextEncoder().encode(s).length;
</storycode:copy>
<a id="f-f3e0210e3ed7-expression_statement"></a>
### `expression_statement`

The tutorial builds this part in [the construction step](../../../../../04-editing.md#f-f3e0210e3ed7-expression_statement).

<a id="f-f3e0210e3ed7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-f3e0210e3ed7-expression_statement-2">
it("refuses shared origins and computed coordinates rather than mapping to another occurrence",()=>{
  const file:RepresentedFile={path:"a.py",hash:"",content:"aaa",provenance:[
    {start:0,end:1,origin:{kind:"literal",doc_path:"virtual",span:[0,1]}},
    {start:1,end:2,origin:{kind:"literal",doc_path:"virtual",span:[0,1]}},
    {start:2,end:3,origin:{kind:"synthetic"}}
  ]};
  expect(filePosition("a",file,0)).toBeNull();
  expect(representationOffset("a",file,{line:0,character:2})).toBeNull();
});
</storycode:copy>

### Assemble `apps/web/src/lsp/representationMapping.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/representationMapping.test.ts"><storycode:paste select="#f-f3e0210e3ed7-bytes"/><storycode:paste select="#f-f3e0210e3ed7-expression_statement"/><storycode:paste select="#f-f3e0210e3ed7-expression_statement-2"/></storycode:file>


</storycode:doc>
