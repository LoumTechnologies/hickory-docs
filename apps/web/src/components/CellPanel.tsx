import { useMemo } from "react";
import type { ExecBlock } from "../api/types";
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
 * expect summary, animated transcript player, and any SVG figure the cell
 * printed. Rendered into a CodeMirror block widget via a React portal.
 */
export function CellPanel({ block, running, onRun }: CellPanelProps) {
  const transcript = block?.transcript ?? [];
  const stdout = useMemo(() => stdoutOf(transcript), [transcript]);
  const svgFigure = useMemo(() => (looksLikeSvg(stdout) ? stdout.trim() : null), [stdout]);

  if (!block) {
    return (
      <div className="cell-panel" data-testid="cell-panel-pending">
        <span className="muted">cell not yet known to the server — save to sync</span>
      </div>
    );
  }

  return (
    <div className="cell-panel" data-testid={`cell-panel-${block.id}`}>
      <div className="cell-panel-bar">
        <span className="cell-container" title={block.image ?? "host"}>
          {block.container}
          {block.image ? ` · ${block.image}` : ""}
        </span>
        <StatusChip status={block.status} running={running} />
        {block.expect && (
          <span className="cell-expect-chip" title={block.expect.body}>
            expects {block.expect.match}
          </span>
        )}
        <button
          className="btn btn-run"
          disabled={running}
          onClick={() => onRun?.(block.id)}
        >
          Run
        </button>
      </div>
      {(transcript.length > 0 || running) && (
        <TranscriptPlayer events={transcript} live={running} />
      )}
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
