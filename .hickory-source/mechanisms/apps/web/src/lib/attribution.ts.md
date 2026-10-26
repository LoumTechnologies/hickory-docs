<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/attribution.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-42f83e0db578"></a>
## `apps/web/src/lib/attribution.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Authorship display accepts non-Git providers](../../../../../guarantees/lineage/authorship-display-accepts-non-git-providers/contract.md).

<a id="f-42f83e0db578-TextOrigin"></a>
### `TextOrigin`

<storycode:copy id="f-42f83e0db578-TextOrigin">
import { api } from "../api/client";
import type { LineBlame } from "../editor/blameGutter";

export type TextOrigin = "human" | "agent" | "imported" | "unknown";
</storycode:copy>
<a id="f-42f83e0db578-AttributionLine"></a>
### `AttributionLine`

<storycode:copy id="f-42f83e0db578-AttributionLine">
export interface AttributionLine extends LineBlame { text: string; origin?: TextOrigin; provider?: string }
</storycode:copy>
<a id="f-42f83e0db578-AttributionProvider"></a>
### `AttributionProvider`

<storycode:copy id="f-42f83e0db578-AttributionProvider">
export interface AttributionProvider {
  id: string;
  label: string;
  lines(path: string): Promise<readonly AttributionLine[]>;
}
</storycode:copy>
<a id="f-42f83e0db578-git"></a>
### `git`

<storycode:copy id="f-42f83e0db578-git">
const git: AttributionProvider = {
  id: "git", label: "Git blame",
  async lines(path) { return (await api.blame(path)).lines.flatMap(line => typeof line.text === "string" ? [{ ...line, text: line.text, provider: line.provider ?? "Git blame", origin: line.origin ?? "unknown" as const }] : []); },
};
</storycode:copy>
<a id="f-42f83e0db578-provider"></a>
### `provider`

<storycode:copy id="f-42f83e0db578-provider">
let provider = git;
</storycode:copy>
<a id="f-42f83e0db578-ATTRIBUTION_CHANGED"></a>
### `ATTRIBUTION_CHANGED`

<storycode:copy id="f-42f83e0db578-ATTRIBUTION_CHANGED">
export const ATTRIBUTION_CHANGED = "hickory:attribution-changed";
</storycode:copy>
<a id="f-42f83e0db578-installAttributionProvider"></a>
### `installAttributionProvider`

Hosts may supply non-Git attribution. Uninstall restores the prior provider.

<storycode:copy id="f-42f83e0db578-installAttributionProvider">
/** Hosts may supply non-Git attribution. Uninstall restores the prior provider. */
export function installAttributionProvider(next: AttributionProvider): () => void {
  const previous = provider;
  provider = next;
  window.dispatchEvent(new Event(ATTRIBUTION_CHANGED));
  return () => { if (provider === next) { provider = previous; window.dispatchEvent(new Event(ATTRIBUTION_CHANGED)); } };
}
</storycode:copy>
<a id="f-42f83e0db578-attributionProvider"></a>
### `attributionProvider`

<storycode:copy id="f-42f83e0db578-attributionProvider">
export function attributionProvider(): AttributionProvider { return provider; }
</storycode:copy>
<a id="f-42f83e0db578-KEY"></a>
### `KEY`

<storycode:copy id="f-42f83e0db578-KEY">
const KEY = "hickory.authorshipColors";
</storycode:copy>
<a id="f-42f83e0db578-authorshipColors"></a>
### `authorshipColors`

<storycode:copy id="f-42f83e0db578-authorshipColors">
export function authorshipColors(): boolean { return localStorage.getItem(KEY) === "1"; }
</storycode:copy>
<a id="f-42f83e0db578-setAuthorshipColors"></a>
### `setAuthorshipColors`

<storycode:copy id="f-42f83e0db578-setAuthorshipColors">
export function setAuthorshipColors(shown: boolean): void {
  localStorage.setItem(KEY, shown ? "1" : "0");
  window.dispatchEvent(new Event(ATTRIBUTION_CHANGED));
}
</storycode:copy>
<a id="f-42f83e0db578-authorColor"></a>
### `authorColor`

Stable credited-author palette; origin is independent of commit identity.

<storycode:copy id="f-42f83e0db578-authorColor">
/** Stable credited-author palette; origin is independent of commit identity. */
export function authorColor(author: string): number {
  let hash = 0;
  for (const char of author) hash = (hash * 31 + char.charCodeAt(0)) | 0;
  return Math.abs(hash % 6);
}
</storycode:copy>

### Assemble `apps/web/src/lib/attribution.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/attribution.ts"><storycode:paste select="#f-42f83e0db578-TextOrigin"/><storycode:paste select="#f-42f83e0db578-AttributionLine"/><storycode:paste select="#f-42f83e0db578-AttributionProvider"/><storycode:paste select="#f-42f83e0db578-git"/><storycode:paste select="#f-42f83e0db578-provider"/><storycode:paste select="#f-42f83e0db578-ATTRIBUTION_CHANGED"/><storycode:paste select="#f-42f83e0db578-installAttributionProvider"/><storycode:paste select="#f-42f83e0db578-attributionProvider"/><storycode:paste select="#f-42f83e0db578-KEY"/><storycode:paste select="#f-42f83e0db578-authorshipColors"/><storycode:paste select="#f-42f83e0db578-setAuthorshipColors"/><storycode:paste select="#f-42f83e0db578-authorColor"/></storycode:file>

</storycode:doc>
