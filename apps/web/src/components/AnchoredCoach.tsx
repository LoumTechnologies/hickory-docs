import { useEffect, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import "./anchoredCoach.css";

/** Nonmodal: the actual target remains operable; never traps or steals focus. */
export function AnchoredCoach({ target, title, children, onDismiss }: { target: string; title: string; children: ReactNode; onDismiss(): void }) {
  const popup = useRef<HTMLElement>(null);
  const [position, setPosition] = useState<{ top: number; left: number } | null>(null);
  const [missing, setMissing] = useState(false);
  function locate() {
    const element = document.querySelector<HTMLElement>(target);
    if (!element) { setMissing(true); return; }
    element.scrollIntoView({ block: "nearest", behavior: "instant" });
    if (!element.hasAttribute("tabindex") && !element.matches("button,input,select,textarea,a")) element.setAttribute("tabindex", "-1");
    element.focus({ preventScroll: true });
  }
  useEffect(() => {
    function place() {
      const element = document.querySelector<HTMLElement>(target);
      if (!element || element.getClientRects().length === 0) { setMissing(true); setPosition(null); return; }
      const box = element.getBoundingClientRect();
      const width = Math.min(400, window.innerWidth - 24);
      const height = popup.current?.offsetHeight ?? 220;
      // Place outside the target rectangle. Large targets fall back to a
      // docked popup with an explicit locate/dismiss path.
      const left = box.right + width + 12 < window.innerWidth ? box.right + 8
        : box.left > width + 12 ? box.left - width - 8 : Math.max(12, Math.min(window.innerWidth - width - 12, box.left));
      const top = box.right + width + 12 < window.innerWidth || box.left > width + 12 ? Math.min(box.top, window.innerHeight - height - 12)
        : box.bottom + height + 12 < window.innerHeight ? box.bottom + 8 : Math.max(12, box.top - height - 8);
      setMissing(false); setPosition({ top: Math.max(12, top), left });
    }
    const key = (event: KeyboardEvent) => { if (event.key === "Escape" && popup.current?.contains(document.activeElement)) { event.preventDefault(); onDismiss(); } };
    place(); const observer = new MutationObserver(place);
    observer.observe(document.body, { childList: true, subtree: true });
    window.addEventListener("resize", place); window.addEventListener("scroll", place, true); window.addEventListener("keydown", key);
    return () => { observer.disconnect(); window.removeEventListener("resize", place); window.removeEventListener("scroll", place, true); window.removeEventListener("keydown", key); };
  }, [target, onDismiss]);
  return createPortal(<aside ref={popup} className="anchored-coach" role="region" aria-label={title} style={position ?? { bottom: 12, right: 12 }}>
    <header><strong>{title}</strong><button onClick={onDismiss} aria-label="Dismiss coach">×</button></header>
    {missing && <p role="status">The target is closed, hidden or unavailable. Open its normal workspace pane, use Locate target, or skip this step.</p>}
    <button onClick={locate}>Locate target</button>
    {children}
  </aside>, document.body);
}
