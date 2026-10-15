import type { ReactNode } from "react";
import "./MarginActions.css";

/** Editor chrome occupies the right margin; opening it never adds text rows. */
export function MarginActions({ label, glyph, children, className = "" }: {
  label: string; glyph: string; children: ReactNode; className?: string;
}) {
  return <details className={`margin-actions ${className}`} onKeyDown={event => {
    if (event.key === "Escape") {
      event.currentTarget.open = false;
      event.currentTarget.querySelector("summary")?.focus();
      event.stopPropagation();
    }
  }}>
    <summary aria-label={label} data-tip={label}>{glyph}</summary>
    <div className="margin-actions__panel" aria-label={label}>{children}</div>
  </details>;
}
