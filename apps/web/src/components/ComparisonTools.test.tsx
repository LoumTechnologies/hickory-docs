import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { request } from "../api/client";
import { materializeProject } from "../editor/hickLang";
import { ComparisonTools } from "./ComparisonTools";
import { ComparisonEditor } from "./ComparisonEditor";

vi.mock("../api/client", () => ({ api: {}, request: vi.fn() }));
vi.mock("../lsp/useLsp", () => ({ useWorkspaceLsp: () => ({ client: null, diagnostics: [] }) }));
vi.mock("../debug/useDebugger", () => ({ useWorkspaceDebugger: () => ({ frames: [], watches: [], status: "idle", start: vi.fn() }) }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
it("registers the completed project after an explicit gesture from a middle patch", async () => {
  const source = `<hick:project/><hick:file id="x" path="x.ts">old\n</hick:file>
<hick:file id="y" path="y.ts">before\n</hick:file>
<hick:patch id="middle" after="x">
--- a/x.ts
+++ b/x.ts
@@ -1 +1 @@
-old
+new
</hick:patch>
<hick:patch id="last" after="y">
--- a/y.ts
+++ b/y.ts
@@ -1 +1 @@
-before
+completed
</hick:patch>`;
  const project = materializeProject("project.md", { "project.md": source });
  vi.mocked(request).mockResolvedValue({ id: "captured", uri: "hick-project:///captured/" });
  render(<ComparisonTools path="x.ts" view={null} source={{ id: "new", label: "New · completed project",
    kind: "authored", text: "new\n", local: project.local_states.middle.new, project: project.new }} />);
  expect(request).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Enable language tools · New · completed project" }));
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST", "/api/project-contexts", expect.anything()));
  const body = vi.mocked(request).mock.calls.find(([method]) => method === "POST")![2] as { snapshot: { entries: Record<string, {bytes: number[]}> } };
  expect(Object.keys(body.snapshot.entries).sort()).toEqual(["x.ts", "y.ts"]);
  expect(new TextDecoder().decode(new Uint8Array(body.snapshot.entries["y.ts"].bytes))).toBe("completed\n");
});

it("does not offer runnable tools for an empty Old endpoint", () => {
  render(<ComparisonTools path="x.ts" view={null} source={{ id: "old", label: "Old · scope input",
    kind: "authored", text: "old\n", project: [] }} />);
  expect((screen.getByRole("button", { name: "Enable language tools · Old · scope input" }) as HTMLButtonElement).disabled).toBe(true);
  expect(screen.getByText("Code absent from Old · scope input; choose another scope endpoint.")).toBeTruthy();
  expect(request).not.toHaveBeenCalled();
});

it("uses each revision's own file path for tools across a rename", () => {
  const entry = (path: string) => ({ path, bytes: [...new TextEncoder().encode("code\n")], kind: "file" as const, executable: false });
  render(<ComparisonEditor path="new.ts" model={{ sources: [
    { id: "old", label: "Old revision", kind: "git", path: "old.ts", text: "code\n", project: [entry("old.ts")] },
    { id: "new", label: "New revision", kind: "git", text: "code\n", project: [entry("new.ts")] },
  ], old: "old", new: "new", policy: "neither", evidence: "unverified" }} />);
  fireEvent.change(screen.getByLabelText("Comparison layout"), { target: { value: "side-by-side" } });
  expect((screen.getByRole("button", { name: "Enable language tools · Old revision" }) as HTMLButtonElement).disabled).toBe(false);
  expect((screen.getByRole("button", { name: "Enable language tools · New revision" }) as HTMLButtonElement).disabled).toBe(false);
  expect(request).not.toHaveBeenCalled();
});
