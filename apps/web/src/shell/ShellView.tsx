// The shell: the tile tree, rendered.
//
// Deliberately dull. Every decision about what happens to a layout lives in
// `layout.ts` as a pure function; this file turns the result into boxes,
// draws the dividers, and hands each tab's content to whoever knows how to
// render it. If something interesting appears here, it belongs next door.
//
// See docs/specs/freeform/shell-layouts.md.

import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";

import {
  activate,
  closePane,
  closeTab,
  collapsePane,
  expandPane,
  focus as focusPane,
  paneById,
  panes as panesOf,
  resize,
  split,
  type Layout,
  type Node,
  type Pane,
  type Tab,
} from "./layout";
import { dropOnCollapsed, moveTab, moveTabToIndex, zoneAt, type DropZone } from "./dragDrop";
import { dividerLocked, gridTracks, groupTabsByFolder, tabIcon } from "./tabStrip";
import { CHANNEL_WIDTH_DEFAULT, clampChannelWidth } from "../lib/channelWidth";
import { attachFlashListener } from "../lib/flashTab";
import type { TabStyle } from "../lib/tabStyle";

/**
 * An "open here" port: a small button living IN a divider (or the edge rail
 * when there is no divider) that opens a file the ribbons point at but no
 * pane is showing. Part of the shell's layout on purpose — the previous
 * design floated labels over pane content, which put a click target on top
 * of text that also wanted the click.
 */
export interface ShellPort {
  /** Stable identity; the ribbon overlay finds the button by this value
   * through the `data-ribbon-port` attribute. */
  id: string;
  label: string;
  title: string;
  onOpen: () => void;
}

export interface ShellViewProps {
  layout: Layout;
  onLayout: (next: Layout) => void;
  /** Render one tab's content. The shell knows nothing about what is inside. */
  render: (tab: Tab, pane: Pane) => ReactNode;
  /** What an empty pane says. */
  empty?: ReactNode;
  /** Open project search (Mod-Shift-F). The shell only owns the key: what a
   * search panel is, and over what, belongs to whoever mounted the shell. */
  onSearch?: () => void;
  /** "Open here" ports for files ribbons reach but no pane shows. Rendered
   * in the root's first vertical divider, or an edge rail when there is none. */
  ports?: readonly ShellPort[];
  /** Tabs across the top of each pane (default), or down a left sidebar
   * grouped by folder. Persisted by lib/tabStyle.ts; the shell only obeys. */
  tabStyle?: TabStyle;
  /** Width (px) of the channel between panes — the grid track where ribbons,
   * braces, and their links live. Persisted by lib/channelWidth.ts; the
   * shell clamps and obeys. */
  channelWidth?: number;
}

export function ShellView({
  layout,
  onLayout,
  render,
  empty,
  onSearch,
  ports,
  tabStyle = "top",
  channelWidth = CHANNEL_WIDTH_DEFAULT,
}: ShellViewProps) {
  useShellKeys(layout, onLayout, onSearch);
  const dragging = useTabDrag(layout, onLayout);
  // "Open" on an already-open file flashes its tab (lib/flashTab.ts) —
  // wherever the tab renders: top strip, side tree, or a collapsed strip.
  const rootRef = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    return attachFlashListener(root);
  }, []);
  // The ports live between panes when there IS a between: the root row
  // split's first divider. A single pane (or a column split) has no vertical
  // divider, so a thin rail at the right edge stands in — still part of the
  // layout, never floating over content.
  const rootHostsPorts = layout.root.type === "split" && layout.root.direction === "row";
  const stacked = ports && ports.length > 0 ? ports : undefined;
  return (
    // The modifier while a tab is in flight lets CSS switch the ribbon
    // overlay's painted bands to pointer-events: none, so hit-testing the
    // drop target never lands on a band that would otherwise take the click.
    <div ref={rootRef} className={`shell${dragging.drag ? " shell--tab-drag" : ""}`}>
      <ShellNode
        node={layout.root}
        layout={layout}
        onLayout={onLayout}
        render={render}
        empty={empty}
        ports={rootHostsPorts ? stacked : undefined}
        dragging={dragging}
        tabStyle={tabStyle}
        channelWidth={channelWidth}
      />
      {!rootHostsPorts && stacked && (
        <div className="shell-ports-rail" role="toolbar" aria-label="Files ribbons lead to">
          <PortStack ports={stacked} />
        </div>
      )}
    </div>
  );
}

