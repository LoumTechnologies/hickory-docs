<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/capturedCode.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d3ab3f2ab7a2"></a>
## `apps/web/src/lib/capturedCode.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d3ab3f2ab7a2-openCapturedCode"></a>
### `openCapturedCode`

<storycode:copy id="f-d3ab3f2ab7a2-openCapturedCode">
import { request } from "../api/client";
import type { ProjectEntry } from "../editor/hickLang";
import type { LspNavigationTarget } from "../lsp/cmLsp";
import { retainCapturedContext } from "./capturedContexts";
import { publishReading, openReading } from "./readingViews";

/** Open one real source file at its requested position, preserving native identity. */
export async function openCapturedCode(target: LspNavigationTarget, debugOwner?: string, isCurrent: () => boolean = () => true) {
  if (!isCurrent()) throw Error("The captured navigation owner changed before loading");
  const release = retainCapturedContext(target.uri);
  try {
    const file = await request<{path:string;bytes:number[];files:ProjectEntry[]}>("GET", `/api/project-contexts/file?uri=${encodeURIComponent(target.uri)}`);
    if (!isCurrent()) throw Error("The captured navigation owner changed while loading");
    const text = new TextDecoder("utf-8", {fatal:true}).decode(new Uint8Array(file.bytes));
    const id = `captured:${target.uri}`;
    publishReading(id, { title: file.path, path: file.path, before: text, after: text,
      status: "Captured project", project: file.files, codeUri: target.uri, debugOwner,
      location: target.range.start, dispose: release });
    openReading(id, file.path);
  } catch (e) { release(); throw e; }
}
</storycode:copy>

### Assemble `apps/web/src/lib/capturedCode.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/capturedCode.ts"><storycode:paste select="#f-d3ab3f2ab7a2-openCapturedCode"/></storycode:file>

</storycode:doc>
