<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/representations.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5e9e398a0278"></a>
## `apps/web/src/api/representations.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-5e9e398a0278-Backing"></a>
### `Backing`

<storycode:copy id="f-5e9e398a0278-Backing">
import { request } from "./client";
import type { Provenance } from "./types";

export type Backing = { kind: "files"; paths: string[] } | { kind: "document"; doc_id: string };
</storycode:copy>
<a id="f-5e9e398a0278-Representation"></a>
### `Representation`

<storycode:copy id="f-5e9e398a0278-Representation">
export interface Representation {
  id: string; backing: Backing; source: string; revision: string;
  files: { path: string; source_path?: string; content: string; hash: string; provenance: Provenance[] }[];
  explanation_stale: boolean; local_warning?: string | null;
}
</storycode:copy>
<a id="f-5e9e398a0278-Comparison"></a>
### `Comparison`

<storycode:copy id="f-5e9e398a0278-Comparison">
export interface Comparison {
  base: string; target: string | null; source: string; target_source: string;
  editable: boolean; revision: string;
}
</storycode:copy>
<a id="f-5e9e398a0278-at"></a>
### `at`

<storycode:copy id="f-5e9e398a0278-at">
const at = (id: string) => `/api/representations/${encodeURIComponent(id)}`;
</storycode:copy>
<a id="f-5e9e398a0278-representations"></a>
### `representations`

<storycode:copy id="f-5e9e398a0278-representations">
export const representations = {
  create: (backing: Backing) => request<Representation>("POST", "/api/representations", { backing }),
  get: (id: string) => request<Representation>("GET", at(id)),
  list: () => request<{ views: Representation[] }>("GET", "/api/representations"),
  edit: (id: string, revision: string, source: string) => request<Representation>("PUT", at(id), { revision, source }),
  refresh: (id: string) => request<Representation>("POST", `${at(id)}/refresh`),
  preview: (id: string, revision: string, source: string) => request<Representation["files"]>("POST", `${at(id)}/preview`, { revision, source }),
  compare: (id: string, base: string, target?: string) => request<Comparison>("GET", `${at(id)}/compare?${new URLSearchParams({ base, ...(target ? { target } : {}) })}`),
  keep: (id: string) => request<{ path: string }>("POST", `${at(id)}/keep`),
  discard: (id: string) => request("DELETE", at(id)),
};
</storycode:copy>
<a id="f-5e9e398a0278-BisectSession"></a>
### `BisectSession`

<storycode:copy id="f-5e9e398a0278-BisectSession">
export interface BisectSession {
  id: string; worktree: string; good: string; bad: string; candidate: string;
  outcome: "first_bad" | "ambiguous" | null; modified: boolean; said: string;
  history: { candidate: string; verdict: string }[]; commits: string;
  graph: {sha:string; parents:string[]; subject:string}[];
}
</storycode:copy>
<a id="f-5e9e398a0278-search"></a>
### `search`

<storycode:copy id="f-5e9e398a0278-search">
const search = (id: string) => `/api/git/bisect/${encodeURIComponent(id)}`;
</storycode:copy>
<a id="f-5e9e398a0278-bisects"></a>
### `bisects`

<storycode:copy id="f-5e9e398a0278-bisects">
export const bisects = {
  list: () => request<{ sessions: BisectSession[] }>("GET", "/api/git/bisect"),
  start: (good: string, bad: string) => request<BisectSession>("POST", "/api/git/bisect", { good, bad }),
  mark: (s: BisectSession, verdict: string) => request<BisectSession>("POST", `${search(s.id)}/mark`, { candidate: s.candidate, verdict }),
  open: (id: string) => request("POST", `${search(id)}/open`),
  restore: (id: string) => request<{ patch: string; session: BisectSession }>("POST", `${search(id)}/restore`),
  finish: (id: string) => request("DELETE", search(id)),
};
</storycode:copy>

### Assemble `apps/web/src/api/representations.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/representations.ts"><storycode:paste select="#f-5e9e398a0278-Backing"/><storycode:paste select="#f-5e9e398a0278-Representation"/><storycode:paste select="#f-5e9e398a0278-Comparison"/><storycode:paste select="#f-5e9e398a0278-at"/><storycode:paste select="#f-5e9e398a0278-representations"/><storycode:paste select="#f-5e9e398a0278-BisectSession"/><storycode:paste select="#f-5e9e398a0278-search"/><storycode:paste select="#f-5e9e398a0278-bisects"/></storycode:file>

</storycode:doc>
