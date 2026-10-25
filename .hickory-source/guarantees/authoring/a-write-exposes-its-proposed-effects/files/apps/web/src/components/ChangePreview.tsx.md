<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/ChangePreview.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A write exposes its proposed effects](../../../../../contract.md)

Captured SHA-256: `2400d202996f3f42aee011fed2584c01bcfb6c57f83c97d82c1f4c6dedfff44e`.

<book:entry path="apps/web/src/components/ChangePreview.tsx">
import { useState, type ReactNode } from "react";
import { EvidenceDisclosure } from "./EvidenceDisclosure";
import { useUiReading } from "../lib/uiReading";
import { ownedText } from "../lib/ownedText";

/** Inspect the proposed bytes and effects before choosing the write. */
export function ChangePreview({ heading, effects, children, collapsible = false }: {
  heading: string; effects?: ReactNode; children: ReactNode; collapsible?: boolean;
}) {
  const [expanded, setExpanded] = useState(false);
  const content = <>{children}{effects && <p className="insert-menu__hint">{effects}</p>}</>;
  const value = { heading, preview: ownedText(children), effects: ownedText(effects) };
  return collapsible
    ? <EvidenceDisclosure className="change-preview" label={heading} value={value} onOpenChange={setExpanded}>{expanded && content}</EvidenceDisclosure>
    : <section className="insert-menu__preview change-preview" aria-label={heading}>
      <PreviewReading value={value} />
      <h3 className="insert-menu__preview-title">{heading}</h3>{content}
    </section>;
}

function PreviewReading({ value }: { value: unknown }) {
  useUiReading("change preview", value);
  return null;
}
</book:entry>

</book:doc>
