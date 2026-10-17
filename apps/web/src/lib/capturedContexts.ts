import { request } from "../api/client";

// Views, navigation records and live debuggers share a captured native identity.
type Context = {id:string;uri:string;workspace?:string};
const files = new Map<string,{entries:Record<string,unknown>;workspace:string}>();
export function capturedFiles(uri:string):string[] {
  const id=idOf(uri);return id ? Object.keys(files.get(id)?.entries ?? {}) : [];
}
export function capturedSourceKey(uri:string):string|null {
  const id=idOf(uri);if(!id)return null;
  const context=files.get(id), path=uri.slice(uri.indexOf(id)+id.length+1);
  const entry=context?.entries[path] as {bytes?:number[];kind?:string}|undefined;
  return entry?.bytes && context ? JSON.stringify([context.workspace,path,entry.bytes]) : null;
}
type Shared = { promise: Promise<Context>; users:number; release?:()=>void };
const shared = new Map<string, Shared>();
const keys = new Map<string,string>();

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
const references = new Map<string, number>();
const idOf = (uri: string) => /^hick-project:\/\/\/([^/]+)\//.exec(uri)?.[1] ?? null;
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
