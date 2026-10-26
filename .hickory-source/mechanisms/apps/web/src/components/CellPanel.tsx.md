<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/CellPanel.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c3c409b02e61"></a>
## `apps/web/src/components/CellPanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Literate File Opens Rendered, And Its Source Is One Click Away](../../../../../guarantees/authoring/a-literate-file-opens-rendered/contract.md), [An Ingested Cell Names Itself In The Document View](../../../../../guarantees/editor-intelligence/an-ingested-cell-names-itself-in-the-document-view/contract.md), [A run result is distinct from what was checked](../../../../../guarantees/execution/a-run-result-is-distinct-from-what-was-checked/contract.md), [A Watched Cell Shows What The Terminal Showed](../../../../../guarantees/terminal/a-watched-cell-shows-what-the-terminal-showed/contract.md).

<a id="f-c3c409b02e61-looksLikeSvg"></a>
### `looksLikeSvg`

<storycode:copy id="f-c3c409b02e61-looksLikeSvg">
import { useUiReading } from "../lib/uiReading";
import { notebookOutputs } from "../lib/notebookOutput";
import { NotebookOutputs } from "./NotebookOutputs";
import { useMemo } from "react";
import { hasReplay } from "../lib/railActions";
import type { ExecBlock } from "../api/types";
import { diffLines } from "../lib/diff";
import { stdoutOf } from "../lib/transcript";
import { StatusChip } from "./StatusChip";
import { WatchingTerminal } from "../terminal/WatchingTerminal";

function looksLikeSvg(s: string): boolean {
  const t = s.trim();
  return t.startsWith("<svg") && t.endsWith("</svg>");
}

</storycode:copy>
<a id="f-c3c409b02e61-IngestedChip"></a>
### `IngestedChip`

The fact the woven markdown's "Ingested from …" caption already states,
said here too: the exec card's own rendering stops before an
`<hick:ingested>` child begins (`editor/rendered.ts`), so without this a
reader scrolling the live Document view saw a status chip and then plain,
unmarked text — no signal at all that what follows arrived from a real
run rather than from the document's author.

<storycode:copy id="f-c3c409b02e61-IngestedChip">
/**
 * The fact the woven markdown's "Ingested from …" caption already states,
 * said here too: the exec card's own rendering stops before an
 * `<hick:ingested>` child begins (`editor/rendered.ts`), so without this a
 * reader scrolling the live Document view saw a status chip and then plain,
 * unmarked text — no signal at all that what follows arrived from a real
 * run rather than from the document's author.
 */
function IngestedChip({ ingested }: { ingested: NonNullable<ExecBlock["ingested"]> }) {
  const short = ingested.sha256.slice(0, 12);
  const fileCount = Number.parseInt(ingested.files, 10);
  const fileWord =
    Number.isFinite(fileCount) && fileCount === 1 ? "file" : "files";
  const skippedNote =
    ingested.skipped && ingested.skipped !== "0" ? `, ${ingested.skipped} skipped` : "";
  return (
    <span
      className="cell-ingested"
      data-tip={`ingested${ingested.at ? ` on ${ingested.at}` : ""} · run ${short}${skippedNote} — these bytes came from that run, not from this document's author`}
    >
      ⎘ ingested{ingested.files ? ` · ${ingested.files} ${fileWord}` : ""}
    </span>
  );
}

</storycode:copy>
<a id="f-c3c409b02e61-CellPanelProps"></a>
### `CellPanelProps`

<storycode:copy id="f-c3c409b02e61-CellPanelProps">
export interface CellPanelProps {
  /** The rendered exec block backing this cell (undefined until the server's
   * render catches up with a freshly typed cell). */
  block?: ExecBlock;
  running?: boolean;
  recordedOutput?: string;
  /** The cell's commands. Passed when this panel STANDS IN for the source —
   * the block is rendered, so the commands are not on screen anywhere else
   * and the panel has to show them. Omitted when the source is visible above
   * the panel, where repeating it would be noise. */
  command?: string;
  /** Whether the transcript is revealed. Owned by the rail, which carries
   * the Replay icon — see lib/railActions.ts. */
  replay?: boolean;
}

</storycode:copy>
<a id="f-c3c409b02e61-CellPanel"></a>
### `CellPanel`

The strip below an exec cell in the Document view: what the cell IS and
what it did — container, status, and, only when it adds information, the
output. Nothing in here is clickable: every verb this cell has (run,
source, replay) is an icon on the action rail, because a rendered card is
something to read and the rail is the column that exists to be clicked.
The cell's source already
shows the command and (when verified) the expect body IS the output, so:
- status ok + expect: no transcript by default (the verified expect body
in the source is the single visible copy); a "Replay" toggle reveals the
animated transcript on demand, and the bar shows "✓ Expected output matches".
- status failed + expect: the actual output earns its second copy — shown
as a compact line diff against the expect body; transcript behind Replay.
- no expect: the transcript IS the output — visible as before.

