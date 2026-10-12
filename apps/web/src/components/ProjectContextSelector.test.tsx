import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { request } from "../api/client";
import { ProjectContextSelector } from "./ProjectContextSelector";

vi.mock("../api/client", () => ({ request: vi.fn() }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
it("defaults to the completed project and makes partial states an explicit choice", async () => {
  const source = `<hick:project/><hick:file id="base" path="app.ts">old\n</hick:file>
<hick:scope id="feature" after="base"><hick:patch id="change">
--- a/app.ts
+++ b/app.ts
@@ -1 +1 @@
-old
+new
</hick:patch></hick:scope>`;
  vi.mocked(request).mockResolvedValue({ entry: "project.md", sources: { "project.md": source } });
  const change = vi.fn();
  render(<ProjectContextSelector doc="project" source={source} value="new" onChange={change} disabled={false} />);
  const selector = screen.getByLabelText("Project tools context") as HTMLSelectElement;
  expect(selector.value).toBe("new");
  expect(request).not.toHaveBeenCalled();
  fireEvent.focus(selector);
  await waitFor(() => expect(screen.getByRole("option", { name: "feature · completed scope" })).toBeTruthy());
  expect(screen.getByRole("group", { name: "Advanced · intermediate states" })).toBeTruthy();
  expect(change).not.toHaveBeenCalled();
  fireEvent.change(selector, { target: { value: "local-before:change" } });
  expect(change).toHaveBeenCalledWith("local-before:change");
});

it("keeps ordinary notes free of project controls", () => {
  render(<ProjectContextSelector doc="note" source="# My note" value="new" onChange={vi.fn()} disabled={false} />);
  expect(screen.queryByLabelText("Project tools context")).toBeNull();
  expect(request).not.toHaveBeenCalled();
});
