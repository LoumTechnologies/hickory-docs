<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/useRibbonActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `998670f90d54f8534a77dc5dd6f25d94b10e4b866d39550dc2108c1a50a0105a`.

<book:entry path="apps/web/src/shell/useRibbonActions.ts">
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
const identities = new WeakMap<object, string>();
function identity(owner: object) { let id = identities.get(owner); if (!id) { id = crypto.randomUUID(); identities.set(owner, id); } return id; }
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
</book:entry>

</book:doc>
