import type { ReactNode } from "react";

/** Occasional tab actions stay discoverable without occupying the toolbar. */
export function MoreActions({ children, label = "More actions" }: { children: ReactNode; label?: string }) {
  return <details className="more-actions"><summary aria-label={label}>⋯</summary>
    <div className="more-actions__items" onClick={event => {
      if ((event.target as HTMLElement).closest("button")) event.currentTarget.closest("details")?.removeAttribute("open");
    }}>{children}</div>
  </details>;
}
