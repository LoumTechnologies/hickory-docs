import { useMemo, useState } from "react";
import type { ExecBlock } from "../api/types";
import { diffLines } from "../lib/diff";
import { stdoutOf } from "../lib/transcript";
import { StatusChip } from "./StatusChip";
import { TranscriptPlayer } from "./TranscriptPlayer";

function looksLikeSvg(s: string): boolean {
  const t = s.trim();
  return t.startsWith("<svg") && t.endsWith("</svg>");
}

export interface CellPanelProps {
  /** The rendered exec block backing this cell (undefined until the server's
   * render catches up with a freshly typed cell). */
  block?: ExecBlock;
  running?: boolean;
  onRun?: (execId: string) => void;
}

/**
 * The strip below an exec cell in the Document view: run button, status chip,
 * and — only when it adds information — output. The cell's source already
 * shows the command and (when verified) the expect body IS the output, so:
 *
 *  - status ok + expect: no transcript by default (the verified expect body
 *    in the source is the single visible copy); a "Replay" toggle reveals the
 *    animated transcript on demand, and the bar shows "✓ output verified".
 *  - status failed + expect: the actual output earns its second copy — shown
 *    as a compact line diff against the expect body; transcript behind Replay.
 *  - no expect: the transcript IS the output — visible as before.
 */
export function CellPanel({ block, running, onRun }: CellPanelProps) {
  const transcript = block?.transcript ?? [];
  const stdout = useMemo(() => stdoutOf(transcript), [transcript]);
  const svgFigure = useMemo(() => (looksLikeSvg(stdout) ? stdout.trim() : null), [stdout]);
  const [replay, setReplay] = useState(false);

  const verified = !running && block?.status === "ok" && !!block?.expect;
  const failedExpect =
    !running && block?.status === "failed" && !!block?.expect && transcript.length > 0;
  const diff = useMemo(
    () => (failedExpect && block?.expect ? diffLines(block.expect.body, stdout) : null),
    [failedExpect, block?.expect, stdout],
  );

  if (!block) {
    return (
      <div className="cell-panel" data-testid="cell-panel-pending">
        <span className="muted">cell not yet known to the server — save to sync</span>
      </div>
    );
  }

  const collapsible = (verified || failedExpect) && transcript.length > 0;
  const showTranscript =
    running || (transcript.length > 0 && (collapsible ? replay : !verified && !failedExpect));

  return (
    <div className="cell-panel" data-testid={`cell-panel-${block.id}`}>
      <div className="cell-panel-bar">
        <span className="cell-container" title={block.image ?? "host"}>
          {block.container}
          {block.image ? ` · ${block.image}` : ""}
        </span>
        <StatusChip status={block.status} running={running} />
        {verified && (
          <span className="cell-verified" title="the expect block above is the verified output">
            ✓ output verified
          </span>
        )}
        {collapsible && (
          <button
            className={`btn btn-ghost${replay ? " on" : ""}`}
            aria-pressed={replay}
            onClick={() => setReplay((v) => !v)}
          >
            Replay
          </button>
        )}
        <button
          className="btn btn-run"
          disabled={running}
          onClick={() => onRun?.(block.id)}
        >
          Run
        </button>
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
      {showTranscript && <TranscriptPlayer events={transcript} live={running} />}
      {svgFigure && !running && (
        <figure
          className="cell-figure"
          // Figure output produced by the executed cell itself (inline SVG).
          dangerouslySetInnerHTML={{ __html: svgFigure }}
        />
      )}
    </div>
  );
}
