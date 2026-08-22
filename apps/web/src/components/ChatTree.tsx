// The conversation zoomed out: every turn as a node, each joined to the turn
// it continued from, newest at the top — the same lane layout the git pane
// uses for commits, because a conversation with rewinds IS a small DAG. The
// highlighted path is the branch the dock is showing; clicking a node makes
// it the tip (a rewind), which is how you go back to where you branched off.

import { useMemo } from "react";
import type { AgentTurn } from "../api/types";
import { graphWidth, laneColor, layout } from "../lib/gitGraph";

const ROW = 30;
const LANE = 14;
const x = (lane: number) => LANE * (lane + 0.5);
function edgePath(from: number, to: number): string {
  const x0 = x(from);
  const x1 = x(to);
  if (from === to) return `M ${x0} 0 L ${x0} ${ROW}`;
  const mid = ROW / 2;
  return `M ${x0} 0 C ${x0} ${mid}, ${x1} ${mid}, ${x1} ${ROW}`;
}

/** The ids on the path from the root to `tip`, as a set. */
export function pathTo(
  turns: readonly AgentTurn[],
  tip: string | null,
): Set<string> {
  const out = new Set<string>();
  let cursor = tip;
  const byId = new Map(turns.map((t) => [t.id, t]));
  while (cursor && byId.has(cursor) && !out.has(cursor)) {
    out.add(cursor);
    cursor = byId.get(cursor)!.parent_id;
  }
  return out;
}

export function ChatTree({
  turns,
  tip,
  onSelect,
}: {
  turns: readonly AgentTurn[];
  tip: string | null;
  onSelect: (id: string) => void;
}) {
  // gitGraph wants newest first, each node naming its parents.
  const ordered = useMemo(() => [...turns].reverse(), [turns]);
  const rows = useMemo(
    () =>
      layout(
        ordered.map((t) => ({
          sha: t.id,
          parents: t.parent_id ? [t.parent_id] : [],
        })),
      ),
    [ordered],
  );
  const onPath = useMemo(() => pathTo(turns, tip), [turns, tip]);
  const graphPx = (graphWidth(rows) + 1) * LANE;
  if (turns.length === 0)
    return (
      <p className="muted chat-empty">No turns yet — nothing to zoom out of.</p>
    );
  return (
    <ol
      className="chat-tree"
      style={{ ["--chat-graph" as string]: `${graphPx}px` }}
      aria-label="Conversation tree"
    >
      {ordered.map((turn, i) => {
        const row = rows[i];
        const here = turn.id === tip;
        const lit = onPath.has(turn.id);
        return (
          <li
            key={turn.id}
            className={`chat-tree__row${here ? " chat-tree__row--tip" : ""}${lit ? " chat-tree__row--on" : ""}`}
          >
            <svg
              className="chat-tree__graph"
              width={graphPx}
              height={ROW}
              aria-hidden="true"
            >
              {row.through.map((line, j) => (
                <path
                  key={j}
                  className={`git-edge git-edge--c${laneColor(line.from)}`}
                  d={edgePath(line.from, line.to)}
                />
              ))}
              <circle
                className={`git-node git-node--c${laneColor(row.lane)}${here ? " chat-tree__node--tip" : ""}`}
                cx={x(row.lane)}
                cy={ROW / 2}
                r={here ? 5.5 : 4}
              />
            </svg>
            <button
              type="button"
              className="chat-tree__turn"
              onClick={() => onSelect(turn.id)}
              data-tip={
                here
                  ? "The current tip"
                  : "Make this the tip — continue from here"
              }
            >
              <span className="chat-tree__prompt">
                {turn.prompt.length > 80
                  ? `${turn.prompt.slice(0, 80)}…`
                  : turn.prompt}
              </span>
              <span className="chat-tree__meta muted">
                {turn.status === "running"
                  ? "running"
                  : turn.status === "error"
                    ? "failed"
                    : turn.model}
              </span>
            </button>
          </li>
        );
      })}
    </ol>
  );
}
