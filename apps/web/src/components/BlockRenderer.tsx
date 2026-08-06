import { useMemo } from "react";
import type { Block, ExecBlock } from "../api/types";
import { stdoutOf } from "../lib/transcript";
import { StatusChip } from "./StatusChip";
import { TranscriptPlayer } from "./TranscriptPlayer";

export interface ExecCellProps {
  block: ExecBlock;
  running?: boolean;
  onRun?: (execId: string) => void;
  onSelectSpan?: (span: [number, number]) => void;
}

function looksLikeSvg(s: string): boolean {
  const t = s.trim();
  return t.startsWith("<svg") && t.endsWith("</svg>");
}

export function ExecCell({ block, running, onRun, onSelectSpan }: ExecCellProps) {
  const transcript = block.transcript ?? [];
  const stdout = useMemo(() => stdoutOf(transcript), [transcript]);
  const svgFigure = useMemo(() => (looksLikeSvg(stdout) ? stdout.trim() : null), [stdout]);

  return (
    <section className="cell cell-exec" data-testid={`exec-${block.id}`}>
      <header className="cell-header" onClick={() => onSelectSpan?.(block.span)}>
        <span className="cell-container" title={block.image ?? "host"}>
          {block.container}
          {block.image ? ` · ${block.image}` : ""}
        </span>
        <StatusChip status={block.status} running={running} />
        <button
          className="btn btn-run"
          disabled={running}
          onClick={(e) => {
            e.stopPropagation();
            onRun?.(block.id);
          }}
        >
          Run
        </button>
      </header>
      <pre className="cell-command">
        <code>{block.command}</code>
      </pre>
      {block.expect && (
        <details className="cell-expect">
          <summary>expected output ({block.expect.match})</summary>
          <pre>
            <code>{block.expect.body}</code>
          </pre>
        </details>
      )}
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
    </section>
  );
}

export interface BlockRendererProps {
  blocks: Block[];
  runningCells?: Set<string>;
  onRunCell?: (execId: string) => void;
  onSelectSpan?: (span: [number, number]) => void;
}

export function BlockRenderer({
  blocks,
  runningCells,
  onRunCell,
  onSelectSpan,
}: BlockRendererProps) {
  return (
    <div className="blocks">
      {blocks.map((block, i) => {
        switch (block.kind) {
          case "prose":
            return (
              <div
                key={i}
                className="block-prose"
                onClick={() => onSelectSpan?.(block.span)}
                dangerouslySetInnerHTML={{ __html: block.html }}
              />
            );
          case "exec":
            return (
              <ExecCell
                key={block.id}
                block={block}
                running={runningCells?.has(block.id)}
                onRun={onRunCell}
                onSelectSpan={onSelectSpan}
              />
            );
          case "file":
            return (
              <section key={i} className="cell cell-file" onClick={() => onSelectSpan?.(block.span)}>
                <header className="cell-header">
                  <span className="cell-container">{block.path}</span>
                  <span className="cell-lang">{block.language}</span>
                </header>
                <pre className="cell-command">
                  <code>{block.body}</code>
                </pre>
              </section>
            );
          case "session-user":
          case "session-assistant":
          case "session-observation": {
            const role = block.kind.replace("session-", "");
            return (
              <section
                key={i}
                className={`session session-${role}`}
                onClick={() => onSelectSpan?.(block.span)}
              >
                <span className="session-role">{role}</span>
                <pre className="session-body">{block.body}</pre>
              </section>
            );
          }
        }
      })}
    </div>
  );
}
