// The action rail: one icon per card, down the outer edge of the editor.
//
// Positioned the way the right-hand number rail is — the editor's own height
// map is the truth, read inside `requestMeasure` so the geometry is
// post-layout, and redrawn on scroll, resize, and document change. The
// buttons themselves are React (they carry handlers, state colours, and
// accessible names); only their `top` is written imperatively, because that
// changes on every scrolled frame and a React render per frame would buy
// nothing.
//
// Unlike the number rail this one TAKES the pointer: every control a card
// has is here now — Run, source, Replay — so it is real UI, keyboard-
// reachable, and each icon is a button with a name.
//
// A card contributes a COLUMN of icons, not one icon. Each gets the same
// wanted top (its card's line). Each group can use the space before the next
// populated line; excess actions open in a floating grid instead of moving
// that next line's icons.

import { createPortal } from "react-dom";
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { useUiCommands } from "../lib/uiActionBridge";
import type { EditorView } from "@codemirror/view";

import { ICON_SIZE, iconVisible, groupRailIcons } from "../lib/cardRail";
import { attrValue } from "../lib/attrSelector";
import type { RailAction } from "../lib/railActions";
import type { DocCard } from "./cards";

/** What an exec card's icon says about its cell without being opened. */
export type CardState = "idle" | "running" | "ok" | "failed" | "unknown" | "stale";

export interface CardRailProps {
  documentId?: string;
  view: EditorView | null;
  cards: DocCard[];
  /** The live state of an exec card; other kinds are always "idle". */
  stateOf: (card: DocCard) => CardState;
  /** The card whose popover is open, so its icon can show as pressed. */
  openKey: string | null;
  /** Block starts currently showing their result rather than their source. */
  renderedAt: readonly number[];
  /** The icons this card offers, top to bottom. */
  actionsOf: (card: DocCard) => RailAction[];
  /** Cells whose transcript is currently revealed, by block start. */
  replayingAt: readonly number[];
  onAction: (card: DocCard, action: RailAction, iconTop: number) => void;
}

/** The glyph for one action. Deliberately text, not an icon font: the app
 * ships no icon set, and these read at any size and in any theme.
 *
 * Two glyphs carry information rather than depicting a thing, and both do it
 * the same way — by saying what the CLICK gets you:
 *
 * - Run is the button that runs the cell and the report of what happened last
 *   time it ran.
 * - Source is a toggle, so it shows its DESTINATION. While the picture is on
 *   screen it is `<>` — click for the code. While the code is on screen it is
 *   the kind's own mark — click for the diagram, the chart, the grid, the
 *   equation.
 *
 * The old design showed the kind's mark in both states, which meant four
 * near-identical squares (`◈ ▦ ▨ ▣`) that depicted what the block IS and left
 * the verb to be guessed. In the state a reader is usually in, every block now
 * shows the same familiar `<>`; the kind only appears when it is the thing
 * being gone back to. */
function glyph(
  card: DocCard,
  action: RailAction,
  state: CardState,
  rendered: boolean,
): string {
  switch (action) {
    case "convert":
      return "≡";
    case "replay":
      return "↺";
    case "source":
      // Showing the result: the click gets you the source.
      if (rendered) return "<>";
      // Showing the source: the click gets you the thing itself.
      if (card.kind === "diagram") return "◈";
      if (card.kind === "math") return "∑";
      if (card.kind === "table") return "▦";
      if (card.kind === "picture") return "▨";
      return "▣";
    case "run":
      switch (state) {
        case "running":
          return "◐";
        case "ok":
          return "✓";
        case "failed":
          return "✕";
        default:
          return "▶";
      }
  }
}

function runSuffix(state: CardState): string {
  switch (state) {
    case "running":
      return "running now";
    case "ok":
      return "recorded · last run succeeded";
    case "failed":
      return "recorded · last run or check failed";
    case "stale":
      return "stale recording — inputs changed";
    case "unknown":
      return "save the document first";
    default:
      return "unrecorded · never run";
  }
}

/**
 * The icon's accessible name and tooltip.
 *
 * Every one of these says what the CLICK does, not what the icon depicts. An
 * icon whose name is a noun leaves a person to guess the verb, and the rail
 * is now the only place any of these verbs exist.
 */
