<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/TooltipLayer.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [Every Hover Hint Is Drawn In The App Theme](../../../../../../guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme/contract.md)

Captured SHA-256: `7a2da625148fff6c45d701a7646a14cb3e28a9e3295bbf25e43d33d864061c56`.

<book:entry path="apps/web/src/components/TooltipLayer.tsx">
import { useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { placeTip, tipTargetOf, TIP_ATTR, type Placement } from "../lib/tooltip";

/** How long the pointer has to rest before a tooltip appears. */
const DELAY = 450;

interface Shown {
  text: string;
  anchor: DOMRect;
}

/**
 * The app's one tooltip.
 *
 * Mounted once per entry point; every other component opts in by putting
 * `data-tip="…"` on an element, exactly where `title="…"` used to go. One
 * delegated listener means a tooltip costs a component nothing — no state, no
 * portal, no hover handlers — and every tooltip in the app is the same card
 * in the same palette, which is the whole point: `title=` is drawn by the OS
 * and never matches the theme.
 *
 * An icon-only control still needs `aria-label`: this is a picture, not an
 * accessible name.
 */
export function TooltipLayer() {
  const [shown, setShown] = useState<Shown | null>(null);
  const [placement, setPlacement] = useState<Placement | null>(null);
  const card = useRef<HTMLDivElement | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const parent = `tooltips:${useId()}`;
  const shownRevision = useUiRevision(() => shown);
  type Target = { el: HTMLElement; text: string; label: string; token: string };
  const [targets, setTargets] = useState<Target[]>([]);
  const cancel = () => { if (timer.current !== null) clearTimeout(timer.current); timer.current = null; };
  const hide = () => { cancel(); setShown(null); setPlacement(null); };
  const show = (found: { el: HTMLElement; text: string }) => {
    if (!found.el.isConnected || tipTargetOf(found.el)?.text !== found.text) return false;
    cancel(); setPlacement(null);
    setShown({ text: found.text, anchor: found.el.getBoundingClientRect() });
    return true;
  };
  const visible = (el: HTMLElement) => {
    if (!el.isConnected || el.closest('[hidden], [inert], [aria-hidden="true"]')) return false;
    const style = getComputedStyle(el), rect = el.getBoundingClientRect();
    return style.visibility !== "hidden" && style.display !== "none" && rect.width > 0 && rect.height > 0 && rect.bottom > 0 && rect.right > 0 && rect.top < window.innerHeight && rect.left < window.innerWidth;
  };
  const current = (target: Target) => visible(target.el) && tipTargetOf(target.el)?.text === target.text;
  useUiReading("shown tooltip", { text: shown?.text ?? null, scope: "Currently displayed tooltip snapshot; not a source-edit baseline" });
  useUiCommands([
    { key: `${parent}:targets`, command: {
      label: "Disclose visible tooltip targets", parent, subject: `${parent}:index`,
      parameters: { offset: { type: "number", integer: true, minimum: 0, maximum: 1000000 }, limit: { type: "number", integer: true, minimum: 1, maximum: 64 } }, prerequisites: [],
      read: () => ({ revision: "explicit-current-tooltip-index", enabled: true }),
      execute: async args => {
        // data-tip is this controller's explicit declarative input. No caller
        // selector, arbitrary element text, hidden subtree or synthetic event.
        const all = [...document.querySelectorAll<HTMLElement>(`[${TIP_ATTR}]`)].filter(el => visible(el) && !!tipTargetOf(el));
        const offset = Number(args.offset), limit = Number(args.limit);
        if (offset > all.length) return { status: "unavailable", reason: "Tooltip page outside the current visible list" };
        const page = all.slice(offset, offset + limit).map(el => ({ el, text: tipTargetOf(el)!.text, label: (el.getAttribute("aria-label") || el.tagName.toLowerCase()).slice(0, 120), token: crypto.randomUUID() }));
        setTargets(page);
        return { status: "completed", value: { disclosed: page.length, total: all.length, next_offset: offset + page.length < all.length ? offset + page.length : null, text_delivery: "Explicit Show then Read shown tooltip bytes; opening is not control activation" } };
      },
    } },
    { key: `${parent}:hide`, command: { label: "Dismiss shown tooltip", parent, subject: parent, parameters: {}, prerequisites: [], read: () => ({ revision: shownRevision(), enabled: !!shown }), execute: async () => { hide(); return { status: "completed", value: { shown: false } }; } } },
    ...targets.map(target => ({ key: `${parent}:${target.token}`, command: {
      label: `Show tooltip: ${target.label}`, parent, subject: `${parent}:${target.token}`, parameters: {}, prerequisites: ["Explicitly disclosed visible target; target/text replacement requires redisclosure"],
      read: () => ({ revision: target.token, enabled: current(target), reason: current(target) ? undefined : "Tooltip target changed or left the viewport; redisclose targets" }),
      execute: async () => {
        if (!current(target) || !show(target)) return { status: "unavailable" as const, reason: "Tooltip target changed or left the viewport; redisclose targets" };
        return { status: "completed" as const, value: { shown: true, text_bytes: new TextEncoder().encode(target.text).length, activated_control: false } };
      },
    } })),
  ]);
  useEffect(() => {
    const arm = (event: Event) => {
      const found = tipTargetOf(event.target);
      cancel();
      if (!found) {
        setShown(null);
        setPlacement(null);
        return;
      }
      timer.current = setTimeout(() => {
        // Re-read the rect at show time: the row may have scrolled or the
        // element may be gone by the time the delay expires.
        show(found);
      }, DELAY);
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") hide();
    };

    document.addEventListener("pointerover", arm, true);
    document.addEventListener("focusin", arm, true);
    document.addEventListener("pointerdown", hide, true);
    document.addEventListener("focusout", hide, true);
    document.addEventListener("keydown", onKey, true);
    // Capture, so a scroll inside any pane counts: a tooltip pinned to where
    // an element used to be is worse than no tooltip.
    window.addEventListener("scroll", hide, true);
    window.addEventListener("resize", hide);
    window.addEventListener("blur", hide);
    return () => {
      cancel();
      document.removeEventListener("pointerover", arm, true);
      document.removeEventListener("focusin", arm, true);
      document.removeEventListener("pointerdown", hide, true);
      document.removeEventListener("focusout", hide, true);
      document.removeEventListener("keydown", onKey, true);
      window.removeEventListener("scroll", hide, true);
      window.removeEventListener("resize", hide);
      window.removeEventListener("blur", hide);
    };
  }, []);

  // Measured, then placed: the card is rendered invisibly for one frame so
  // its real size decides whether it fits above the anchor.
  useLayoutEffect(() => {
    const el = card.current;
    if (!shown || !el || placement) return;
    const box = el.getBoundingClientRect();
    setPlacement(
      placeTip(shown.anchor, box, {
        width: window.innerWidth,
        height: window.innerHeight,
      }),
    );
  }, [shown, placement]);

  if (!shown || typeof document === "undefined") return null;
  return createPortal(
    <div
      ref={card}
      className="tip"
      // Purely visual: screen readers read the control's own label, and a
      // second copy of the same words is noise.
      aria-hidden="true"
      data-side={placement?.side}
      style={
        placement
          ? { left: `${placement.left}px`, top: `${placement.top}px` }
          : { left: 0, top: 0, visibility: "hidden" }
      }
    >
      {shown.text}
    </div>,
    document.body,
  );
}
</book:entry>

</book:doc>