/**
 * The ports, stacked vertically. One small button per unopened target — a
 * divider spans the pane's full height, so even a dozen 20px buttons stay
 * legible where a horizontal row would not fit at all.
 */
function PortStack({ ports }: { ports: readonly ShellPort[] }) {
  return (
    <div className="shell-ports">
      {ports.map((port) => (
        <button
          key={port.id}
          type="button"
          className="shell-port"
          data-ribbon-port={port.id}
          data-tip={port.title}
          aria-label={port.title}
          // A port sits inside the divider, whose pointerdown starts a
          // resize drag: pressing the button must not also grab the divider.
          onPointerDown={(event) => event.stopPropagation()}
          onClick={port.onOpen}
        >
          {port.label.slice(0, 1).toUpperCase()}
        </button>
      ))}
    </div>
  );
}

interface NodeProps extends Omit<ShellViewProps, "layout"> {
  node: Node;
  layout: Layout;
  dragging: TabDragging;
}

function ShellNode({ node, layout, onLayout, render, empty, ports, dragging, tabStyle, channelWidth }: NodeProps) {
  if (node.type === "pane") {
    if (node.collapsed) {
      return <CollapsedStrip pane={node} layout={layout} onLayout={onLayout} dragging={dragging} />;
    }
    return (
      <PaneBox
        pane={node}
        layout={layout}
        onLayout={onLayout}
        render={render}
        empty={empty}
        dragging={dragging}
        tabStyle={tabStyle}
      />
    );
  }
  return (
    <SplitBox
      node={node}
      layout={layout}
      onLayout={onLayout}
      render={render}
      empty={empty}
      ports={ports}
      dragging={dragging}
      tabStyle={tabStyle}
      channelWidth={channelWidth}
    />
  );
}

