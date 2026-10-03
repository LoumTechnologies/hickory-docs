import { request } from "./client";
import type { Provenance } from "./types";

export type Backing = { kind: "files"; paths: string[] } | { kind: "document"; doc_id: string };
export interface Representation {
  id: string; backing: Backing; source: string; revision: string;
  files: { path: string; source_path?: string; content: string; hash: string; provenance: Provenance[] }[];
  explanation_stale: boolean; local_warning?: string | null;
}
export interface Comparison {
  base: string; target: string | null; source: string; target_source: string;
  editable: boolean; revision: string;
}
const at = (id: string) => `/api/representations/${encodeURIComponent(id)}`;
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
export interface BisectSession {
  id: string; worktree: string; good: string; bad: string; candidate: string;
  outcome: "first_bad" | "ambiguous" | null; modified: boolean; said: string;
  history: { candidate: string; verdict: string }[]; commits: string;
  graph: {sha:string; parents:string[]; subject:string}[];
}
const search = (id: string) => `/api/git/bisect/${encodeURIComponent(id)}`;
export const bisects = {
  list: () => request<{ sessions: BisectSession[] }>("GET", "/api/git/bisect"),
  start: (good: string, bad: string) => request<BisectSession>("POST", "/api/git/bisect", { good, bad }),
  mark: (s: BisectSession, verdict: string) => request<BisectSession>("POST", `${search(s.id)}/mark`, { candidate: s.candidate, verdict }),
  open: (id: string) => request("POST", `${search(id)}/open`),
  restore: (id: string) => request<{ patch: string; session: BisectSession }>("POST", `${search(id)}/restore`),
  finish: (id: string) => request("DELETE", search(id)),
};
