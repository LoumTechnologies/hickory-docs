<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/CellPanel.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [An Ingested Cell Names Itself In The Document View](../../../../../../guarantees/editor-intelligence/an-ingested-cell-names-itself-in-the-document-view/contract.md)
- [A run result is distinct from what was checked](../../../../../../guarantees/execution/a-run-result-is-distinct-from-what-was-checked/contract.md)

Captured SHA-256: `42297044a15dc86b6b72c7580e09f68ba06e956fa20163536829a6e1cf240d29`.

<book:entry path="apps/web/src/components/CellPanel.test.tsx">
// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { CellPanel } from "./CellPanel";
import { hasReplay } from "../lib/railActions";
import type { ExecBlock, TranscriptEvent } from "../api/types";

afterEach(cleanup);

const transcript: TranscriptEvent[] = [
  { t: 0, kind: "cmd", data: "sort fruit.csv" },
  { t: 300, kind: "out", data: "apple,3\nbanana,5\n" },
  { t: 350, kind: "exit", code: 0 },
];

function cell(overrides: Partial<ExecBlock>): ExecBlock {
  return {
    kind: "exec",
    id: "c1",
    container: "shell",
    command: "sort fruit.csv",
    span: [0, 10],
    transcript,
    ...overrides,
  };
}

describe("CellPanel de-duplication", () => {
  it("verified cell (ok + expect): no transcript by default, ✓ affordance, Replay reveals it", () => {
    const { container, rerender } = render(
      <CellPanel
        block={cell({ status: "ok", expect: { match: "exact", body: "apple,3\nbanana,5\n" } })}
      />,
    );
    // The expect body in the source is the single visible copy of the output.
    expect(container.querySelector(".transcript")).toBeNull();
    expect(screen.getByText("✓ Expected output matches")).toBeTruthy();
    // No separate "expects …" summary chip anymore.
    expect(container.textContent).not.toContain("expects exact");
    // Nothing in the panel is clickable now — the Replay verb is an icon on
    // the action rail, and the panel only reflects the state it is handed.
    expect(container.querySelector("button")).toBeNull();
    rerender(
      <CellPanel
        block={cell({ status: "ok", expect: { match: "exact", body: "apple,3\nbanana,5\n" } })}
        replay
      />,
    );
    expect(container.querySelector(".transcript")).toBeTruthy();
  });

  it("failed cell with expect: compact line diff of actual vs expected", () => {
    const { container } = render(
      <CellPanel
        block={cell({
          status: "failed",
          transcript: [
            { t: 0, kind: "cmd", data: "sort fruit.csv" },
            { t: 300, kind: "out", data: "apple,3\ncherry,7\n" },
            { t: 350, kind: "exit", code: 1 },
          ],
          expect: { match: "exact", body: "apple,3\nbanana,5\n" },
        })}
      />,
    );
    const diff = screen.getByTestId("cell-diff");
    expect(diff.textContent).toContain("- banana,5");
    expect(diff.textContent).toContain("+ cherry,7");
    // Transcript stays collapsed behind Replay (the diff is the second copy).
    expect(container.querySelector(".transcript")).toBeNull();
    // The Replay verb lives on the rail; `hasReplay` is what puts it there,
    // and it agrees with the panel about there being something to reveal.
    expect(
      hasReplay({ status: "failed", hasExpect: true, transcriptLength: 3, running: false }),
    ).toBe(true);
  });

  it("cell without expect keeps its transcript visible — that IS the output", () => {
    const { container } = render(<CellPanel block={cell({ status: "ok" })} />);
    expect(container.querySelector(".transcript")).toBeTruthy();
    // What the panel shows is the watching binding of the terminal, not a
    // `<pre>` — so what it *draws* is asserted in
    // `terminal/WatchingTerminal.test.tsx` against a real emulator, and what
    // it is *handed* in `lib/watchStream.test.ts`. All this panel decides is
    // whether it appears at all.
    expect(container.querySelector("[data-testid='watch-terminal']")).toBeTruthy();
  });
});