function SplitBox({ node, layout, onLayout, render, empty, ports, dragging, tabStyle, channelWidth }: NodeProps & { node: { type: "split" } & Node }) {
  const box = useRef<HTMLDivElement | null>(null);
  const split = node as Extract<Node, { type: "split" }>;
  const horizontal = split.direction === "row";

  // Dragging a divider is the one thing here that touches pixels: everything
  // else is fractions, and the conversion happens once, at the edge.
  const startDrag = useCallback(
    (index: number, event: React.PointerEvent) => {
      // A collapsed pane's strip is a fixed track: the divider beside it has
      // nothing it may resize, so the drag never starts.
      if (dividerLocked(split.children, index)) return;
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
    [horizontal, layout, onLayout, split.id, split.sizes, split.children],
  );

  return (
    <div
      ref={box}
      className={`shell-split shell-split--${split.direction}`}
      style={{
        // A real track, not a hairline: the channel between panes is where
        // ribbons, braces, and their connecting lines live — they need room
        // to be seen and clicked. The divider paints itself as a thin
        // centred line, so the width reads as space, not chrome. How MUCH
        // room is the user's channel-width setting (lib/channelWidth.ts),
        // clamped here so a bad value can never wreck the grid.
        // A collapsed pane's track is a fixed 36px strip; everything else
        // keeps its fraction (tabStrip.ts's gridTracks decides which).
        gridTemplateColumns: horizontal
          ? gridTracks(split.children, split.sizes).join(` ${clampChannelWidth(channelWidth)}px `)
          : undefined,
        gridTemplateRows: horizontal
          ? undefined
          : gridTracks(split.children, split.sizes).join(` ${clampChannelWidth(channelWidth)}px `),
      }}
    >
      {split.children.map((child, index) => (
        <FragmentWithDivider
          key={child.id}
          last={index === split.children.length - 1}
          direction={split.direction}
          onDrag={(event) => startDrag(index, event)}
          // The FIRST divider hosts the "open here" ports: it is the gap
          // beside whatever opened first, which is the document in every
          // arrangement the app builds. Ribbons find the buttons by
          // attribute, so where they land is a presentation choice.
          extra={index === 0 && ports ? <PortStack ports={ports} /> : undefined}
        >
          <ShellNode
            node={child}
            layout={layout}
            onLayout={onLayout}
            render={render}
            empty={empty}
            dragging={dragging}
            tabStyle={tabStyle}
            channelWidth={channelWidth}
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
  extra,
}: {
  children: ReactNode;
  last: boolean;
  direction: "row" | "column";
  onDrag: (event: React.PointerEvent) => void;
  /** Content living IN the divider — the "open here" ports. */
  extra?: ReactNode;
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
        >
          {/* The channel is mostly RIBBON territory: the divider itself is
              click-through, and only this slim centred grip (plus the ports)
              takes the pointer — so a wide channel never steals clicks from
              the bands, braces, and connector lines drawn through it. The
              grip's pointerdown bubbles here to start the resize. */}
          <div className="shell-divider__grip" />
          {extra}
        </div>
      )}
    </>
  );
}

/**
 * One tab, wherever tabs render. The data attributes are the contract with
 * the ribbon overlay (a ribbon to an inactive file terminates ON its tab),
 * so they ride along whether the tab sits in the top strip or the side tree;
 * `data-shell-tab-vertical` tells the overlay to land on the tab's facing
 * vertical edge instead of its underside.
 */
function TabChip({
  tab,
  index,
  pane,
  layout,
  onLayout,
  dragging,
  side,
  depth = 0,
}: {
  tab: Tab;
  index: number;
  pane: Pane;
  layout: Layout;
  onLayout: (next: Layout) => void;
  dragging: TabDragging;
  side: boolean;
  depth?: number;
}) {
  // Middle-click is a press and a release on the same tab; between them this
  // remembers that the press was ours (see onPointerDown below).
  const middlePress = useRef(false);
  return (
    <span
      className={`shell-tab${index === pane.active ? " shell-tab--on" : ""}${side ? " shell-tab--side" : ""}`}
      // The ribbon overlay measures these: a ribbon whose file is
      // open but not the active tab terminates ON the tab, so the
      // pointer says "it's here" instead of floating a label over
      // whatever the pane is actually showing.
      data-shell-tab-kind={tab.kind}
      data-shell-tab-target={tab.target}
      data-shell-tab-vertical={side ? "" : undefined}
      // How a side-tree drop knows which PANE index a visual row means.
      data-shell-tab-index={index}
      style={side && depth > 0 ? { paddingLeft: `${depth * 12}px` } : undefined}
      // Middle-press and middle-release on the same tab closes it: the
      // gesture every browser and editor trained people to expect. Deliberately
      // NOT `auxclick`, which is what a browser would give us — WebKitGTK, the
      // engine behind the Linux window, does not reliably raise a click event
      // for the middle button on an ordinary element, so the whole gesture has
      // to be read from the pointer stream the drag code already listens to.
      onPointerDown={(event) => {
        if (event.button === 1) {
          // Suppresses the compatibility mouse events, and with them GTK's
          // middle-click paste and the autoscroll Windows would start.
          event.preventDefault();
          middlePress.current = true;
          return;
        }
        middlePress.current = false;
        dragging.startTabDrag(pane.id, tab.id, event);
      }}
      // A press that began on some other tab and ended here is not a close:
      // the release only counts where its press landed.
      onPointerUp={(event) => {
        if (event.button !== 1 || !middlePress.current) return;
        middlePress.current = false;
        onLayout(closeTab(layout, pane.id, tab.id));
      }}
      onPointerLeave={() => {
        middlePress.current = false;
      }}
    >
      <button
        type="button"
        role="tab"
        aria-selected={index === pane.active}
        onClick={() => {
          // A drag that came back to rest is not a click: activating
          // here would undo whatever the drop just chose.
          if (dragging.suppressClick.current) return;
          onLayout(activate(layout, pane.id, index));
        }}
        data-tip={tab.target}
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
  );
}

/** The picture a strip icon draws: tiny inline SVGs and text monograms,
 * derived by tabStrip.ts — no icon library. */
function TabGlyph({ tab }: { tab: Tab }) {
  const icon = tabIcon(tab);
  if (icon.kind === "folder") {
    return (
      <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden>
        <path
          d="M1.5 3.5h4.2l1.6 2h7.2v7h-13z"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.4"
          strokeLinejoin="round"
        />
      </svg>
    );
  }
  if (icon.kind === "doc") {
    return (
      <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden>
        <path
          d="M3.5 1.5h6l3 3v10h-9z"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.4"
          strokeLinejoin="round"
        />
        <path d="M9.5 1.5v3h3" fill="none" stroke="currentColor" strokeWidth="1.4" />
      </svg>
    );
  }
  return <span className="shell-strip__mono">{icon.text}</span>;
}

/**
 * A collapsed pane: a slim vertical strip in the pane's place in the grid,
 * one icon per tab, VS Code activity-bar style. Clicking an icon expands the
 * pane AND activates that tab — a collapsed pane is a promise the contents
 * are one click away, and the click should land on the thing it named.
 */
function CollapsedStrip({
  pane,
  layout,
  onLayout,
  dragging,
}: {
  pane: Pane;
  layout: Layout;
  onLayout: (next: Layout) => void;
  dragging: TabDragging;
}) {
  const over = dragging.drag?.over;
  const dropping = over?.kind === "zone" && over.paneId === pane.id;
  return (
    <section
      className={`shell-pane shell-pane--collapsed${dropping ? " shell-pane--collapsed-drop" : ""}`}
      data-shell-pane-id={pane.id}
      aria-label={pane.region ? `${pane.region} pane (collapsed)` : "Collapsed pane"}
    >
      <button
        type="button"
        className="shell-strip__expand"
        data-tip="Expand this pane"
        aria-label="Expand pane"
        onClick={() => onLayout(expandPane(layout, pane.id))}
      >
        ›
      </button>
      <div className="shell-strip__icons">
        {pane.tabs.map((tab, index) => (
          <button
            key={tab.id}
            type="button"
            className={`shell-strip__icon${index === pane.active ? " shell-strip__icon--on" : ""}`}
            // The ribbon contract survives the collapse: a ribbon to a file
            // in a strip terminates on its icon, edge-on.
            data-shell-tab-kind={tab.kind}
            data-shell-tab-target={tab.target}
            data-shell-tab-vertical=""
            data-tip={tab.title ?? tab.target.split("/").pop() ?? tab.target}
            aria-label={`Expand and show ${tab.title ?? tab.target}`}
            onClick={() => onLayout(activate(expandPane(layout, pane.id), pane.id, index))}
          >
            <TabGlyph tab={tab} />
          </button>
        ))}
      </div>
    </section>
  );
}

function PaneBox({
  pane,
  layout,
  onLayout,
  render,
  empty,
  dragging,
  tabStyle,
}: Omit<NodeProps, "node"> & { pane: Pane }) {
  const active = pane.tabs[pane.active] ?? null;
  const isFocused = layout.focus === pane.id;
  const side = tabStyle === "side";
  const over = dragging.drag?.over;
  const dropZone = over?.kind === "zone" && over.paneId === pane.id ? over.zone : null;
  const dropCaret = over?.kind === "tabs" && over.paneId === pane.id ? over : null;
  const rows = side ? groupTabsByFolder(pane.tabs) : null;

  const body = (
    <div className="shell-pane__body">
      {active ? render(active, pane) : <div className="shell-pane__empty">{empty}</div>}
      {/* The drop preview: the third that would become a new pane, or the
          whole body for "join this pane". A tinted div, nothing more — the
          actual decision is dragDrop.ts's. */}
      {dropZone && <div className={`shell-drop shell-drop--${dropZone}`} aria-hidden />}
    </div>
  );

  return (
    <section
      className={`shell-pane${isFocused ? " shell-pane--focus" : ""}${side ? " shell-pane--side" : ""}`}
      // How the drag's hit-testing finds this pane from elementFromPoint.
      data-shell-pane-id={pane.id}
      onPointerDownCapture={() => {
        if (!isFocused) onLayout(focusPane(layout, pane.id));
      }}
      aria-label={pane.region ? `${pane.region} pane` : "Editor pane"}
    >
      <header className="shell-tabs">
        {/* The region's name, when a layout declared one: a pane that belongs
            to something should say what, or the arrangement is a mystery. */}
        {pane.region && <span className="shell-region">{pane.region}</span>}
        {side ? (
          <div className="shell-tabstrip shell-tabstrip--spacer" aria-hidden />
        ) : (
          <div className="shell-tabstrip" role="tablist">
            {pane.tabs.map((tab, index) => (
              <TabChip
                key={tab.id}
                tab={tab}
                index={index}
                pane={pane}
                layout={layout}
                onLayout={onLayout}
                dragging={dragging}
                side={false}
              />
            ))}
            {dropCaret && !dropCaret.vertical && (
              <span className="shell-drop-caret" style={{ left: dropCaret.caretAt }} aria-hidden />
            )}
          </div>
        )}
        <div className="shell-pane__controls">
          {/* The last visible pane refuses to collapse (layout.ts) — with no
              sibling to take the freed space, a strip would just float in an
              empty shell. Disabled with the reason, rather than a button
              that silently does nothing. */}
          {(() => {
            const lastVisible =
              panesOf(layout.root).filter((candidate) => !candidate.collapsed).length <= 1;
            return (
              <button
                type="button"
                disabled={lastVisible}
                data-tip={
                  lastVisible
                    ? "The only visible pane cannot collapse — there is no other pane to give the space to"
                    : "Collapse pane to icon strip"
                }
                aria-label="Collapse pane"
                onClick={() => onLayout(collapsePane(layout, pane.id))}
              >
                ⌄
              </button>
            );
          })()}
          <button
            type="button"
            data-tip="Split right"
            aria-label="Split right"
            onClick={() => onLayout(split(layout, pane.id, "row"))}
          >
            ⇥
          </button>
          <button
            type="button"
            data-tip="Split down"
            aria-label="Split down"
            onClick={() => onLayout(split(layout, pane.id, "column"))}
          >
            ⇩
          </button>
          <button
            type="button"
            data-tip="Close this pane"
            aria-label="Close pane"
            onClick={() => onLayout(closePane(layout, pane.id))}
          >
            ✕
          </button>
        </div>
      </header>
      {side && rows ? (
        <div className="shell-pane__row">
          <nav className="shell-sidetabs" role="tablist" aria-orientation="vertical">
            {rows.map((row) =>
              row.kind === "header" ? (
                // Directory headers are wayfinding, not controls: the tabs
                // beneath them are the interactive rows.
                <div
                  key={`dir:${row.depth}:${row.name}`}
                  className="shell-sidetabs__dir"
                  style={row.depth > 0 ? { paddingLeft: `${8 + row.depth * 12}px` } : undefined}
                  aria-hidden
                >
                  {row.name}
                </div>
              ) : (
                <TabChip
                  key={row.tab.id}
                  tab={row.tab}
                  index={row.index}
                  pane={pane}
                  layout={layout}
                  onLayout={onLayout}
                  dragging={dragging}
                  side
                  depth={row.depth}
                />
              ),
            )}
            {dropCaret && dropCaret.vertical && (
              <span
                className="shell-drop-caret shell-drop-caret--side"
                style={{ top: dropCaret.caretAt }}
                aria-hidden
              />
            )}
          </nav>
          {body}
        </div>
      ) : (
        body
      )}
    </section>
  );
}

// ---------------------------------------------------------------------------
// Dragging a tab
// ---------------------------------------------------------------------------

/** What the drag is over right now: a pane's body (a zone) or a tab bar
 * (a caret). `caretAt` is content-relative to the strip — an x along a top
 * tab bar, a y down a side tree (`vertical`) — so it scrolls with the tabs. */
type TabDragOver =
  | { paneId: string; kind: "zone"; zone: DropZone }
  | { paneId: string; kind: "tabs"; index: number; caretAt: number; vertical: boolean };

interface TabDrag {
  fromPane: string;
  tabId: string;
  over: TabDragOver | null;
}

interface TabDragging {
  drag: TabDrag | null;
  startTabDrag: (paneId: string, tabId: string, event: React.PointerEvent) => void;
  /** True for the tick after a real drag ends: the click that follows the
   * pointerup must not activate the tab the drag happened to end on. */
  suppressClick: React.MutableRefObject<boolean>;
}

/**
 * Pointer events, not the HTML5 drag API. The shell already speaks pointer
 * events (divider resize, pane focus), the 4px threshold that keeps a plain
 * click a click needs pointer arithmetic anyway, and the native API insists
 * on its own ghost image and its own cursor where the shell wants a tinted
 * third of a pane. Nothing here needs to leave the window, which is the one
 * thing the native API would buy.
 */
function useTabDrag(layout: Layout, onLayout: (next: Layout) => void): TabDragging {
  const [drag, setDrag] = useState<TabDrag | null>(null);
  // The window listeners live across renders; refs keep them reading the
  // layout of NOW rather than the one captured at pointerdown (pressing a
  // tab already refocuses the pane, which is itself a layout change).
  const layoutRef = useRef(layout);
  layoutRef.current = layout;
  const onLayoutRef = useRef(onLayout);
  onLayoutRef.current = onLayout;
  const suppressClick = useRef(false);

  const startTabDrag = useCallback((paneId: string, tabId: string, event: React.PointerEvent) => {
    if (event.button !== 0) return;
    // The × is for closing; a drag that starts there was aimed at it.
    if ((event.target as HTMLElement).closest(".shell-tab__close")) return;
    const grab = event.currentTarget as HTMLElement;
    const pointerId = event.pointerId;
    const startX = event.clientX;
    const startY = event.clientY;
    let active = false;

    const overAt = (x: number, y: number): TabDragOver | null => {
      // Hit-testing by point rather than per-pane listeners: the pane bodies
      // are full of editors that stopPropagation for their own reasons, and
      // a point asks the page instead of trusting the bubble.
      const hit = document.elementFromPoint(x, y);
      const paneEl = hit?.closest<HTMLElement>("[data-shell-pane-id]");
      const overPane = paneEl?.dataset.shellPaneId;
      if (!paneEl || !overPane) return null;
      // A collapsed strip is one target, not five: 36px has no thirds worth
      // aiming at, so anywhere on it means "join" (dropOnCollapsed expands).
      if (paneEl.classList.contains("shell-pane--collapsed")) {
        return { paneId: overPane, kind: "zone", zone: "center" };
      }
      // The side tree: same caret idea as the top strip, turned vertical.
      // The caret index is the hovered row's PANE index (the visual order
      // regroups by folder; the identity does not).
      const sideStrip = hit?.closest<HTMLElement>(".shell-sidetabs");
      if (sideStrip) {
        const stripRect = sideStrip.getBoundingClientRect();
        const rows = Array.from(sideStrip.querySelectorAll<HTMLElement>(".shell-tab"));
        let index = layoutRef.current
          ? (paneById(layoutRef.current, overPane)?.tabs.length ?? rows.length)
          : rows.length;
        let caretAt = 0;
        let found = false;
        for (const row of rows) {
          const rect = row.getBoundingClientRect();
          if (y < rect.top + rect.height / 2) {
            index = Number(row.dataset.shellTabIndex ?? rows.length);
            caretAt = rect.top - stripRect.top;
            found = true;
            break;
          }
        }
        if (!found && rows.length > 0) {
          caretAt = rows[rows.length - 1].getBoundingClientRect().bottom - stripRect.top;
        }
        return {
          paneId: overPane,
          kind: "tabs",
          index,
          caretAt: caretAt + sideStrip.scrollTop,
          vertical: true,
        };
      }
      if (hit && hit.closest(".shell-tabs")) {
        const strip = paneEl.querySelector<HTMLElement>(".shell-tabstrip");
        if (!strip) return null;
        const stripRect = strip.getBoundingClientRect();
        const tabs = Array.from(strip.querySelectorAll<HTMLElement>(".shell-tab"));
        let index = tabs.length;
        let caretX = 0;
        for (let i = 0; i < tabs.length; i += 1) {
          const rect = tabs[i].getBoundingClientRect();
          if (x < rect.left + rect.width / 2) {
            index = i;
            caretX = rect.left - stripRect.left;
            break;
          }
        }
        if (index === tabs.length && tabs.length > 0) {
          caretX = tabs[tabs.length - 1].getBoundingClientRect().right - stripRect.left;
        }
        return {
          paneId: overPane,
          kind: "tabs",
          index,
          caretAt: caretX + strip.scrollLeft,
          vertical: false,
        };
      }
      const body = paneEl.querySelector<HTMLElement>(".shell-pane__body") ?? paneEl;
      return { paneId: overPane, kind: "zone", zone: zoneAt(body.getBoundingClientRect(), x, y) };
    };

    const move = (e: PointerEvent) => {
      if (!active) {
        // Under the threshold it is still a click; a tab must not twitch
        // because a finger did.
        if (Math.hypot(e.clientX - startX, e.clientY - startY) < 4) return;
        active = true;
        // Captured only once the drag is real: capturing at pointerdown
        // would retarget the plain click a non-drag still wants to be.
        try {
          grab.setPointerCapture(pointerId);
        } catch {
          // A tab that re-rendered away mid-press has nothing to capture.
        }
      }
      setDrag({ fromPane: paneId, tabId, over: overAt(e.clientX, e.clientY) });
    };
    const finish = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      window.removeEventListener("keydown", key, true);
      try {
        grab.releasePointerCapture(pointerId);
      } catch {
        // Never captured, or already released by the pointerup itself.
      }
      setDrag(null);
    };
    const up = (e: PointerEvent) => {
      if (active) {
        // The click that follows this pointerup is the drag's shadow.
        suppressClick.current = true;
        window.setTimeout(() => {
          suppressClick.current = false;
        }, 0);
        const over = overAt(e.clientX, e.clientY);
        const current = layoutRef.current;
        if (over) {
          onLayoutRef.current(
            over.kind === "zone"
              ? paneById(current, over.paneId)?.collapsed
                ? // A drop on a strip expands it and joins its center —
                  // edge zones were already guarded off in overAt.
                  dropOnCollapsed(current, paneId, tabId, over.paneId)
                : moveTab(current, paneId, tabId, over.paneId, over.zone)
              : moveTabToIndex(current, paneId, tabId, over.paneId, over.index),
          );
        }
      }
      finish();
    };
    const key = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      // Capture phase, and stopped: while a drag is in flight, Escape means
      // "put it back", never whatever a panel underneath would do with it.
      e.stopPropagation();
      finish();
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    window.addEventListener("keydown", key, true);
  }, []);

  return { drag, startTabDrag, suppressClick };
}


/**
 * The keys an editor is expected to have.
 *
 * Deliberately few, and all of them about the arrangement rather than the
 * text: anything that edits belongs to the editor in the pane, which already
 * has its own keymap and its own idea of what has focus.
 */
function useShellKeys(layout: Layout, onLayout: (next: Layout) => void, onSearch?: () => void) {
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const chord = event.metaKey || event.ctrlKey;
      if (!chord) return;
      const pane = layout.focus;
      if (event.shiftKey && (event.key === "f" || event.key === "F")) {
        // Mod-Shift-F asks about the whole folder, so it works from anywhere
        // — including with an editor focused, which is why the editors leave
        // the shifted chord alone and keep plain Mod-F for themselves.
        if (!onSearch) return;
        event.preventDefault();
        onSearch();
        return;
      }
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
  }, [layout, onLayout, onSearch]);
}
