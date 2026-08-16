// The shell: the tile tree, rendered.
//
// Deliberately dull. Every decision about what happens to a layout lives in
// `layout.ts` as a pure function; this file turns the result into boxes,
// draws the dividers, and hands each tab's content to whoever knows how to
// render it. If something interesting appears here, it belongs next door.
//
// See docs/specs/freeform/shell-layouts.md.

import { useCallback, useEffect, useRef, type ReactNode } from "react";

import {
  activate,
  closePane,
  closeTab,
  focus as focusPane,
  panes as panesOf,
  resize,
  split,
  type Layout,
  type Node,
  type Pane,
  type Tab,
} from "./layout";

export interface ShellViewProps {
  layout: Layout;
  onLayout: (next: Layout) => void;
  /** Render one tab's content. The shell knows nothing about what is inside. */
  render: (tab: Tab, pane: Pane) => ReactNode;
  /** What an empty pane says. */
  empty?: ReactNode;
}

export function ShellView({ layout, onLayout, render, empty }: ShellViewProps) {
  useShellKeys(layout, onLayout);
  return (
    <div className="shell">
      <ShellNode
        node={layout.root}
        layout={layout}
        onLayout={onLayout}
        render={render}
        empty={empty}
      />
    </div>
  );
}

interface NodeProps extends Omit<ShellViewProps, "layout"> {
  node: Node;
  layout: Layout;
}

function ShellNode({ node, layout, onLayout, render, empty }: NodeProps) {
  if (node.type === "pane") {
    return (
      <PaneBox
        pane={node}
        layout={layout}
        onLayout={onLayout}
        render={render}
        empty={empty}
      />
    );
  }
  return (
    <SplitBox node={node} layout={layout} onLayout={onLayout} render={render} empty={empty} />
  );
}

function SplitBox({ node, layout, onLayout, render, empty }: NodeProps & { node: { type: "split" } & Node }) {
  const box = useRef<HTMLDivElement | null>(null);
  const split = node as Extract<Node, { type: "split" }>;
  const horizontal = split.direction === "row";

  // Dragging a divider is the one thing here that touches pixels: everything
  // else is fractions, and the conversion happens once, at the edge.
  const startDrag = useCallback(
    (index: number, event: React.PointerEvent) => {
      event.preventDefault();
      const container = box.current;
      if (!container) return;
      const rect = container.getBoundingClientRect();
      const total = horizontal ? rect.width : rect.height;
      const before = split.sizes.slice(0, index).reduce((sum, size) => sum + size, 0);
      const pair = split.sizes[index] + split.sizes[index + 1];

      const move = (e: PointerEvent) => {
        const at = horizontal ? (e.clientX - rect.left) / total : (e.clientY - rect.top) / total;
        // A pane you cannot see is a pane you cannot get back, so neither side
        // of a divider is allowed to reach zero.
        const first = Math.max(0.08, Math.min(pair - 0.08, at - before));
        const sizes = [...split.sizes];
        sizes[index] = first;
        sizes[index + 1] = pair - first;
        onLayout(resize(layout, split.id, sizes));
      };
      const up = () => {
        window.removeEventListener("pointermove", move);
        window.removeEventListener("pointerup", up);
      };
      window.addEventListener("pointermove", move);
      window.addEventListener("pointerup", up);
    },
    [horizontal, layout, onLayout, split.id, split.sizes],
  );

  return (
    <div
      ref={box}
      className={`shell-split shell-split--${split.direction}`}
      style={{
        gridTemplateColumns: horizontal ? split.sizes.map((s) => `${s}fr`).join(" 4px ") : undefined,
        gridTemplateRows: horizontal ? undefined : split.sizes.map((s) => `${s}fr`).join(" 4px "),
      }}
    >
      {split.children.map((child, index) => (
        <FragmentWithDivider
          key={child.id}
          last={index === split.children.length - 1}
          direction={split.direction}
          onDrag={(event) => startDrag(index, event)}
        >
          <ShellNode
            node={child}
            layout={layout}
            onLayout={onLayout}
            render={render}
            empty={empty}
          />
        </FragmentWithDivider>
      ))}
    </div>
  );
}