// Protects docs/guarantees/editor-intelligence/an-ingested-cell-names-itself-in-the-document-view.md
describe("CellPanel ingested provenance", () => {
  it("shows a chip naming the run when the cell owns a hick:ingested child", () => {
    const { container } = render(
      <CellPanel
        block={cell({
          status: "ok",
          ingested: {
            from: "#scaffold",
            at: "2026-09-01",
            sha256: "85fd490b43d487c91da6c62753723a71a7c04d0",
            files: "2",
            skipped: "5",
          },
        })}
      />,
    );
    expect(screen.getByText(/ingested · 2 files/)).toBeTruthy();
    const chip = container.querySelector(".cell-ingested");
    expect(chip).toBeTruthy();
    expect(chip?.getAttribute("data-tip")).toContain("2026-09-01");
    expect(chip?.getAttribute("data-tip")).toContain("85fd490b43d4");
    expect(chip?.getAttribute("data-tip")).toContain("5 skipped");
  });

  it("says nothing about skipping when nothing was skipped", () => {
    const { container } = render(
      <CellPanel
        block={cell({
          status: "ok",
          ingested: {
            from: "#scaffold",
            at: "2026-09-01",
            sha256: "abc123",
            files: "1",
            skipped: "0",
          },
        })}
      />,
    );
    expect(screen.getByText(/ingested · 1 file(?!s)/)).toBeTruthy();
    expect(container.querySelector(".cell-ingested")?.getAttribute("data-tip")).not.toContain(
      "skipped",
    );
  });

  it("shows no chip at all for an ordinary cell", () => {
    const { container } = render(<CellPanel block={cell({ status: "ok" })} />);
    expect(container.querySelector(".cell-ingested")).toBeNull();
  });
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses only currently visible transcript and expires reads when replay/source changes", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const block = cell({ status: "ok", expect: { match: "exact", body: "apple,3\nbanana,5\n" } });
  const content = (replay: boolean) => <UiActionContext.Provider value={registry}><CellPanel block={block} replay={replay} /></UiActionContext.Provider>;
  const view = render(content(false));
  const parent = registry.groups().groups.find(g => g.startsWith("reading:"))!;
  const action = registry.observe(parent).actions[0];
  expect(JSON.stringify(registry.observe(parent))).not.toContain("apple,3");
  const reading = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected bytes");
  const decoded = new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes));
  expect(decoded).not.toContain("apple,3"); expect(JSON.parse(decoded).transcript_hidden).toBe(true);
  view.rerender(content(true));
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: { offset: 0, limit: 8192 }, request_id: "old" })).toMatchObject({ status: "stale" });
  const current = registry.observe(parent).actions[0];
  const expanded = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: current.handle, arguments: { offset: 0, limit: 8192 }, request_id: "expanded" });
  if (expanded.status !== "result" || expanded.outcome.status !== "completed") throw Error("Expected replay bytes");
  expect(new TextDecoder().decode(new Uint8Array((expanded.outcome.value as { bytes: number[] }).bytes))).toContain("apple,3");
});

it("shows a failed run with no transcript as an unavailable cell outcome, retaining older evidence without a fresh pass", async () => {
  const { completedRunBlocks } = await import("../views/documentSession");
  const { failedRunBanner } = await import("../views/documentSession");
  const receipt = { id: "failed-run", status: "failed" as const, started_at: "now", blocks: [], error: "cell exceeded its timeout" };
  // This is the actual host Err shape: no blocks and no streamed events.
  const original = cell({ status: "unrecorded", transcript: [] });
  const blocks = completedRunBlocks(receipt, [original], new Set(), new Set([original.id]))!;
  expect(failedRunBanner(receipt)?.text).toContain("cell exceeded its timeout");
  const view = render(<CellPanel block={blocks[0] as ExecBlock} />);
  expect(screen.getByText("Run failed · cell outcome unavailable")).toBeTruthy();
  expect(view.container.textContent).not.toContain("never run");
  expect(screen.getByRole("status").textContent).toContain("does not establish which cells executed");
  const earlier = cell({ status: "ok", expect: { match: "exact", body: "apple,3\nbanana,5\n" } });
  const retained = completedRunBlocks(receipt, [earlier], new Set(), new Set([earlier.id]))!;
  view.rerender(<CellPanel block={retained[0] as ExecBlock} replay />);
  expect(screen.queryByText("✓ Expected output matches")).toBeNull();
  expect(screen.getByRole("status").textContent).toContain("Earlier recorded output is retained");
  expect((retained[0] as ExecBlock).transcript).toEqual(earlier.transcript);
  // A real fresh receipt retires the transient attempt error.
  const next = completedRunBlocks({ ...receipt, status: "ok", blocks: [earlier], error: null }, retained, new Set())!;
  view.rerender(<CellPanel block={next[0] as ExecBlock} />);
  expect(screen.queryByText("Run failed · cell outcome unavailable")).toBeNull();
  expect(screen.getByText("✓ Expected output matches")).toBeTruthy();
});
</book:entry>

</book:doc>
