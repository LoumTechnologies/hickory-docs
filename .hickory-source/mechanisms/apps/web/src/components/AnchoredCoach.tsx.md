<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/AnchoredCoach.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-01e0c9a8e706"></a>
## `apps/web/src/components/AnchoredCoach.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-01e0c9a8e706-AnchoredCoach"></a>
### `AnchoredCoach`

<storycode:copy id="f-01e0c9a8e706-AnchoredCoach">
import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiOutcome } from "../lib/uiActions";
import { createPortal } from "react-dom";
import "./anchoredCoach.css";

/** Nonmodal: the actual target remains operable; never traps or steals focus. */
export function AnchoredCoach({ target, title, children, onDismiss }: { target: string; title: string; children: ReactNode; onDismiss(): void }) {
  const popup = useRef<HTMLElement>(null);
  const [position, setPosition] = useState<{ top: number; left: number } | null>(null);
  const [missing, setMissing] = useState(false);
  const subject = `coach-popup:${useId()}`;
  const revision = useUiRevision(() => [target, title, missing]);
  function locate(): UiOutcome {
    const element = document.querySelector<HTMLElement>(target);
    if (!element || element.getClientRects().length === 0) { setMissing(true); return { status: "unavailable", reason: "Target closed, hidden or unavailable; open its normal workspace pane first" }; }
    element.scrollIntoView({ block: "nearest", behavior: "instant" });
    if (!element.hasAttribute("tabindex") && !element.matches("button,input,select,textarea,a")) element.setAttribute("tabindex", "-1");
    element.focus({ preventScroll: true });
    return { status: "completed", value: { scroll_requested: true, focused: document.activeElement === element } };
  }
  const invoke = useUiCommands([
    { key: `${subject}:locate`, command: { label: "Locate coach target", parent: subject, subject, parameters: {}, prerequisites: ["The actual workspace target must be mounted and visible"], read: () => ({ revision: revision(), enabled: true, value: { missing } }), execute: async () => locate() } },
    { key: `${subject}:dismiss`, command: { label: "Dismiss anchored coach", parent: subject, subject, parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async () => { onDismiss(); return { status: "completed", value: { dismiss_requested: true } }; } } },
  ]);

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
      setMissing(false); setPosition({ top: Math.max(12, Math.min(window.innerHeight - height - 12, top)), left });
    }
    const key = (event: KeyboardEvent) => { if (event.key === "Escape" && popup.current?.contains(document.activeElement)) { event.preventDefault(); void invoke(`${subject}:dismiss`); } };
    place(); const observer = new MutationObserver(place);
    observer.observe(document.body, { childList: true, subtree: true });
    window.addEventListener("resize", place); window.addEventListener("scroll", place, true); window.addEventListener("keydown", key);
    return () => { observer.disconnect(); window.removeEventListener("resize", place); window.removeEventListener("scroll", place, true); window.removeEventListener("keydown", key); };
  }, [target, onDismiss]);
  return createPortal(<aside ref={popup} className="anchored-coach" role="region" aria-label={title} style={position ?? { bottom: 12, right: 12 }}>
    <header><strong>{title}</strong><button onClick={() => void invoke(`${subject}:dismiss`)} aria-label="Dismiss coach">×</button></header>
    {missing && <p role="status">The target is closed, hidden or unavailable. Open its normal workspace pane, use Locate target, or skip this step.</p>}
    <button onClick={() => void invoke(`${subject}:locate`)}>Locate target</button>
    {children}
  </aside>, document.body);
}
</storycode:copy>

### Assemble `apps/web/src/components/AnchoredCoach.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/AnchoredCoach.tsx"><storycode:paste select="#f-01e0c9a8e706-AnchoredCoach"/></storycode:file>

</storycode:doc>
