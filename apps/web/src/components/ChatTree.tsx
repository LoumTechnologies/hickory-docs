// The conversation zoomed out: every turn as a node, each joined to the turn
// it continued from, newest at the top — the same lane layout the git pane
// uses for commits, because a conversation with rewinds IS a small DAG. The
// highlighted path is the branch the dock is showing; clicking a node makes
// it the tip (a rewind), which is how you go back to where you branched off.

import { useMemo, useState } from "react";
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

/** Place parents below their children, with one horizontal slot per leaf.
 * Iterative traversal keeps deep conversations off the JS call stack. */
export function botanicalLayout(turns: readonly AgentTurn[]) {
  const byId = new Map(turns.map(turn => [turn.id, turn]));
  const children = new Map<string, string[]>();
  for (const turn of turns) {
    if (turn.parent_id && byId.has(turn.parent_id)) {
      const siblings = children.get(turn.parent_id) ?? [];
      siblings.push(turn.id);
      children.set(turn.parent_id, siblings);
    }
  }
  const nodes = new Map<string, { x: number; depth: number; parent: string | null }>();
  const order: string[] = [];
  let leaves = 0;
  let depth = 0;
  // Visit true roots first. Any remaining component is malformed/cyclic;
  // sever its back edge for display without changing retained conversation data.
  const seeds = [...turns.filter(t => !t.parent_id || !byId.has(t.parent_id)), ...turns];
  for (const seed of seeds) {
    if (nodes.has(seed.id)) continue;
    const stack = [{ id: seed.id, parent: null as string | null, depth: 0, exit: false }];
    while (stack.length) {
      const item = stack.pop()!;
      if (item.exit) {
        const childNodes = (children.get(item.id) ?? [])
          .filter(id => nodes.get(id)?.parent === item.id).map(id => nodes.get(id)!);
        nodes.get(item.id)!.x = childNodes.length
          ? (childNodes[0].x + childNodes[childNodes.length - 1].x) / 2
          : 106 + leaves++ * 196;
        continue;
      }
      if (nodes.has(item.id)) continue;
      nodes.set(item.id, { x: 0, depth: item.depth, parent: item.parent });
      order.push(item.id);
      depth = Math.max(depth, item.depth);
      stack.push({ ...item, exit: true });
      for (const id of [...(children.get(item.id) ?? [])].reverse()) {
        if (!nodes.has(id)) stack.push({ id, parent: item.id, depth: item.depth + 1, exit: false });
      }
    }
  }
  return { nodes, order, width: Math.max(320, leaves * 196 + 16), height: (depth + 1) * 100 + 80 };
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
  const [botanical, setBotanical] = useState(() => {
    try { return localStorage.getItem("hickory.conversation-tree") === "botanical"; }
    catch { return false; }
  });
  const [zoom, setZoom] = useState(1);
  const pickStyle = (next: boolean) => {
    setBotanical(next);
    try { localStorage.setItem("hickory.conversation-tree", next ? "botanical" : "compact"); }
    catch { /* The current view still works without storage. */ }
  };
  const tree = useMemo(() => botanicalLayout(botanical ? turns : []), [turns, botanical]);
  const turnsById = useMemo(() => new Map(turns.map(t => [t.id, t])), [turns]);
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
    <section aria-label="Conversation branches">
      <div className="hickory-tree__controls" role="group" aria-label="Conversation tree style">
        <button type="button" className="btn-link" aria-pressed={!botanical} onClick={() => pickStyle(false)}>Compact</button>
        <button type="button" className="btn-link" aria-pressed={botanical} onClick={() => pickStyle(true)}>Hickory tree · experiment</button>
        {botanical && <>
          <button type="button" className="btn-link" aria-label="Zoom out tree" disabled={zoom <= .5} onClick={() => setZoom(z => Math.max(.5, z - .25))}>−</button>
          <button type="button" className="btn-link" aria-label="Reset tree zoom" onClick={() => setZoom(1)}>{Math.round(zoom * 100)}%</button>
          <button type="button" className="btn-link" aria-label="Zoom in tree" disabled={zoom >= 1.5} onClick={() => setZoom(z => Math.min(1.5, z + .25))}>+</button>
        </>}
      </div>
      {botanical ? <>
        <p className="muted">Roots below, new growth above. Choose a turn to continue its branch.</p>
        <div className="hickory-tree__canvas" tabIndex={0} role="region" aria-label="Hickory conversation tree">
          <svg width={tree.width * zoom} height={tree.height * zoom} viewBox={`0 0 ${tree.width} ${tree.height}`}>
            {tree.order.map(id => {
              const node = tree.nodes.get(id)!;
              const y = tree.height - 70 - node.depth * 100;
              const parent = node.parent ? tree.nodes.get(node.parent) : null;
              const py = parent ? tree.height - 70 - parent.depth * 100 : tree.height - 15;
              const px = parent?.x ?? node.x;
              return <path key={id} aria-hidden="true" className={`hickory-tree__branch${onPath.has(id) ? " hickory-tree__branch--on" : ""}`}
                strokeWidth={parent ? 4 : 9} d={`M ${px} ${py} C ${px} ${(py + y) / 2}, ${node.x} ${(py + y) / 2}, ${node.x} ${y}`} />;
            })}
            {tree.order.map(id => {
              const turn = turnsById.get(id)!;
              const node = tree.nodes.get(id)!;
              const y = tree.height - 70 - node.depth * 100;
              return <g key={id}>
                <path aria-hidden="true" className="hickory-tree__leaf" d={`M ${node.x + 76} ${y - 18} q 4 -28 27 -28 q 0 23 -27 28 Z`} />
                <foreignObject x={node.x - 86} y={y - 29} width={172} height={58}>
                  <button type="button" className="hickory-tree__node" aria-current={id === tip ? "true" : undefined}
                    title={turn.prompt} onClick={() => onSelect(id)}>
                    <span>{turn.prompt || "Untitled turn"}</span>
                    <span className="muted">{id === tip ? "Current · " : ""}{turn.status === "running" ? "running" : turn.status === "error" ? "failed" : turn.model}</span>
                  </button>
                </foreignObject>
              </g>;
            })}
          </svg>
        </div>
      </> : (
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
      )}
    </section>
  );
}
