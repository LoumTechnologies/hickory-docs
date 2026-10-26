<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/projectComparison.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7ddde77909ef"></a>
## `apps/web/src/lib/projectComparison.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Comparison edit permissions do not establish replay evidence](../../../../../guarantees/literate-projects/comparison-permissions-and-evidence-are-independent/contract.md).

<a id="f-7ddde77909ef-EditPolicy"></a>
### `EditPolicy`

<storycode:copy id="f-7ddde77909ef-EditPolicy">
import { projectTransition, type ProjectEntry } from "../editor/hickLang";

export type EditPolicy = "neither" | "old" | "new" | "both";
</storycode:copy>
<a id="f-7ddde77909ef-RevisionKind"></a>
### `RevisionKind`

<storycode:copy id="f-7ddde77909ef-RevisionKind">
export type RevisionKind = "authored" | "directory" | "git" | "mercurial" | "crdt";
</storycode:copy>
<a id="f-7ddde77909ef-ComparisonSource"></a>
### `ComparisonSource`

<storycode:copy id="f-7ddde77909ef-ComparisonSource">
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
</storycode:copy>
<a id="f-7ddde77909ef-ComparisonModel"></a>
### `ComparisonModel`

<storycode:copy id="f-7ddde77909ef-ComparisonModel">
export interface ComparisonModel {
  sources: readonly ComparisonSource[];
  old: string;
  new: string;
  policy: EditPolicy;
  evidence: "unverified" | "verified";
}
</storycode:copy>
<a id="f-7ddde77909ef-canEdit"></a>
### `canEdit`

<storycode:copy id="f-7ddde77909ef-canEdit">
export function canEdit(model: ComparisonModel, side: "old" | "new"): boolean {
  const source = model.sources.find(s => s.id === model[side]);
  return !!source?.target && (source.kind === "authored" || source.kind === "directory")
    && (model.policy === side || model.policy === "both");
}
</storycode:copy>
<a id="f-7ddde77909ef-editComparison"></a>
### `editComparison`

<storycode:copy id="f-7ddde77909ef-editComparison">
export function editComparison(model: ComparisonModel, side: "old" | "new", text: string): ComparisonModel {
  if (!canEdit(model, side)) throw new Error("Fork this immutable reading before editing it.");
  return { ...model, evidence: "unverified", sources: model.sources.map(source =>
    source.id === model[side] ? { ...source, text, project: undefined, local: undefined, versions: undefined } : source) };
}
</storycode:copy>
<a id="f-7ddde77909ef-forkComparison"></a>
### `forkComparison`

<storycode:copy id="f-7ddde77909ef-forkComparison">
export function forkComparison(model: ComparisonModel): ComparisonModel {
  return { ...model, policy: "both", evidence: "unverified", sources: model.sources.map(s =>
    s.id === model.old || s.id === model.new ? { ...s, kind: "authored", target: `draft:${s.id}` } : s) };
}
</storycode:copy>
<a id="f-7ddde77909ef-comparisonDiff"></a>
### `comparisonDiff`

Uses the native engine's exact transition serializer, including newline markers.

<storycode:copy id="f-7ddde77909ef-comparisonDiff">
/** Uses the native engine's exact transition serializer, including newline markers. */
export function comparisonDiff(path: string, before: string, after: string): string {
  const entry = (text: string): ProjectEntry[] => [{ path, bytes: [...new TextEncoder().encode(text)], kind: "file", executable: false }];
  return projectTransition(entry(before), entry(after));
}
</storycode:copy>

### Assemble `apps/web/src/lib/projectComparison.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/projectComparison.ts"><storycode:paste select="#f-7ddde77909ef-EditPolicy"/><storycode:paste select="#f-7ddde77909ef-RevisionKind"/><storycode:paste select="#f-7ddde77909ef-ComparisonSource"/><storycode:paste select="#f-7ddde77909ef-ComparisonModel"/><storycode:paste select="#f-7ddde77909ef-canEdit"/><storycode:paste select="#f-7ddde77909ef-editComparison"/><storycode:paste select="#f-7ddde77909ef-forkComparison"/><storycode:paste select="#f-7ddde77909ef-comparisonDiff"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-a9f1fc73a39f"></a>
## `apps/web/src/lib/projectComparison.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Comparison edit permissions do not establish replay evidence](../../../../../guarantees/literate-projects/comparison-permissions-and-evidence-are-independent/contract.md), [Keep a patch edit as a draft until the whole construction still works](../../../../../guarantees/literate-projects/patch-edits-revalidate-downstream-construction/contract.md).

