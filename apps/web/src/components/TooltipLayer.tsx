import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { placeTip, tipTargetOf, type Placement } from "../lib/tooltip";

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

  useEffect(() => {
    const cancel = () => {
      if (timer.current !== null) clearTimeout(timer.current);
      timer.current = null;
    };
    const hide = () => {
      cancel();
      setShown(null);
      setPlacement(null);
    };
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
        if (!found.el.isConnected) return;
        setPlacement(null);
        setShown({ text: found.text, anchor: found.el.getBoundingClientRect() });
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
