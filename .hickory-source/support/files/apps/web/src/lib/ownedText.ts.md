<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/ownedText.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `93a7350cf965e3556fd73c217c5f0072b536f0b6b8cd3f2a932614eb67da8145`.

<book:entry path="apps/web/src/lib/ownedText.ts">
import { Fragment, isValidElement, type ReactNode } from "react";

/** Literal text owned by this render, not a DOM scan or evaluation of child components. */
export function ownedText(value: ReactNode): { text: string; unresolved_components: number } {
  const chunks: string[] = [];
  let unresolved = 0;
  const visit = (node: ReactNode) => {
    if (typeof node === "string" || typeof node === "number") { chunks.push(String(node)); return; }
    if (Array.isArray(node)) { node.forEach(visit); return; }
    if (!isValidElement<{ children?: ReactNode; hidden?: boolean; open?: boolean; style?: { display?: string } }>(node)) return;
    if (node.props.hidden || node.props.style?.display === "none") return;
    if (typeof node.type !== "string" && node.type !== Fragment) { unresolved++; return; }
    if (node.type === "details" && !node.props.open) {
      const children = Array.isArray(node.props.children) ? node.props.children : [node.props.children];
      children.filter(child => isValidElement(child) && child.type === "summary").forEach(visit);
      return;
    }
    visit(node.props.children);
  };
  visit(value);
  return { text: chunks.join(""), unresolved_components: unresolved };
}
</book:entry>

</book:doc>