function title(
  card: DocCard,
  action: RailAction,
  state: CardState,
  rendered: boolean,
  replaying: boolean,
): string {
  switch (action) {
    case "convert":
      return `${card.label} — click to make it a cell`;
    case "replay":
      return replaying
        ? `${card.label} — hiding the transcript; click to show what ran`
        : `${card.label} — click to replay what ran`;
    case "source":
      return rendered
        ? `${card.label} — showing its result; click for the source`
        : `${card.label} — showing its source; click to render it`;
    case "run":
      if (state === "unknown") return `${card.label} — save this document to enable running`;
      return state === "running"
        ? `${card.label} — running; click does nothing until it finishes`
        : `${card.label} — ${runSuffix(state)}; click to run it`;
  }
}

export function CardRail({
  documentId = "editor",
  view,
  cards,
  stateOf,
  openKey,
  renderedAt,
  actionsOf,
  replayingAt,
  onAction,
}: CardRailProps) {
  const instance = useId();
  const parent = `document:${documentId}/rail:${instance}`;
  const [actionError, setActionError] = useState("");
  const [overflowGroups, setOverflowGroups] = useState<{ id: string; top: number; keys: string[] }[]>([]);
  const [expanded, setExpanded] = useState<{ id: string; left: number; top: number } | null>(null);
  const overflowPanel = useRef<HTMLDivElement>(null);
  const overflowTrigger = useRef<HTMLButtonElement | null>(null);
  useEffect(() => {
    if (!expanded) return;
    overflowPanel.current?.querySelector<HTMLButtonElement>("button")?.focus();
    const outside = (event: PointerEvent) => { if (event.target instanceof Node && !overflowPanel.current?.contains(event.target) && !overflowTrigger.current?.contains(event.target)) setExpanded(null); };
    const keyboard = (event: KeyboardEvent) => { if (event.key === "Escape") { event.preventDefault(); setExpanded(null); overflowTrigger.current?.focus(); } };
    const close = () => setExpanded(null);
    document.addEventListener("pointerdown", outside, true); document.addEventListener("keydown", keyboard, true);
    view?.scrollDOM.addEventListener("scroll", close); window.addEventListener("resize", close);
    return () => { document.removeEventListener("pointerdown", outside, true); document.removeEventListener("keydown", keyboard, true); view?.scrollDOM.removeEventListener("scroll", close); window.removeEventListener("resize", close); };
  }, [expanded, view]);
  // One entry per ICON. Every action of a card wants the same top — its
  // card's line — and the stacker turns that into a column.
  const icons = useMemo(
    () =>
      cards.flatMap((card) =>
        actionsOf(card).map((action) => ({ card, action, key: `${card.key}:${action}` })),
      ),
    // `actionsOf` is rebuilt every render by the parent; the CARDS and what
    // they offer are what the list depends on.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [cards, actionsOf],
  );
  const railRef = useRef<HTMLDivElement | null>(null);
  // Icon tops, shared between the placement effect and the click handler so
  // the popover opens exactly level with the icon that opened it.
  const topsRef = useRef<Map<string, number>>(new Map());
  const renderedDocument = view?.state.doc;
  const invoke = useUiCommands(icons.map(({ card, action, key }) => ({
    key: `${parent}/${key}`,
    command: {
      label: `${action === "run" ? "Run" : action === "source" ? "Toggle source" : action === "replay" ? "Toggle transcript" : "Convert to cell"}: ${card.label}`,
      subject: `${parent}/${card.key}`, parent, parameters: {},
      prerequisites: action === "run" ? ["Saved cell identity; otherwise Save requires user input"] : [],
      read: () => {
        const state = stateOf(card);
        const settled = view?.state.doc === renderedDocument;
        return {
          revision: `${documentRevision(view?.state.doc)}:${state}:${renderedAt.includes(card.at)}:${replayingAt.includes(card.at)}:${openKey}`,
          enabled: settled && !(action === "run" && state === "running"),
          reason: !settled ? "Editor parse is updating; observe again" : action === "run" && state === "running" ? "Cell is already running" : undefined,
          value: { state, source_visible: !renderedAt.includes(card.at), transcript_visible: replayingAt.includes(card.at) },
        };
      },
      execute: async () => {
        onAction(card, action, topsRef.current.get(key) ?? 0);
        if (action === "run" && stateOf(card) === "unknown") return { status: "requires_input" as const, reason: "Save this document using the opened Save control, then observe again" };
        // onAction dispatches the existing UI command. Runtime completion and
        // verification are established only by the actual run receipt.
        return { status: "completed" as const, value: { ui_handler_returned: true, runtime_verification: "not established", action } };
      },
    },
  })));
  // The card list changes identity on every keystroke (it is derived from
  // the parse); its CONTENT is what placement depends on.
  const signature = useMemo(
    () => icons.map((i) => `${i.key}:${i.card.at}`).join("|"),
    [icons],
  );

  useEffect(() => {
    const rail = railRef.current;
    if (!view || !rail) return;
    let frame: number | null = null;
    // Set the instant this effect is torn down. Without it a measure queued
    // by the interval or a scroll can land after the editor was destroyed,
    // where CodeMirror walks a detached DOM and throws.
    let disposed = false;

    const place = () => {
      // A view whose DOM has left the page is a view mid-teardown: asking it
      // to measure walks a detached tree, which throws rather than answering.
      if (disposed || !view.dom.isConnected) return;
      const scroller = view.scrollDOM;
      const railTop = rail.getBoundingClientRect().top;
      // Document coordinates become rail-relative through the document's own
      // screen position — the same conversion the number rail and the ribbon
      // overlay use, so none of the three can disagree about where a line is.
      const offset = view.documentTop - railTop;
      const wanted = icons.map(({ card }) => {
        const pos = Math.min(card.at, view.state.doc.length);
        return view.lineBlockAt(pos).top + offset;
      });
      const { tops, overflow } = groupRailIcons(wanted, scroller.clientHeight || Number.POSITIVE_INFINITY);
      // The band is the rail's OWN box, because `tops` are rail-relative
      // viewport pixels — the same space `el.style.top` is written in. Asking
      // the scroller for `scrollTop` here would be a document coordinate, and
      // the two agree only while the document is scrolled to the very top;
      // below that every icon reads as off-screen and the rail goes blank.
      const band = { top: 0, bottom: scroller.clientHeight };
      const map = new Map<string, number>();
      icons.forEach((icon, i) => {
        const el = rail.querySelector<HTMLElement>(
          `[data-card=${attrValue(icon.key)}]`,
        );
        const top = tops[i] ?? overflow.find(group => group.indices.includes(i))?.top ?? wanted[i];
        map.set(icon.key, top);
        if (!el) return;
        el.style.top = `${top}px`;
        el.style.display = tops[i] !== null && iconVisible(top, band) ? "" : "none";
      });
      topsRef.current = map;
      const groups = overflow.filter(group => iconVisible(group.top, band)).map(group => ({ id: icons[group.indices[0]].key, top: group.top, keys: group.indices.map(index => icons[index].key) }));
      setOverflowGroups(previous => JSON.stringify(previous) === JSON.stringify(groups) ? previous : groups);
    };

    const schedule = () => {
      if (disposed || frame !== null) return;
      frame = requestAnimationFrame(() => {
        frame = null;
        if (disposed || !view.dom.isConnected) return;
        view.requestMeasure({ read: place });
      });
    };
    schedule();

    view.scrollDOM.addEventListener("scroll", schedule, { passive: true });
    window.addEventListener("resize", schedule);
    const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(schedule);
    observer?.observe(view.scrollDOM);
    observer?.observe(view.contentDOM);
    // The same last-resort tick the number rail keeps, for geometry that
    // settled without firing anything (a font loading, an image decoding).
    const timer = window.setInterval(schedule, 500);

    return () => {
      disposed = true;
      view.scrollDOM.removeEventListener("scroll", schedule);
      window.removeEventListener("resize", schedule);
      observer?.disconnect();
      window.clearInterval(timer);
      if (frame !== null) cancelAnimationFrame(frame);
    };
    // `signature` stands in for the icon list, which is rebuilt per parse.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [view, signature]);

  useEffect(() => { setExpanded(null); }, [view, signature]);

  const renderIcon = (icon: typeof icons[number], popup = false) => {
    const { card, action, key } = icon;
    const state = stateOf(card);
    const rendered = renderedAt.includes(card.at);
    const replaying = replayingAt.includes(card.at);
    // Only an icon that TOGGLES something reports a pressed state. Run
    // is a verb — it does not stay down — so marking it pressed because
    // its cell is rendered would be a lie a screen reader repeats.
    const pressed =
      action === "source"
        ? rendered
        : action === "replay"
          ? replaying
          : action === "convert"
            ? openKey === key
            : undefined;
    return (
      <button
        key={key}
        type="button"
        data-card={popup ? undefined : key}
        className={`cm-card-rail__icon cm-card-rail__${card.kind} cm-card-rail__act-${action} is-${state}${
          pressed ? " on" : ""
        }`}
        style={{ height: ICON_SIZE, width: ICON_SIZE, ...(popup ? { position: "relative", left: "auto", transform: "none", top: "auto" } as const : {}) }}
        disabled={action === "run" && state === "running"}
        aria-label={title(card, action, state, rendered, replaying)}
        aria-pressed={pressed}
        data-tip={title(card, action, state, rendered, replaying)}
        onClick={() => { setExpanded(null); setActionError(""); void invoke(`${parent}/${key}`).catch(error => setActionError(String(error))); }}
      >
        {glyph(card, action, state, rendered)}
      </button>
    );
  };

  return (
    <div className="cm-card-rail" data-empty={icons.length === 0} role="toolbar" aria-label="Cell and diagram actions" ref={railRef}>
      {actionError && createPortal(<div className="cm-card-action-error" role="alert">
        <strong>Action failed</strong>
        <p>{actionError}</p>
        <button type="button" onClick={() => setActionError("")}>Dismiss</button>
      </div>, document.body)}
      {icons.map(icon => renderIcon(icon))}
      {overflowGroups.map(group => <button key={group.id} type="button" data-overflow-group={group.id}
        className="cm-card-rail__icon cm-card-rail__more" style={{ top: group.top, width: ICON_SIZE, height: ICON_SIZE }}
        aria-label={`Show ${group.keys.length} more gutter icons`} aria-haspopup="dialog" aria-expanded={expanded?.id === group.id}
        onClick={event => {
          if (expanded?.id === group.id) { setExpanded(null); return; }
          const box = event.currentTarget.getBoundingClientRect(); overflowTrigger.current = event.currentTarget;
          const height = Math.min(window.innerHeight - 16, Math.ceil(group.keys.length / 4) * 22 + 14);
          setExpanded({ id: group.id, left: Math.max(8, Math.min(box.left - 108, window.innerWidth - 116)), top: Math.max(8, Math.min(box.top, window.innerHeight - height - 8)) });
        }}>⋯</button>)}
      {expanded && createPortal(<div ref={overflowPanel} role="dialog" aria-label="Gutter icons"
        style={{ position: "fixed", left: expanded.left, top: expanded.top, display: "grid", gridTemplateColumns: "repeat(4, 20px)", gap: 2, padding: 6, maxHeight: "calc(100vh - 16px)", overflowY: "auto", zIndex: 10000, background: "var(--bg-raised)", color: "var(--fg)", border: "1px solid var(--border)", borderRadius: 6, boxShadow: "0 4px 18px #0005" }}>
        {(overflowGroups.find(group => group.id === expanded.id)?.keys ?? []).flatMap(key => { const icon = icons.find(candidate => candidate.key === key); return icon ? [renderIcon(icon, true)] : []; })}
      </div>, document.body)}
    </div>
  );
}

const documentRevisions = new WeakMap<object, string>();
function documentRevision(document: object | undefined): string {
  if (!document) return "unavailable";
  let revision = documentRevisions.get(document);
  if (!revision) { revision = crypto.randomUUID(); documentRevisions.set(document, revision); }
  return revision;
}
