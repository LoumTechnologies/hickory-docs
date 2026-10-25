<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/EmbeddingFixture.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `5303e4f45dc81214daa4bbcd26c31f5b7da54b9d151c51f3047978278b000558`.

<book:entry path="apps/web/src/embed/EmbeddingFixture.test.tsx">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { EmbeddingFixture } from "./EmbeddingFixture";
vi.mock("./index", () => ({
  DocumentEmbed: ({ path, source, readOnly }: { path: string; source: string; readOnly: boolean }) => <textarea aria-label={path} value={source} readOnly={readOnly} onChange={() => {}} />,
  DebuggableDocument: () => <p>Actual interpreter is covered by the focused browser witness</p>,
}));
vi.mock("./measure", () => ({ measurePortableCore: async () => ({ captured_bytes: 0 }) }));
afterEach(cleanup);
it("shares native replacement/save with agent read-only and unmount controls", async () => {
  const registry = new UiActions({ workspace: "fixture", window: "host" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><EmbeddingFixture /></UiActionContext.Provider>);
  const run = async (label: string) => {
    const action = registry.observe("embedding-fixture", 64).actions.find(action => action.label === label)!;
    await act(async () => { expect(await registry.act(registry.scope, { id: "parent", kind: "agent" }, { handle: action.handle, arguments: {}, request_id: crypto.randomUUID() })).toMatchObject({ status: "result", outcome: { status: "completed" } }); });
  };
  await run("Toggle read-only");
  expect((screen.getByLabelText("first.md") as HTMLTextAreaElement).readOnly).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: "Replace source" }));
  await act(async () => {});
  await run("Save explicitly");
  expect(screen.getByLabelText("Saved document").textContent).toBe("# Host replacement\n");
  await run("Mount/unmount");
  expect(screen.queryByLabelText("first.md")).toBeNull();
  await run("Mount/unmount");
  expect((screen.getByLabelText("first.md") as HTMLTextAreaElement).value).toBe("# Host replacement\n");
});
</book:entry>

</book:doc>
