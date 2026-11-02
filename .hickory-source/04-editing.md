<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# 4. Connect the document and generated-file editors

We can construct and replay files. Now we need an editor that can show those
files and return an action to the authored Markdown. We will map navigation first,
then add the stricter conditions needed to apply an edit.

## Navigate from a generated position to its source

Convert the language server's position to an output byte offset, then require a
single matching provenance span. Translate its byte delta back to an editor
character offset. Captured bytes still have a source location even though they
are not editable.

<a id="f-237646581124-representationOffset"></a>
<book:copy id="f-237646581124-representationOffset">
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

</book:copy>

A missing or ambiguous origin returns `null`; guessing would send the reader to
an unrelated occurrence.

## Require a contiguous authored destination for edits

Navigation can point into an immutable capture; an edit cannot. Refuse any range
that touches captured bytes. For authored ranges, require complete provenance
coverage, contiguous source bytes and matching text before returning a destination.

<a id="f-237646581124-representationRange"></a>
<book:copy id="f-237646581124-representationRange">
/** An edit can span fragments only when their source bytes are contiguous. */
export function representationRange(source:string, file:RepresentedFile, range:{start:LspPosition;end:LspPosition}): {from:number;to:number}|null {
  const start=charToByte(file.content,positionToUtf16(file.content,range.start));
  const end=charToByte(file.content,positionToUtf16(file.content,range.end));
  if (end<start) return null;
  if (file.provenance.some(p => p.origin.kind === "ingested" &&
    (start === end ? p.start <= start && start <= p.end : start < p.end && end > p.start))) return null;
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
</book:copy>

This is where the [immutable-base rule](guarantees/lineage/ingested-bytes-are-not-yours/contract.md)
changes edit behavior without removing navigation.

## Exercise reordered fragments and Unicode

Put two fragments in a different narrative order from their generated order.
Check that navigation follows their origins, that character and byte offsets agree
for Unicode, and that a range crossing disjoint source fragments refuses.

<a id="f-f3e0210e3ed7-expression_statement"></a>
<book:copy id="f-f3e0210e3ed7-expression_statement">
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
</book:copy>

The remaining mapping helpers and second refusal witness stay with the module's
file assemblies:
<book:include file="mechanisms/apps/web/src/lsp/representationMapping.ts.md" />

## Install those boundaries in the two editor views

Compose the document editor with parsing, folding and collaboration. Install the
capture transaction guard there, and protect provenance ranges in the output pane.
Keep a patch draft editable, but [revalidate downstream construction](guarantees/literate-projects/patch-edits-revalidate-downstream-construction/contract.md)
before treating it as an applicable change.

<book:include file="mechanisms/apps/web/src/editor/DocumentEditor.tsx.md" />
<book:include file="mechanisms/apps/web/src/components/OutputEditorPane.tsx.md" />

Run the focused coordinate witnesses from `apps/web`:

```sh
npx vitest run src/lsp/representationMapping.test.ts
```

The editor can now distinguish a navigable byte from an editable one. Next,
[protect that same invariant when saving to disk](05-saving.md).
</book:doc>
