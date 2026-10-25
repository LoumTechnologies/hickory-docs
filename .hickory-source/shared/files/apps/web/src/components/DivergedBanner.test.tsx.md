<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/DivergedBanner.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [An Output That Cannot Be Carried Back Is Held, Never Undone](../../../../../../guarantees/authoring/an-output-that-cannot-be-carried-back-is-held/contract.md)
- [Conflict actions name the write](../../../../../../guarantees/authoring/conflict-actions-name-the-write/contract.md)

Captured SHA-256: `813ca63e4befae8ed182f818369fc7b0d6212dd88af7570a81a81bb305e9a069`.

<book:entry path="apps/web/src/components/DivergedBanner.test.tsx">
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
// Protects docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md
// and docs/specs/freeform/three-axes.md, axis 3.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { DivergedBanner } from "./DivergedBanner";

afterEach(cleanup);

describe("the one diverged surface", () => {
  it("offers the same three ways out, and says what mine and theirs are", () => {
    const keep = vi.fn();
    const take = vi.fn();
    const merge = vi.fn();
    render(
      <DivergedBanner
        keepLabel="Leave disk unchanged" keepEffect="Dismisses the warning without writing."
        takeLabel="Regenerate from document" takeEffect="Replaces the disk file with the document output."
        what="client/client.py"
        reason="an edit to generated text has no source to carry back to"
        mine="what is on disk"
        theirs="what the document produces"
        onKeepMine={keep}
        onTakeTheirs={take}
        onMerge={merge}
      />,
    );
    expect(screen.getByRole("alert").textContent).toMatch(/what is on disk is not what the document produces/);
    fireEvent.click(screen.getByRole("button", { name: "Leave disk unchanged" }));
    fireEvent.click(screen.getByRole("button", { name: "Regenerate from document" }));
    fireEvent.click(screen.getByRole("button", { name: "Merge…" }));
    expect(keep).toHaveBeenCalled();
    expect(take).toHaveBeenCalled();
    expect(merge).toHaveBeenCalled();
  });

  it("says what would make theirs exist when there is nothing to take yet", () => {
    render(
      <DivergedBanner
        keepLabel="Leave disk unchanged" keepEffect="Dismisses the warning without writing."
        takeLabel="Regenerate from document" takeEffect="Replaces the disk file with the document output."
        what="report.md"
        reason="2 cell(s) are unrecorded"
        mine="what is on disk"
        theirs="what the document produces"
        onKeepMine={() => {}}
        takeTheirsHint="Run the document to have a version to take."
      />,
    );
    expect(screen.queryByRole("button", { name: "Regenerate from document" })).toBeNull();
    expect(screen.getByText(/Run the document/)).toBeTruthy();
  });
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("binds decisions to the live owner and retains asynchronous host failures", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let source = "PRIVATE FIRST BUFFER";
  const take = vi.fn().mockRejectedValueOnce(new Error("reload failed")).mockResolvedValue({ reloaded: true, hash: "actual-host-hash" });
  render(<UiActionContext.Provider value={registry}><DivergedBanner what="a.txt" reason="disk moved" mine="draft" theirs="disk" keepLabel="Keep draft" keepEffect="Request overwrite" takeLabel="Reload disk" takeEffect="Replace draft" onTakeTheirs={take} sourceRevision={() => source} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(g => registry.observe(g).actions);
  const old = actions().find(a => a.label === "Reload disk")!;
  expect(JSON.stringify(actions())).not.toContain(source);
  source = "PRIVATE CHANGED BUFFER";
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "stale" })).toMatchObject({ status: "stale" });
  expect(take).not.toHaveBeenCalled();
  let id = 0;
  const run = async () => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === "Reload disk")!.handle, arguments: {}, request_id: String(++id) }); }); return result; };
  expect(await run()).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: "reload failed" } });
  expect(screen.getByText("reload failed")).toBeTruthy();
  expect(await run()).toMatchObject({ status: "result", outcome: { status: "completed", value: { owner_reply: JSON.stringify({ reloaded: true, hash: "actual-host-hash" }), verified: false } } });
  expect(take).toHaveBeenCalledTimes(2);
});
</book:entry>

</book:doc>
