<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lsp/representationMapping.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

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

<storycode:copy id="f-237646581124-representationOffset">
export function representationOffset(source: string, file: RepresentedFile, position: LspPosition): number | null {
  const at = charToByte(file.content, positionToUtf16(file.content, position));
  const matches = file.provenance.filter(p => at >= p.start && at < p.end && ["literal", "paste", "ingested"].includes(p.origin.kind) && "span" in p.origin && p.origin.span && (!file.source_path || ("doc_path" in p.origin && p.origin.doc_path === file.source_path)) && p.end - p.start === p.origin.span[1] - p.origin.span[0]);
  if (!matches.length && at === charToByte(file.content, file.content.length)) matches.push(...file.provenance.filter(p => p.end === at && ["literal", "paste", "ingested"].includes(p.origin.kind) && "span" in p.origin && p.origin.span && (!file.source_path || ("doc_path" in p.origin && p.origin.doc_path === file.source_path)) && p.end - p.start === p.origin.span[1] - p.origin.span[0]));
  if (matches.length !== 1) return null;
  const p = matches[0];
  if (!("span" in p.origin) || !p.origin.span) return null;
  const delta = at - p.start;
  if (delta > p.origin.span[1] - p.origin.span[0]) return null;
  return byteToChar(source, p.origin.span[0] + delta);
}

</storycode:copy>
<a id="f-237646581124-representationRange"></a>
### `representationRange`

An edit can span fragments only when their source bytes are contiguous.

<storycode:copy id="f-237646581124-representationRange">
/** An edit can span fragments only when their source bytes are contiguous. */
export function representationRange(source:string, file:RepresentedFile, range:{start:LspPosition;end:LspPosition}): {from:number;to:number}|null {
  const start=charToByte(file.content,positionToUtf16(file.content,range.start));
  const end=charToByte(file.content,positionToUtf16(file.content,range.end));
  if (end<start) return null;
  if (start===end) { const from=representationOffset(source,file,range.start); return from===null?null:{from,to:from}; }
  let at=start, first:number|null=null, previous:number|null=null;
  for (const p of file.provenance) {
    if(p.end<=at || p.start>=end) continue;
    if(p.start>at || !["literal","paste","ingested"].includes(p.origin.kind) || !("span" in p.origin) || !p.origin.span || p.end-p.start!==p.origin.span[1]-p.origin.span[0] || (file.source_path && (!("doc_path" in p.origin)||p.origin.doc_path!==file.source_path))) return null;
    const from=p.origin.span[0]+at-p.start;
    if(previous!==null && previous!==from) return null;
    if(first===null) first=from;
    const until=Math.min(end,p.end); previous=from+until-at; at=until;
    if(at===end) break;
  }
  if(at!==end || first===null || previous===null) return null;
  const from=byteToChar(source,first),to=byteToChar(source,previous);
  return source.slice(from,to)===file.content.slice(byteToChar(file.content,start),byteToChar(file.content,end))?{from,to}:null;
}
</storycode:copy>

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

<storycode:copy id="f-f3e0210e3ed7-expression_statement">
it("maps reordered literate fragments to real files, including Unicode and boundaries",()=>{
  const first="é = 1\n", second="other = 2\n";
  const source=`# Explain\n${second}# Earlier\n${first}`;
  const a=source.indexOf(first), b=source.indexOf(second);
  const file:RepresentedFile={path:"a.py",hash:"",content:first+second,provenance:[
    {start:0,end:bytes(first),origin:{kind:"paste",doc_path:"virtual",span:[bytes(source.slice(0,a)),bytes(source.slice(0,a+first.length))]}},
    {start:bytes(first),end:bytes(first+second),origin:{kind:"paste",doc_path:"virtual",span:[bytes(source.slice(0,b)),bytes(source.slice(0,b+second.length))]}}
  ]};
  expect(filePosition(source,file,a+2)).toEqual({line:0,character:2});
  expect(filePosition(source,file,b)).toEqual({line:1,character:0});
  expect(representationOffset(source,file,{line:1,character:0})).toBe(b);
  expect(representationOffset(source,file,{line:0,character:2})).toBe(a+2);
  expect(filePosition(source,file,0)).toBeNull();
  expect(filePosition(source,{...file,source_path:"other-document"},a)).toBeNull();
  expect(representationRange(source,file,{start:{line:0,character:0},end:{line:1,character:9}})).toBeNull();
  expect(representationRange(source,file,{start:{line:0,character:0},end:{line:0,character:1}})).toEqual({from:a,to:a+1});
});
</storycode:copy>
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
