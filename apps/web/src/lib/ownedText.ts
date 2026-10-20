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
