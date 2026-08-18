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
// Unlike the number rail this one TAKES the pointer: it is the only way to
// reach a cell's Run button now, so it is real UI, keyboard-reachable, and
// each icon is a button with a name.

import { useEffect, useMemo, useRef } from "react";
import type { EditorView } from "@codemirror/view";

import { ICON_SIZE, iconVisible, stackIcons } from "../lib/cardRail";
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
  onOpen: (card: DocCard, iconTop: number) => void;
}

/** The glyph for a card. Deliberately text, not an icon font: the app ships
 * no icon set, and these read at any size and in any theme. */
function glyph(card: DocCard, state: CardState): string {
  if (card.kind === "diagram") return "◈";
  if (card.kind === "fence") return "≡";
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

function title(card: DocCard, state: CardState, rendered: boolean): string {
  if (card.kind === "fence") return `${card.label} (click to convert)`;
  // The rail is the way back from a rendered block, so the icon has to say
  // which way it goes — "click to view" on something already on screen is a
  // dead end a person only finds by trying it.
  const swap = rendered ? "showing its result — click for the source" : "click to render it";
  if (card.kind === "diagram") return `${card.label} — ${swap}`;
  const suffix =
    state === "running"
      ? "running"
      : state === "ok"
        ? "last run passed"
        : state === "failed"
          ? "last run failed"
          : state === "unknown"
            ? "not known to the server yet"
            : "never run";
  return `${card.label} — ${suffix}; ${swap}`;
}

export function CardRail({
  view,
  cards,
  stateOf,
  openKey,
  renderedAt,
  onOpen,
}: CardRailProps) {
  const railRef = useRef<HTMLDivElement | null>(null);
  // Icon tops, shared between the placement effect and the click handler so
  // the popover opens exactly level with the icon that opened it.
  const topsRef = useRef<Map<string, number>>(new Map());
  // The card list changes identity on every keystroke (it is derived from
  // the parse); its CONTENT is what placement depends on.
  const signature = useMemo(() => cards.map((c) => `${c.key}:${c.at}`).join("|"), [cards]);

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
      const wanted = cards.map((card) => {
        const pos = Math.min(card.at, view.state.doc.length);
        return view.lineBlockAt(pos).top + offset;
      });
      const tops = stackIcons(wanted);
      const band = { top: scroller.scrollTop, bottom: scroller.scrollTop + scroller.clientHeight };
      const map = new Map<string, number>();
      cards.forEach((card, i) => {
        const el = rail.querySelector<HTMLElement>(`[data-card="${CSS.escape(card.key)}"]`);
        map.set(card.key, tops[i]);
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
    // `signature` stands in for the card list, which is rebuilt per parse.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [view, signature]);

  return (
    <div className="cm-card-rail" role="toolbar" aria-label="Cells and diagrams" ref={railRef}>
      {cards.map((card) => {
        const state = stateOf(card);
        const rendered = renderedAt.includes(card.at);
        return (
          <button
            key={card.key}
            type="button"
            data-card={card.key}
            className={`cm-card-rail__icon cm-card-rail__${card.kind} is-${state}${
              openKey === card.key || rendered ? " on" : ""
            }`}
            style={{ height: ICON_SIZE, width: ICON_SIZE }}
            aria-label={title(card, state, rendered)}
            aria-pressed={openKey === card.key || rendered}
            data-tip={title(card, state, rendered)}
            onClick={() => onOpen(card, topsRef.current.get(card.key) ?? 0)}
          >
            {glyph(card, state)}
          </button>
        );
      })}
    </div>
  );
}
