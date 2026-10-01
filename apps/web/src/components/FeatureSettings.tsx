import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import "./FeatureSettings.css";

/** Settings stay beside the feature they control, outside pane clipping. */
export function FeatureSettings({ label, children }: { label: string; children: ReactNode }) {
  const [position, setPosition] = useState<{ right: number; top?: number; bottom?: number } | null>(null);
  const id = useId();
  const root = useRef<HTMLSpanElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  const close = () => setPosition(null);
  useEffect(() => {
    if (!position) return;
    const dismiss = (event: PointerEvent) => {
      const target = event.target as Node;
      if (!root.current?.contains(target) && !panel.current?.contains(target)) close();
    };
    document.addEventListener("pointerdown", dismiss);
    window.addEventListener("resize", close);
    return () => { document.removeEventListener("pointerdown", dismiss); window.removeEventListener("resize", close); };
  }, [position]);
  const escape = (event: React.KeyboardEvent) => { if (event.key === "Escape") close(); };
  return <span ref={root} className="feature-settings" onKeyDown={escape}>
    <button type="button" className="feature-settings__gear" aria-label={label} aria-expanded={!!position} aria-controls={id}
      data-tip={label} onClick={() => {
        if (position) return close();
        const rect = root.current!.getBoundingClientRect();
        const right = Math.max(8, Math.min(window.innerWidth - rect.right, window.innerWidth - Math.min(352, window.innerWidth * .8) - 8));
        setPosition(root.current!.closest(".status-bar")
          ? { right, bottom: window.innerHeight - rect.top }
          : { right, top: rect.bottom });
      }}><span aria-hidden="true">⚙</span></button>
    {position && createPortal(<div ref={panel} id={id} style={position} className="feature-settings__panel" role="region" aria-label={label} onKeyDown={escape}>
      <strong>{label}</strong>{children}
    </div>, document.body)}
  </span>;
}
