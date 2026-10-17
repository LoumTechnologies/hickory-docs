import { request } from "../api/client";

// Views, navigation records and live debuggers share a captured native identity.
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
      void request("DELETE", `/api/project-contexts/${id}`).catch(() => {});
    }
  };
}
