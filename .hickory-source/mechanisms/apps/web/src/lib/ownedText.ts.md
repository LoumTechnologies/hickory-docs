<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/ownedText.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8099adee74d0"></a>
## `apps/web/src/lib/ownedText.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-8099adee74d0-ownedText"></a>
### `ownedText`

<storycode:copy id="f-8099adee74d0-ownedText">
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
</storycode:copy>

### Assemble `apps/web/src/lib/ownedText.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/ownedText.ts"><storycode:paste select="#f-8099adee74d0-ownedText"/></storycode:file>

</storycode:doc>
