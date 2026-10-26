<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/EmbeddingFixture.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8cbb75d36cd7"></a>
## `apps/web/src/embed/EmbeddingFixture.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-8cbb75d36cd7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-8cbb75d36cd7-expression_statement">
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
</storycode:copy>
<a id="f-8cbb75d36cd7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-8cbb75d36cd7-expression_statement-2">
vi.mock("./measure", () => ({ measurePortableCore: async () => ({ captured_bytes: 0 }) }));
</storycode:copy>
<a id="f-8cbb75d36cd7-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-8cbb75d36cd7-expression_statement-3">
afterEach(cleanup);
</storycode:copy>
<a id="f-8cbb75d36cd7-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-8cbb75d36cd7-expression_statement-4">
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
</storycode:copy>

### Assemble `apps/web/src/embed/EmbeddingFixture.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/EmbeddingFixture.test.tsx"><storycode:paste select="#f-8cbb75d36cd7-expression_statement"/><storycode:paste select="#f-8cbb75d36cd7-expression_statement-2"/><storycode:paste select="#f-8cbb75d36cd7-expression_statement-3"/><storycode:paste select="#f-8cbb75d36cd7-expression_statement-4"/></storycode:file>

</storycode:doc>
