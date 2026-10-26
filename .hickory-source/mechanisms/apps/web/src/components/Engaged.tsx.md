<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/Engaged.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0d070de8f2cc"></a>
## `apps/web/src/components/Engaged.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-0d070de8f2cc-Engaged"></a>
### `Engaged`

The map-in-a-page rule, as a wrapper any embedded widget can stand in.
A widget portalled into the document competes with it for the wheel: a
table that scrolls internally, a canvas that zooms, a terminal that pages
all turn "scrolling past" into "trapped inside". The rule that fixes every
case the same way: **the wheel belongs to whoever owns the moment.** At
rest the widget is scenery and the wheel scrolls the document; clicking
into the widget engages it (focus is the signal, exactly as it is for
keys — see GraphEditorPanel, which implements the same rule inline because
it must also feed the state to React Flow); Escape, or clicking away,
hands the wheel back.
What "engaged" gates is the widget's own business, keyed off the
`engaged-gate--on` class — the table hides its internal overflow until
engaged, the canvas turns zoom-on-scroll on. This wrapper only owns the
signal.

<storycode:copy id="f-0d070de8f2cc-Engaged">
// The map-in-a-page rule, as a wrapper any embedded widget can stand in.
//
// A widget portalled into the document competes with it for the wheel: a
// table that scrolls internally, a canvas that zooms, a terminal that pages
// all turn "scrolling past" into "trapped inside". The rule that fixes every
// case the same way: **the wheel belongs to whoever owns the moment.** At
// rest the widget is scenery and the wheel scrolls the document; clicking
// into the widget engages it (focus is the signal, exactly as it is for
// keys — see GraphEditorPanel, which implements the same rule inline because
// it must also feed the state to React Flow); Escape, or clicking away,
// hands the wheel back.
//
// What "engaged" gates is the widget's own business, keyed off the
// `engaged-gate--on` class — the table hides its internal overflow until
// engaged, the canvas turns zoom-on-scroll on. This wrapper only owns the
// signal.

import type { ReactNode } from "react";
import { useWidgetEngagement } from "./useWidgetEngagement";

export function Engaged({ className, children }: { className?: string; children: ReactNode }) {
  const engagement = useWidgetEngagement("embedded widget");
  return <div ref={engagement.root} {...engagement.handlers} tabIndex={-1}
    className={`engaged-gate${engagement.engaged ? " engaged-gate--on" : ""}${className ? ` ${className}` : ""}`}>
    {children}
  </div>;
}
</storycode:copy>

### Assemble `apps/web/src/components/Engaged.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/Engaged.tsx"><storycode:paste select="#f-0d070de8f2cc-Engaged"/></storycode:file>

</storycode:doc>
