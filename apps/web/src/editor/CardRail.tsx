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
// wanted top (its card's line) and `stackIcons` pushes the rest down, which
// is the same rule that keeps two crowded cards apart — so a card's actions
// read as a group without any second layout pass.

import { useEffect, useMemo, useRef } from "react";
import type { EditorView } from "@codemirror/view";

import { ICON_SIZE, iconVisible, stackIcons } from "../lib/cardRail";
import type { RailAction } from "../lib/railActions";
import type { DocCard } from "./cards";

/** What an exec card's icon says about its cell without being opened. */
export type CardState = "idle" | "running" | "ok" | "failed" | "unknown";

export interface CardRailProps {
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
 * Run is the only glyph that varies, because it is the only one carrying
 * information: it is both the button that runs the cell and the report of
 * what happened last time it ran. */
function glyph(card: DocCard, action: RailAction, state: CardState): string {
  switch (action) {
    case "convert":
      return "≡";
    case "replay":
      return "↺";
    case "source":
      // A diagram and an equation each have only this one icon, so each keeps
      // its own mark and the rail says what KIND of thing sits on that line;
      // a cell's column needs Run and source to be told apart at a glance.
      if (card.kind === "diagram") return "◈";
      if (card.kind === "math") return "∑";
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
      return "last run passed";
    case "failed":
      return "last run failed";
    case "unknown":
      return "not known to the server yet";
    default:
      return "never run";
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
      return state === "running"
        ? `${card.label} — running; click does nothing until it finishes`
        : `${card.label} — ${runSuffix(state)}; click to run it`;
  }
}

export function CardRail({
  view,
  cards,
  stateOf,
  openKey,
  renderedAt,
  actionsOf,
  replayingAt,
  onAction,
}: CardRailProps) {
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
    [cards],
  );
  const railRef = useRef<HTMLDivElement | null>(null);
  // Icon tops, shared between the placement effect and the click handler so
  // the popover opens exactly level with the icon that opened it.
  const topsRef = useRef<Map<string, number>>(new Map());
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
      const tops = stackIcons(wanted);
      // The band is the rail's OWN box, because `tops` are rail-relative
      // viewport pixels — the same space `el.style.top` is written in. Asking
      // the scroller for `scrollTop` here would be a document coordinate, and
      // the two agree only while the document is scrolled to the very top;
      // below that every icon reads as off-screen and the rail goes blank.
      const band = { top: 0, bottom: scroller.clientHeight };
      const map = new Map<string, number>();
      icons.forEach((icon, i) => {
        const el = rail.querySelector<HTMLElement>(`[data-card="${CSS.escape(icon.key)}"]`);
        map.set(icon.key, tops[i]);
        if (!el) return;
        el.style.top = `${tops[i]}px`;
        el.style.display = iconVisible(tops[i], band) ? "" : "none";
      });
      topsRef.current = map;
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

  return (
    <div className="cm-card-rail" role="toolbar" aria-label="Cell and diagram actions" ref={railRef}>
      {icons.map(({ card, action, key }) => {
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
            data-card={key}
            className={`cm-card-rail__icon cm-card-rail__${card.kind} cm-card-rail__act-${action} is-${state}${
              pressed ? " on" : ""
            }`}
            style={{ height: ICON_SIZE, width: ICON_SIZE }}
            disabled={action === "run" && state === "running"}
            aria-label={title(card, action, state, rendered, replaying)}
            aria-pressed={pressed}
            data-tip={title(card, action, state, rendered, replaying)}
            onClick={() => onAction(card, action, topsRef.current.get(key) ?? 0)}
          >
            {glyph(card, action, state)}
          </button>
        );
      })}
    </div>
  );
}
