<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/capturedContexts.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-62754565d899"></a>
## `apps/web/src/lib/capturedContexts.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-62754565d899-Context"></a>
### `Context`

<storycode:copy id="f-62754565d899-Context">
import { request } from "../api/client";

// Views, navigation records and live debuggers share a captured native identity.
type Context = {id:string;uri:string;workspace?:string};
</storycode:copy>
<a id="f-62754565d899-files"></a>
### `files`

<storycode:copy id="f-62754565d899-files">
const files = new Map<string,{entries:Record<string,unknown>;workspace:string}>();
</storycode:copy>
<a id="f-62754565d899-capturedFiles"></a>
### `capturedFiles`

<storycode:copy id="f-62754565d899-capturedFiles">
export function capturedFiles(uri:string):string[] {
  const id=idOf(uri);return id ? Object.keys(files.get(id)?.entries ?? {}) : [];
}
</storycode:copy>
<a id="f-62754565d899-capturedSourceKey"></a>
### `capturedSourceKey`

<storycode:copy id="f-62754565d899-capturedSourceKey">
export function capturedSourceKey(uri:string):string|null {
  const id=idOf(uri);if(!id)return null;
  const context=files.get(id), path=uri.slice(uri.indexOf(id)+id.length+1);
  const entry=context?.entries[path] as {bytes?:number[];kind?:string}|undefined;
  return entry?.bytes && context ? JSON.stringify([context.workspace,path,entry.bytes]) : null;
}
</storycode:copy>
<a id="f-62754565d899-Shared"></a>
### `Shared`

<storycode:copy id="f-62754565d899-Shared">
type Shared = { promise: Promise<Context>; users:number; release?:()=>void };
</storycode:copy>
<a id="f-62754565d899-shared"></a>
### `shared`

<storycode:copy id="f-62754565d899-shared">
const shared = new Map<string, Shared>();
</storycode:copy>
<a id="f-62754565d899-keys"></a>
### `keys`

<storycode:copy id="f-62754565d899-keys">
const keys = new Map<string,string>();

</storycode:copy>
<a id="f-62754565d899-acquireCapturedContext"></a>
### `acquireCapturedContext`

Identical complete endpoints share native source and breakpoint identities.

<storycode:copy id="f-62754565d899-acquireCapturedContext">
/** Identical complete endpoints share native source and breakpoint identities. */
export async function acquireCapturedContext(entries: Record<string, unknown>, namespace = ""): Promise<Context & {release:()=>void}> {
  const key = JSON.stringify([namespace,Object.fromEntries(Object.entries(entries).sort(([a],[b])=>a.localeCompare(b)))]);
  let held = shared.get(key);
  if (!held) {
    held = {users:0, promise:request<Context>("POST", "/api/project-contexts", {snapshot:{entries}})};
    shared.set(key,held);
    held.promise = held.promise.then(context => { keys.set(context.id,key); files.set(context.id,{entries,workspace:`${context.workspace ?? "workspace"}:${namespace}`}); return context; }, error => { shared.delete(key); throw error; });
  }
  const record = held;
  record.users++;
  let context: Context;
  try { context = await record.promise; }
  catch (error) {record.users--; throw error;}
  record.release ??= retainCapturedContext(context.uri,true);
  let released = false;
  return {...context, release:() => {
    if (released) return;
    released = true;
    if (--record.users === 0) { const release = record.release; record.release = undefined; release?.(); }
  }};
}
</storycode:copy>
<a id="f-62754565d899-references"></a>
### `references`

<storycode:copy id="f-62754565d899-references">
const references = new Map<string, number>();
</storycode:copy>
<a id="f-62754565d899-idOf"></a>
### `idOf`

<storycode:copy id="f-62754565d899-idOf">
const idOf = (uri: string) => /^hick-project:\/\/\/([^/]+)\//.exec(uri)?.[1] ?? null;
</storycode:copy>
<a id="f-62754565d899-retainCapturedContext"></a>
### `retainCapturedContext`

<storycode:copy id="f-62754565d899-retainCapturedContext">
export function retainCapturedContext(uri: string, adopt = false): () => void {
  const id = idOf(uri);
  if (!id || (!adopt && !references.has(id))) return () => {};
  references.set(id, (references.get(id) ?? 0) + 1);
  let released = false;
  return () => {
    if (released) return;
    released = true;
    const remaining = (references.get(id) ?? 1) - 1;
    if (remaining) references.set(id, remaining);
    else {
      references.delete(id);
      files.delete(id);
      const key = keys.get(id);
      if (key !== undefined) shared.delete(key);
      keys.delete(id);
      void request("DELETE", `/api/project-contexts/${id}`).catch(() => {});
    }
  };
}
</storycode:copy>

### Assemble `apps/web/src/lib/capturedContexts.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/capturedContexts.ts"><storycode:paste select="#f-62754565d899-Context"/><storycode:paste select="#f-62754565d899-files"/><storycode:paste select="#f-62754565d899-capturedFiles"/><storycode:paste select="#f-62754565d899-capturedSourceKey"/><storycode:paste select="#f-62754565d899-Shared"/><storycode:paste select="#f-62754565d899-shared"/><storycode:paste select="#f-62754565d899-keys"/><storycode:paste select="#f-62754565d899-acquireCapturedContext"/><storycode:paste select="#f-62754565d899-references"/><storycode:paste select="#f-62754565d899-idOf"/><storycode:paste select="#f-62754565d899-retainCapturedContext"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-b385e5ebe593"></a>
## `apps/web/src/lib/capturedContexts.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b385e5ebe593-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b385e5ebe593-expression_statement">
import { expect, it, vi } from "vitest";
import { request } from "../api/client";
import { acquireCapturedContext, retainCapturedContext } from "./capturedContexts";
vi.mock("../api/client", () => ({request:vi.fn()}));
</storycode:copy>
<a id="f-b385e5ebe593-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b385e5ebe593-expression_statement-2">
it("shares a complete endpoint across patch panes and retains it throughout debugging", async () => {
  vi.mocked(request).mockResolvedValue({id:"same",uri:"hick-project:///same/"});
  const entries={"main.py":{bytes:[1]},"library.py":{bytes:[2]}};
  const [caller,callee]=await Promise.all([acquireCapturedContext(entries),acquireCapturedContext({...entries})]);
  expect(request).toHaveBeenCalledTimes(1);
  expect(caller.uri).toBe(callee.uri);
  const debug=retainCapturedContext(caller.uri);
  caller.release();callee.release();
  expect(request).not.toHaveBeenCalledWith("DELETE",expect.anything());
  const reopened=await acquireCapturedContext(entries);
  expect(reopened.uri).toBe(caller.uri);
  expect(request).toHaveBeenCalledTimes(1);
  reopened.release();debug();
  expect(request).toHaveBeenLastCalledWith("DELETE","/api/project-contexts/same");
});
</storycode:copy>

### Assemble `apps/web/src/lib/capturedContexts.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/capturedContexts.test.ts"><storycode:paste select="#f-b385e5ebe593-expression_statement"/><storycode:paste select="#f-b385e5ebe593-expression_statement-2"/></storycode:file>


</storycode:doc>
