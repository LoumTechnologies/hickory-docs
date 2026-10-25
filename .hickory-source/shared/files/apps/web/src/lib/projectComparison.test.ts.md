<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/projectComparison.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Comparison edit permissions do not establish replay evidence](../../../../../../guarantees/literate-projects/comparison-permissions-and-evidence-are-independent/contract.md)
- [Patch regeneration revalidates downstream construction](../../../../../../guarantees/literate-projects/patch-edits-revalidate-downstream-construction/contract.md)

Captured SHA-256: `78236a3adf03fbdf20bb2be47e322f302ae2c6e06f1c205a92e88b5e5808c71d`.

<book:entry path="apps/web/src/lib/projectComparison.test.ts">
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
</book:entry>

</book:doc>