<a id="f-a9f1fc73a39f-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a9f1fc73a39f-expression_statement">
import { describe, expect, it } from "vitest";
import { canEdit, comparisonDiff, editComparison, forkComparison, type ComparisonModel, type EditPolicy } from "./projectComparison";
import { materializeProject, regenerateProjectPatch, type ProjectEntry } from "../editor/hickLang";

// Guarantees: docs/guarantees/literate-projects/comparison-permissions-and-evidence-are-independent.md
describe("comparison edits", () => {
  const model = (policy: EditPolicy): ComparisonModel => ({ sources: [
    { id: "a", label: "Old", kind: "authored", target: "old.md", text: "before" },
    { id: "b", label: "New", kind: "authored", target: "new.md", text: "after" },
    { id: "c", label: "Another revision", kind: "git", text: "other" },
  ], old: "a", new: "b", policy, evidence: "verified" });
  it.each(["neither", "old", "new", "both"] as const)("enforces %s independently and keeps the other endpoint fixed", policy => {
    const original = model(policy);
    for (const side of ["old", "new"] as const) {
      const allowed = policy === side || policy === "both";
      expect(canEdit(original, side)).toBe(allowed);
      if (!allowed) { expect(() => editComparison(original, side, "edit")).toThrow(); continue; }
      const edited = editComparison(original, side, "edit");
      expect(edited.evidence).toBe("unverified");
      expect(edited.sources.find(s => s.id === original[side === "old" ? "new" : "old"])?.text).toBe(side === "old" ? "after" : "before");
      expect(original.evidence).toBe("verified");
    }
  });
  it("requires an authored fork even if an immutable source has a target", () => {
    const original = model("both"); original.sources[0].kind = "git";
    expect(canEdit(original, "old")).toBe(false);
    const fork = forkComparison(original);
    expect(editComparison(fork, "old", "draft").sources[0].text).toBe("draft");
    expect(original.sources[0].text).toBe("before");
  });
  it("uses shared replay for CRLF, Unicode and final-newline changes", () => {
    const before = "é\r\n", after = "changed";
    const diff = comparisonDiff("app.ts", before, after);
    const source = `<hick:project/><hick:patch>\n${diff}</hick:patch>`;
    const input: ProjectEntry[] = [{ path: "app.ts", bytes: [...new TextEncoder().encode(before)], kind: "file", executable: false }];
    const result = materializeProject("p.md", { "p.md": source }, input);
    expect(new TextDecoder().decode(new Uint8Array(result.new[0].bytes))).toBe(after);
  });
});

</storycode:copy>
<a id="f-a9f1fc73a39f-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-a9f1fc73a39f-expression_statement-2">
// Guarantee: docs/guarantees/literate-projects/patch-edits-revalidate-downstream-construction.md
it("WASM patch regeneration refuses a later patch whose context an early edit removed", () => {
  const sources = { "p.md": `<hick:project/><hick:file id="base" path="app.ts">a\n</hick:file>
<hick:patch id="early" after="base">\n--- a/app.ts\n+++ b/app.ts\n@@ -1 +1 @@\n-a\n+b\n</hick:patch>
<hick:patch id="late" after="early">\n--- a/app.ts\n+++ b/app.ts\n@@ -1 +1 @@\n-b\n+c\n</hick:patch>` };
  const project = materializeProject("p.md", sources);
  const replacement = project.local_states.early.new.map(e => ({ ...e, bytes: [...new TextEncoder().encode("unexpected\n")], origins: [] }));
  expect(() => regenerateProjectPatch("p.md", sources, [], "early", replacement)).toThrow(/invalidates/);
  expect(new TextDecoder().decode(new Uint8Array(materializeProject("p.md", sources).new[0].bytes))).toBe("c\n");
});
</storycode:copy>

### Assemble `apps/web/src/lib/projectComparison.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/projectComparison.test.ts"><storycode:paste select="#f-a9f1fc73a39f-expression_statement"/><storycode:paste select="#f-a9f1fc73a39f-expression_statement-2"/></storycode:file>


</storycode:doc>
