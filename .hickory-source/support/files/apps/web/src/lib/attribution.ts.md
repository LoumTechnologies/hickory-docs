<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/attribution.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Authorship display accepts non-Git providers](../../../../../../guarantees/lineage/authorship-display-accepts-non-git-providers/contract.md)

Captured SHA-256: `7537bc811b0d9962ec73142c87a8b975b465588f7b6dec1ebb0531d92da503a9`.

<book:entry path="apps/web/src/lib/attribution.ts">
import { api } from "../api/client";
import type { LineBlame } from "../editor/blameGutter";

export type TextOrigin = "human" | "agent" | "imported" | "unknown";
export interface AttributionLine extends LineBlame { text: string; origin?: TextOrigin; provider?: string }
export interface AttributionProvider {
  id: string;
  label: string;
  lines(path: string): Promise<readonly AttributionLine[]>;
}
const git: AttributionProvider = {
  id: "git", label: "Git blame",
  async lines(path) { return (await api.blame(path)).lines.flatMap(line => typeof line.text === "string" ? [{ ...line, text: line.text, provider: line.provider ?? "Git blame", origin: line.origin ?? "unknown" as const }] : []); },
};
let provider = git;
export const ATTRIBUTION_CHANGED = "hickory:attribution-changed";
/** Hosts may supply non-Git attribution. Uninstall restores the prior provider. */
export function installAttributionProvider(next: AttributionProvider): () => void {
  const previous = provider;
  provider = next;
  window.dispatchEvent(new Event(ATTRIBUTION_CHANGED));
  return () => { if (provider === next) { provider = previous; window.dispatchEvent(new Event(ATTRIBUTION_CHANGED)); } };
}
export function attributionProvider(): AttributionProvider { return provider; }
const KEY = "hickory.authorshipColors";
export function authorshipColors(): boolean { return localStorage.getItem(KEY) === "1"; }
export function setAuthorshipColors(shown: boolean): void {
  localStorage.setItem(KEY, shown ? "1" : "0");
  window.dispatchEvent(new Event(ATTRIBUTION_CHANGED));
}
/** Stable credited-author palette; origin is independent of commit identity. */
export function authorColor(author: string): number {
  let hash = 0;
  for (const char of author) hash = (hash * 31 + char.charCodeAt(0)) | 0;
  return Math.abs(hash % 6);
}
</book:entry>

</book:doc>
