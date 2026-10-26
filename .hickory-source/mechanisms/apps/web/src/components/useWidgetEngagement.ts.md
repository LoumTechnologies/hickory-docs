<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/useWidgetEngagement.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c8af150e1c34"></a>
## `apps/web/src/components/useWidgetEngagement.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c8af150e1c34-useWidgetEngagement"></a>
### `useWidgetEngagement`

<storycode:copy id="f-c8af150e1c34-useWidgetEngagement">
import { useId, useRef, useState, type HTMLAttributes } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";

/** Focus owns wheel behavior; do not blur/commit an active field implicitly. */
export function useWidgetEngagement(label: string) {
  const [on, paintOn] = useState(false);
  const root = useRef<HTMLDivElement>(null), current = useRef(false);
  const parent = `widget-engagement:${useId()}`;
  const setOn = (next: boolean) => { current.current = next; paintOn(next); };
  const revision = useUiRevision(() => [current.current]);
  const engage = (next: boolean): UiOutcome => {
    const element = root.current;
    if (!element?.isConnected) return { status: "unavailable", reason: "Embedded widget was closed" };
    const focused = document.activeElement;
    const leavesField = focused instanceof HTMLElement && ((!next && element.contains(focused) && focused.closest("input,textarea,[contenteditable]")) || (next && !element.contains(focused) && focused.closest("input,textarea")));
    if (leavesField) return { status: "requires_input", request: `${parent}:finish-editor`, reason: "Finish or cancel the focused field before moving widget focus" };
    if (next) {
      if (!element.contains(focused)) element.focus({ preventScroll: true });
      if (!element.contains(document.activeElement)) return { status: "unavailable", reason: "The widget could not receive focus" };
    } else if (focused instanceof HTMLElement && element.contains(focused)) focused.blur();
    setOn(next);
    return { status: "completed", value: { engaged: next, focus_inside: element.contains(document.activeElement) } };
  };
  const invoke = useUiCommands([{ key: parent, command: {
    parent, subject: parent, label: `Set ${label} engagement`, parameters: { engaged: { type: "boolean" } }, prerequisites: ["Finish or cancel a focused field before releasing widget scrolling"],
    read: () => ({ revision: revision(), enabled: !!root.current?.isConnected, value: { engaged: current.current } }), execute: async args => engage(Boolean(args.engaged)),
  } }]);
  useUiReading(`${label} engagement`, { engaged: on, scope: "Focus and wheel ownership for this embedded widget; no direct source edit or execution; normal focus callbacks can have effects" });
  const handlers: Pick<HTMLAttributes<HTMLDivElement>, "onFocus" | "onBlur" | "onKeyDown" | "onPointerDownCapture"> = {
    onFocus: () => setOn(true),
    onBlur: event => { if (!event.currentTarget.contains(event.relatedTarget as Element | null)) setOn(false); },
    onKeyDown: event => {
      const target = event.target as HTMLElement;
      if (event.key === "Escape" && !target.closest("input, textarea")) {
        event.stopPropagation(); void invoke(parent, { engaged: false }).catch(() => {});
      }
    },
    onPointerDownCapture: event => {
      const element = event.currentTarget, target = event.target as HTMLElement;
      if (target.closest("input, textarea, [contenteditable]")) return;
      if (!element.contains(document.activeElement)) element.focus({ preventScroll: true });
    },
  };
  return { root, engaged: on, handlers };
}
</storycode:copy>

### Assemble `apps/web/src/components/useWidgetEngagement.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/useWidgetEngagement.ts"><storycode:paste select="#f-c8af150e1c34-useWidgetEngagement"/></storycode:file>

</storycode:doc>
