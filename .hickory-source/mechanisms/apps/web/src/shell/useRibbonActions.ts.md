<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/useRibbonActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-925155ba4bba"></a>
## `apps/web/src/shell/useRibbonActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-925155ba4bba-RibbonActionShape"></a>
### `RibbonActionShape`

<storycode:copy id="f-925155ba4bba-RibbonActionShape">
import { useId } from "react";
import type { EditorState } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";
import { useUiCommands } from "../lib/uiActionBridge";
import type { UiOutcome } from "../lib/uiActions";
import { useUiReading } from "../lib/uiReading";
import { openReview, reviewError, reviewRelationship, type ReviewTarget } from "../lib/reviews";
import type { RibbonFamily, RibbonTarget } from "./Ribbons";
export interface RibbonActionShape {
  key: string; family: RibbonFamily; target: RibbonTarget; review?: ReviewTarget;
  hl: { view: EditorView; from: number; to: number }[];
  states: readonly EditorState[];
}
</storycode:copy>
<a id="f-925155ba4bba-identities"></a>
### `identities`

<storycode:copy id="f-925155ba4bba-identities">
const identities = new WeakMap<object, string>();
</storycode:copy>
<a id="f-925155ba4bba-identity"></a>
### `identity`

<storycode:copy id="f-925155ba4bba-identity">
function identity(owner: object) { let id = identities.get(owner); if (!id) { id = crypto.randomUUID(); identities.set(owner, id); } return id; }
</storycode:copy>
<a id="f-925155ba4bba-useRibbonActions"></a>
### `useRibbonActions`

Shares semantic ribbon commands; no coordinate click-through or hidden source scraping.

<storycode:copy id="f-925155ba4bba-useRibbonActions">
/** Shares semantic ribbon commands; no coordinate click-through or hidden source scraping. */
export function useRibbonActions<T extends RibbonActionShape>(shapes: readonly T[], shown: (shape: T) => boolean, reveal: (shape: T, visible: boolean) => void, navigate: ((shape: T) => void) | undefined) {
  const parent = `ribbon-connections:${useId()}`;
  const current = (shape: T) => shape.hl.every((side, index) => side.view.dom.isConnected && side.view.state === shape.states[index]);
  const revision = (shape: T) => JSON.stringify([shape.key, shape.family, shape.target, shape.review, shown(shape), shape.hl.map(side => [identity(side.view), identity(side.view.state), side.from, side.to]), current(shape)]);
  useUiReading("ribbon connection disclosure", shapes.map((shape, index) => ({ index, family: shape.family, shown: shown(shape), current: current(shape), ...(shown(shape) ? { target: shape.target } : {}) })));
  const invoke = useUiCommands(shapes.flatMap((shape, index) => ["reveal", "hide", "navigate", "review"].map(operation => ({ key: `${parent}:${shape.key}:${operation}`, command: {
    label: `${operation === "navigate" ? "Follow" : operation === "review" ? "Review" : operation === "hide" ? "Hide" : "Reveal"} ${shape.family} connection ${index + 1}`,
    parent, subject: `${parent}:${shape.key}`, parameters: {}, prerequisites: ["Current measured connection and captured editor states; reveal before following or reviewing"],
    read: () => ({ revision: revision(shape), enabled: current(shape) && (operation === "reveal" || operation === "hide" || shown(shape) && (operation !== "navigate" || !!navigate)), reason: "Reveal the current connection and wait for current editor measurement; navigation needs an attached handler" }),
    execute: async (): Promise<UiOutcome> => {
      if (!current(shape)) return { status: "unavailable", reason: "The connection's measured source or editor owner changed" };
      if (operation === "reveal" || operation === "hide") { reveal(shape, operation === "reveal"); return { status: "completed", value: { disclosure_requested: operation === "reveal" } }; }
      if (!shown(shape)) return { status: "unavailable", reason: "Reveal this connection before following or reviewing it" };
      if (operation === "navigate") {
        if (!navigate) return { status: "unavailable", reason: "This connection has no navigation handler" };
        navigate(shape); return { status: "completed", value: { navigation_requested: true, target: { ...shape.target } } };
      }
      if (shape.review) openReview(shape.review);
      else {
        const side = shape.hl[0];
        if (!side) return { status: "unavailable", reason: "This connection has no captured source for a review" };
        await reviewRelationship(side.view, [side.from, side.to], { family: shape.family, target: shape.target, key: shape.key });
      }
      return { status: "completed", value: { review_requested: true, human_resolution: false } };
    },
  } }))));
  return (shape: T, operation: "navigate" | "review") => { void invoke(`${parent}:${shape.key}:${operation}`).catch(reviewError); };
}
</storycode:copy>

### Assemble `apps/web/src/shell/useRibbonActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/useRibbonActions.ts"><storycode:paste select="#f-925155ba4bba-RibbonActionShape"/><storycode:paste select="#f-925155ba4bba-identities"/><storycode:paste select="#f-925155ba4bba-identity"/><storycode:paste select="#f-925155ba4bba-useRibbonActions"/></storycode:file>

</storycode:doc>
