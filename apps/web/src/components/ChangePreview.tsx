import { useState, type ReactNode } from "react";

/** Inspect the proposed bytes and effects before choosing the write. */
export function ChangePreview({ heading, effects, children, collapsible = false }: {
  heading: string; effects?: ReactNode; children: ReactNode; collapsible?: boolean;
}) {
  const [expanded, setExpanded] = useState(false);
  const content = <>{children}{effects && <p className="insert-menu__hint">{effects}</p>}</>;
  return collapsible
    ? <details className="change-preview" onToggle={event => setExpanded(event.currentTarget.open)}><summary>{heading}</summary>{expanded && content}</details>
    : <section className="insert-menu__preview change-preview" aria-label={heading}>
      <h3 className="insert-menu__preview-title">{heading}</h3>{content}
    </section>;
}
