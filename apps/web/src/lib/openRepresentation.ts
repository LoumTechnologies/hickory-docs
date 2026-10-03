import { representations, type Backing } from "../api/representations";
export const OPEN_REPRESENTATION = "hickory.open-representation";
export function showRepresentation(id: string, base?: string, target?: string) {
  window.dispatchEvent(new CustomEvent(OPEN_REPRESENTATION, { detail: { id, base, target } }));
}
export async function openLiterate(backing: Backing, base?: string, target?: string) {
  const view = await representations.create(backing);
  showRepresentation(view.id, base, target);
}