<storycode:copy id="f-c3c409b02e61-CellPanel">
/**
 * The strip below an exec cell in the Document view: what the cell IS and
 * what it did — container, status, and, only when it adds information, the
 * output. Nothing in here is clickable: every verb this cell has (run,
 * source, replay) is an icon on the action rail, because a rendered card is
 * something to read and the rail is the column that exists to be clicked.
 * The cell's source already
 * shows the command and (when verified) the expect body IS the output, so:
 *
 *  - status ok + expect: no transcript by default (the verified expect body
 *    in the source is the single visible copy); a "Replay" toggle reveals the
 *    animated transcript on demand, and the bar shows "✓ Expected output matches".
 *  - status failed + expect: the actual output earns its second copy — shown
 *    as a compact line diff against the expect body; transcript behind Replay.
 *  - no expect: the transcript IS the output — visible as before.
 */
export function CellPanel({
  block,
  running,
  command,
  replay = false,
  recordedOutput,
}: CellPanelProps) {
  const transcript = useMemo(() => (block?.transcript ?? []).map(event => event.kind === "cmd" && (event.data.startsWith('python3 -c "exec(bytes.fromhex(') || event.data === "python3 -" || event.data === "python -") ? { ...event, data: block?.command ?? event.data } : event), [block]);
  const savedOnly = !transcript.length && recordedOutput !== undefined;
  const stdout = useMemo(() => savedOnly ? recordedOutput ?? "" : stdoutOf(transcript), [transcript, recordedOutput, savedOnly]);
  const rich = useMemo(() => notebookOutputs(stdout), [stdout]);
  const displayTranscript = useMemo(() => rich.outputs.length ? transcript.filter(e => e.kind !== "out").concat(rich.text.trim() ? [{ t: 0, kind: "out" as const, data: rich.text }] : []) : transcript, [transcript, rich]);
  const svgFigure = useMemo(() => (looksLikeSvg(stdout) ? stdout.trim() : null), [stdout]);

  const verified = !running && !block?.runError && block?.status === "ok" && !!block?.expect;
  const failedExpect =
    !running && !block?.runError && block?.status === "failed" && !!block?.expect && transcript.length > 0;
  const diff = useMemo(
    () => (failedExpect && block?.expect ? diffLines(block.expect.body, stdout) : null),
    [failedExpect, block?.expect, stdout],
  );

  const collapsible = block ? hasReplay({
    status: block.status,
    hasExpect: !!block.expect,
    transcriptLength: transcript.length,
    running: !!running,
  }) : false;
  const showTranscript =
    running || (transcript.length > 0 && (collapsible ? replay : !verified && !failedExpect)
      && (rich.outputs.length === 0 || !!rich.text.trim() || transcript.some(e => e.kind === "err") || replay));

  useUiReading("cell rendering", { id: block?.id ?? null, command: command ?? null, status: block?.runError ? "run_failed_cell_outcome_unavailable" : block?.status ?? "save_required", recorded_status: block?.status ?? null, run_error: block?.runError ?? null, running: !!running, container: block?.container ?? null, image: block?.image ?? null, saved_output: block && savedOnly ? rich.text : null, expected_output_matches: verified, diff, replay_available: collapsible, transcript: block && showTranscript ? displayTranscript : null, transcript_hidden: !!block && !showTranscript && transcript.length > 0, rich_outputs: block && !running ? rich.outputs : [], svg: block && !running ? svgFigure : null, ingested: block?.ingested ?? null, scope: "Current cell view; recorded output/expectation match does not verify prose or certify current source/run freshness" }, block);

  // A cell the server has not rendered yet still shows its commands and can
  // still be read; only the run is unavailable, and it says why.
  if (!block) {
    return (
      <div className="cell-panel" data-testid="cell-panel-pending">
        {command !== undefined && <CommandLines command={command} />}
        <div className="cell-panel-bar">
          <span className="muted">Save this document before running the cell.</span>
        </div>
      </div>
    );
  }


  return (
    <div className="cell-panel" data-testid={`cell-panel-${block.id}`}>
      {command !== undefined && <CommandLines command={command} />}
      <div className="cell-panel-bar">
        <span className="cell-container" data-tip={block.image ?? "host"}>
          {block.container}
          {block.image ? ` · ${block.image}` : ""}
        </span>
        {block.runError && !running
          ? <span className="chip chip-failed" data-tip={block.runError}>Run failed · cell outcome unavailable</span>
          : <StatusChip status={block.status} running={running} />}
        {savedOnly && <span className="muted">Saved output</span>}
        {verified && (
          <span className="cell-verified" data-tip="The recorded stdout matches this cell’s expectation; this does not assert that the prose is correct.">
            ✓ Expected output matches
          </span>
        )}
        {block.ingested && <IngestedChip ingested={block.ingested} />}
      </div>
      {block.runError && !running && <p role="status">The document run failed without per-cell results. This does not establish which cells executed.{transcript.length > 0 ? " Earlier recorded output is retained." : ""}</p>}
      {diff && (
        <div className="cell-diff" data-testid="cell-diff">
          <div className="cell-diff-title">output vs expected</div>
          <pre>
            {diff.map((l, i) => (
              <span key={i} className={`d-${l.kind}`}>
                {l.kind === "del" ? "- " : l.kind === "ins" ? "+ " : "  "}
                {l.text}
                {"\n"}
              </span>
            ))}
          </pre>
        </div>
      )}
      {showTranscript && (
        // The watching binding of the one terminal component
        // (`docs/specs/freeform/a-terminal-that-writes-the-document.md`):
        // input none, writes nothing. It replaced a `<pre>` that showed a
        // build's escape sequences as literal text.
        <div className="transcript">
          <WatchingTerminal events={displayTranscript} live={running} />
        </div>
      )}
      {savedOnly && !!rich.text.trim() && !svgFigure && <pre className="notebook-saved-text">{rich.text}</pre>}
      {!running && rich.outputs.length > 0 && <NotebookOutputs outputs={rich.outputs} />}
      {svgFigure && !running && (
        <figure className="cell-figure"><img alt="Cell SVG output" src={`data:image/svg+xml,${encodeURIComponent(svgFigure)}`} /></figure>
      )}
    </div>
  );
}

