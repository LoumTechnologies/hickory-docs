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

  const verified = !running && block?.status === "ok" && !!block?.expect;
  const failedExpect =
    !running && block?.status === "failed" && !!block?.expect && transcript.length > 0;
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

  useUiReading("cell rendering", { id: block?.id ?? null, command: command ?? null, status: block?.status ?? "save_required", running: !!running, container: block?.container ?? null, image: block?.image ?? null, saved_output: block && savedOnly ? rich.text : null, expected_output_matches: verified, diff, replay_available: collapsible, transcript: block && showTranscript ? displayTranscript : null, transcript_hidden: !!block && !showTranscript && transcript.length > 0, rich_outputs: block && !running ? rich.outputs : [], svg: block && !running ? svgFigure : null, ingested: block?.ingested ?? null, scope: "Current cell view; recorded output/expectation match does not verify prose or certify current source/run freshness" }, block);

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
        <StatusChip status={block.status} running={running} />
        {savedOnly && <span className="muted">Saved output</span>}
        {verified && (
          <span className="cell-verified" data-tip="The recorded stdout matches this cell’s expectation; this does not assert that the prose is correct.">
            ✓ Expected output matches
          </span>
        )}
        {block.ingested && <IngestedChip ingested={block.ingested} />}
      </div>
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

