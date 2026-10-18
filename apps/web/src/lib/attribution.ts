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