function FragmentWithDivider({
  children,
  last,
  direction,
  onDrag,
}: {
  children: ReactNode;
  last: boolean;
  direction: "row" | "column";
  onDrag: (event: React.PointerEvent) => void;
}) {
  return (
    <>
      {children}
      {!last && (
        <div
          className={`shell-divider shell-divider--${direction}`}
          onPointerDown={onDrag}
          role="separator"
          aria-orientation={direction === "row" ? "vertical" : "horizontal"}
          aria-label="Resize"
        />
      )}
    </>
  );
}

function PaneBox({
  pane,
  layout,
  onLayout,
  render,
  empty,
}: Omit<NodeProps, "node"> & { pane: Pane }) {
  const active = pane.tabs[pane.active] ?? null;
  const isFocused = layout.focus === pane.id;

  return (
    <section
      className={`shell-pane${isFocused ? " shell-pane--focus" : ""}`}
      onPointerDownCapture={() => {
        if (!isFocused) onLayout(focusPane(layout, pane.id));
      }}
      aria-label={pane.region ? `${pane.region} pane` : "Editor pane"}
    >
      <header className="shell-tabs">
        {/* The region's name, when a layout declared one: a pane that belongs
            to something should say what, or the arrangement is a mystery. */}
        {pane.region && <span className="shell-region">{pane.region}</span>}
        <div className="shell-tabstrip" role="tablist">
          {pane.tabs.map((tab, index) => (
            <span key={tab.id} className={`shell-tab${index === pane.active ? " shell-tab--on" : ""}`}>
              <button
                type="button"
                role="tab"
                aria-selected={index === pane.active}
                onClick={() => onLayout(activate(layout, pane.id, index))}
                title={tab.target}
              >
                {tab.title ?? tab.target.split("/").pop()}
              </button>
              <button
                type="button"
                className="shell-tab__close"
                aria-label={`Close ${tab.title ?? tab.target}`}
                onClick={(event) => {
                  event.stopPropagation();
                  onLayout(closeTab(layout, pane.id, tab.id));
                }}
              >
                ×
              </button>
            </span>
          ))}
        </div>
        <div className="shell-pane__controls">
          <button
            type="button"
            title="Split right"
            aria-label="Split right"
            onClick={() => onLayout(split(layout, pane.id, "row"))}
          >
            ⇥
          </button>
          <button
            type="button"
            title="Split down"
            aria-label="Split down"
            onClick={() => onLayout(split(layout, pane.id, "column"))}
          >
            ⇩
          </button>
          <button
            type="button"
            title="Close this pane"
            aria-label="Close pane"
            onClick={() => onLayout(closePane(layout, pane.id))}
          >
            ✕
          </button>
        </div>
      </header>
      <div className="shell-pane__body">
        {active ? render(active, pane) : <div className="shell-pane__empty">{empty}</div>}
      </div>
    </section>
  );
}


/**
 * The keys an editor is expected to have.
 *
 * Deliberately few, and all of them about the arrangement rather than the
 * text: anything that edits belongs to the editor in the pane, which already
 * has its own keymap and its own idea of what has focus.
 */
function useShellKeys(layout: Layout, onLayout: (next: Layout) => void) {
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const chord = event.metaKey || event.ctrlKey;
      if (!chord) return;
      const pane = layout.focus;
      if (event.key === "\\") {
        // Cmd-\ splits right, Cmd-Shift-\ splits down: the pair VS Code and
        // Zed both use, so the muscle memory people arrive with works.
        event.preventDefault();
        onLayout(split(layout, pane, event.shiftKey ? "column" : "row"));
        return;
      }
      if (event.key === "w") {
        const current = layout.root;
        const found = panesOf(current).find((candidate) => candidate.id === pane);
        const tab = found?.tabs[found.active];
        if (!tab) return;
        event.preventDefault();
        onLayout(closeTab(layout, pane, tab.id));
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [layout, onLayout]);
}
