<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/capturedCode.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `84224a00daf3ea073f8449d40246137f69b0ae57f6ad5eed6a36a435de0bbbb9`.

<book:entry path="apps/web/src/lib/capturedCode.ts">
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
</book:entry>

</book:doc>
