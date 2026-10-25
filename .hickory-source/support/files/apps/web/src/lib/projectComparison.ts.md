<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/projectComparison.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Comparison edit permissions do not establish replay evidence](../../../../../../guarantees/literate-projects/comparison-permissions-and-evidence-are-independent/contract.md)

Captured SHA-256: `a98392492340f86e356dfa99c4698502f0459fca6680982979e18b027d6fad55`.

<book:entry path="apps/web/src/lib/projectComparison.ts">
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
  /** An already registered immutable source, opened by code navigation. */
  codeUri?: string;
  debugOwner?: string;
  debugSurface?: { view: import("@codemirror/view").EditorView; entry: string; source: string; path?: string };
  location?: {line:number;character:number};
  target?: string;
  /** The complete selected project, independently of the displayed local text. */
  project?: ProjectEntry[];
  /** Evaluated whole-project versions, oldest first, for exact-source tools. */
  versions?: ProjectEntry[][];
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
    source.id === model[side] ? { ...source, text, project: undefined, local: undefined, versions: undefined } : source) };
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
</book:entry>

</book:doc>