</storycode:copy>
<a id="f-c3c409b02e61-CommandLines"></a>
### `CommandLines`

The cell's commands, shown as a shell would echo them.
`$ ` prefixes are decoration, not text: the document holds the bare
command, and this is the panel standing in for source that is currently
folded away. Clicking anywhere in here goes back to that source, because
"I want to change this command" is the obvious next thought.

<storycode:copy id="f-c3c409b02e61-CommandLines">
/**
 * The cell's commands, shown as a shell would echo them.
 *
 * `$ ` prefixes are decoration, not text: the document holds the bare
 * command, and this is the panel standing in for source that is currently
 * folded away. Clicking anywhere in here goes back to that source, because
 * "I want to change this command" is the obvious next thought.
 */
function CommandLines({ command }: { command: string }) {
  const lines = command.split("\n");
  return (
    <pre className="cell-command" data-testid="cell-command">
      {lines.map((line, i) => (
        <span key={i} className="cell-command-line">
          <span className="cell-command-prompt">$ </span>
          {line}
          {"\n"}
        </span>
      ))}
    </pre>
  );
}

</storycode:copy>

### Assemble `apps/web/src/components/CellPanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/CellPanel.tsx"><storycode:paste select="#f-c3c409b02e61-looksLikeSvg"/><storycode:paste select="#f-c3c409b02e61-IngestedChip"/><storycode:paste select="#f-c3c409b02e61-CellPanelProps"/><storycode:paste select="#f-c3c409b02e61-CellPanel"/><storycode:paste select="#f-c3c409b02e61-CommandLines"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-85e341faa089"></a>
## `apps/web/src/components/CellPanel.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [An Ingested Cell Names Itself In The Document View](../../../../../guarantees/editor-intelligence/an-ingested-cell-names-itself-in-the-document-view/contract.md), [A run result is distinct from what was checked](../../../../../guarantees/execution/a-run-result-is-distinct-from-what-was-checked/contract.md).

<a id="f-85e341faa089-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-85e341faa089-expression_statement">
// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { CellPanel } from "./CellPanel";
import { hasReplay } from "../lib/railActions";
import type { ExecBlock, TranscriptEvent } from "../api/types";

afterEach(cleanup);

</storycode:copy>
<a id="f-85e341faa089-transcript"></a>
### `transcript`

<storycode:copy id="f-85e341faa089-transcript">
const transcript: TranscriptEvent[] = [
  { t: 0, kind: "cmd", data: "sort fruit.csv" },
  { t: 300, kind: "out", data: "apple,3\nbanana,5\n" },
  { t: 350, kind: "exit", code: 0 },
];

</storycode:copy>
<a id="f-85e341faa089-cell"></a>
### `cell`

<storycode:copy id="f-85e341faa089-cell">
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

</storycode:copy>
<a id="f-85e341faa089-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-85e341faa089-expression_statement-2">
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

</storycode:copy>
<a id="f-85e341faa089-expression_statement-3"></a>
### `expression_statement`

Protects docs/guarantees/editor-intelligence/an-ingested-cell-names-itself-in-the-document-view.md

<storycode:copy id="f-85e341faa089-expression_statement-3">
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

</storycode:copy>
<a id="f-85e341faa089-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-85e341faa089-expression_statement-4">
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

</storycode:copy>
<a id="f-85e341faa089-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-85e341faa089-expression_statement-5">
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
</storycode:copy>

### Assemble `apps/web/src/components/CellPanel.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/CellPanel.test.tsx"><storycode:paste select="#f-85e341faa089-expression_statement"/><storycode:paste select="#f-85e341faa089-transcript"/><storycode:paste select="#f-85e341faa089-cell"/><storycode:paste select="#f-85e341faa089-expression_statement-2"/><storycode:paste select="#f-85e341faa089-expression_statement-3"/><storycode:paste select="#f-85e341faa089-expression_statement-4"/><storycode:paste select="#f-85e341faa089-expression_statement-5"/></storycode:file>


</storycode:doc>
