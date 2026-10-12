import { projectTransition, type ProjectEntry } from "../editor/hickLang";

export type EditPolicy = "neither" | "old" | "new" | "both";
export type RevisionKind = "authored" | "directory" | "git" | "mercurial" | "crdt";
export interface ComparisonSource {
  id: string;
  label: string;
  kind: RevisionKind;
  text: string;
  /** Revision-local path, which can differ across a rename comparison. */
  path?: string;
  target?: string;
  /** The complete selected project, independently of the displayed local text. */
  project?: ProjectEntry[];
  /** Displayed construction tree, if different from the completed endpoint. */
  local?: ProjectEntry[];
}
export interface ComparisonModel {
  sources: readonly ComparisonSource[];
  old: string;
  new: string;
  policy: EditPolicy;
  evidence: "unverified" | "verified";
}
export function canEdit(model: ComparisonModel, side: "old" | "new"): boolean {
  const source = model.sources.find(s => s.id === model[side]);
  return !!source?.target && (source.kind === "authored" || source.kind === "directory")
    && (model.policy === side || model.policy === "both");
}
export function editComparison(model: ComparisonModel, side: "old" | "new", text: string): ComparisonModel {
  if (!canEdit(model, side)) throw new Error("Fork this immutable reading before editing it.");
  return { ...model, evidence: "unverified", sources: model.sources.map(source =>
    source.id === model[side] ? { ...source, text, project: undefined, local: undefined } : source) };
}
export function forkComparison(model: ComparisonModel): ComparisonModel {
  return { ...model, policy: "both", evidence: "unverified", sources: model.sources.map(s =>
    s.id === model.old || s.id === model.new ? { ...s, kind: "authored", target: `draft:${s.id}` } : s) };
}
/** Uses the native engine's exact transition serializer, including newline markers. */
export function comparisonDiff(path: string, before: string, after: string): string {
  const entry = (text: string): ProjectEntry[] => [{ path, bytes: [...new TextEncoder().encode(text)], kind: "file", executable: false }];
  return projectTransition(entry(before), entry(after));
}
