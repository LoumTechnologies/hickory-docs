// docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
export const COLLAPSED_LINEAGE_KEY = "hickory.conversationCollapsedLineage";
export function loadCollapsedLineage(): boolean {
  return typeof localStorage !== "undefined" && localStorage.getItem(COLLAPSED_LINEAGE_KEY) === "true";
}
export function saveCollapsedLineage(value: boolean): void {
  localStorage.setItem(COLLAPSED_LINEAGE_KEY, String(value));
}

/** The innermost rendered element is the anchor, not its hidden source lines. */
export function conversationAnchor(host: HTMLElement, from: number, to: number, includeCollapsed: boolean): HTMLElement | null {
  const matches = [...host.querySelectorAll<HTMLElement>("[data-session-from]")].filter(element =>
    Number(element.dataset.sessionFrom) <= from && Number(element.dataset.sessionTo) >= to);
  const item = matches.sort((a, b) => (Number(a.dataset.sessionTo) - Number(a.dataset.sessionFrom)) -
    (Number(b.dataset.sessionTo) - Number(b.dataset.sessionFrom)))[0];
  if (!item) return null;
  const ownFold = item.firstElementChild instanceof HTMLDetailsElement ? item.firstElementChild : null;
  let parent: HTMLElement | null = ownFold ?? item;
  while (parent && host.contains(parent)) {
    if (parent instanceof HTMLDetailsElement && !parent.open) {
      return includeCollapsed ? parent.querySelector<HTMLElement>("summary") : null;
    }
    parent = parent.parentElement;
  }
  return item;
}
