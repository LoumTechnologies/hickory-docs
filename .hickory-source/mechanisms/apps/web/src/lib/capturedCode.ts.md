<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/capturedCode.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d3ab3f2ab7a2"></a>
## `apps/web/src/lib/capturedCode.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d3ab3f2ab7a2-openCapturedCode"></a>
### `openCapturedCode`

Read the captured bytes independently of positional authority. Preserve a UTF-8
BOM in the readable text; decoding must not silently change source identity. A
caller with an unknown frame hash or adapter-owned source can open the existing
captured file without a location. The original stop guard still retires a held
response before it publishes or navigates.

<storycode:copy id="f-d3ab3f2ab7a2-openCapturedCode">
import { request } from "../api/client";
import type { ProjectEntry } from "../editor/hickLang";
import type { LspNavigationTarget } from "../lsp/cmLsp";
import { retainCapturedContext } from "./capturedContexts";
import { publishReading, openReading } from "./readingViews";

/** Open one real source file at its requested position, preserving native identity. */
export async function openCapturedCode(target: LspNavigationTarget, debugOwner?: string, isCurrent: () => boolean = () => true,positioned=true) {
  if (!isCurrent()) throw Error("The captured navigation owner changed before loading");
  const release = retainCapturedContext(target.uri);
  try {
    const file = await request<{path:string;bytes:number[];files:ProjectEntry[]}>("GET", `/api/project-contexts/file?uri=${encodeURIComponent(target.uri)}`);
    if (!isCurrent()) throw Error("The captured navigation owner changed while loading");
    const text = new TextDecoder("utf-8", {fatal:true,ignoreBOM:true}).decode(new Uint8Array(file.bytes));
    const id = `captured:${target.uri}`;
    publishReading(id, { title: file.path, path: file.path, before: text, after: text,
      status: "Captured project", project: file.files, codeUri: target.uri, debugOwner,
      location: positioned?target.range.start:undefined, dispose: release });
    openReading(id, file.path);
  } catch (e) { release(); throw e; }
}
</storycode:copy>

### Assemble `apps/web/src/lib/capturedCode.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/capturedCode.ts"><storycode:paste select="#f-d3ab3f2ab7a2-openCapturedCode"/></storycode:file>

## Focused readable-source receipt — 2026-11-16

Both capturedCode tests passed: a retired stop cannot publish a delayed reading, and an unknown-position reading preserves the UTF-8 BOM while omitting the invented location. The real captured-literal gutter fixture additionally passed the known-hash generated-source navigation and Stop path. The unknown-hash branch has focused model/adapter evidence; actual DAP sourceReference retrieval is not implemented or claimed by opening an existing captured file.

</storycode:doc>
