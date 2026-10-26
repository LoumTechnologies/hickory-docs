<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/elements/exec/view.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b875f66d311d"></a>
## `apps/web/src/elements/exec/view.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b875f66d311d-execView"></a>
### `execView`

<storycode:copy id="f-b875f66d311d-execView">
import { CellPanel } from "../../components/CellPanel";
import { matchExecBlock } from "../../lib/blockMatch";
import type { ElementView } from "../types";

/** `<hick:exec>`: a cell, drawn as its command, its status and its output. */
export const execView: ElementView = {
  kind: "exec",
  draws: (block) => block.name === "exec",
  render(slot, cx) {
    const block = matchExecBlock({ span: slot.span, index: slot.index }, cx.execBlocks);
    const running = block ? cx.runningCells.has(block.id) : false;
    if (slot.outputOnly && !block?.transcript?.length && slot.recordedOutput === undefined && !block?.runError && !running) return null;
    return (
      <CellPanel
        block={block}
        recordedOutput={slot.recordedOutput}
        running={running}
        command={slot.outputOnly ? undefined : slot.text}
        replay={cx.replaying.includes(slot.at)}
      />
    );
  },
};
</storycode:copy>

### Assemble `apps/web/src/elements/exec/view.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/exec/view.tsx"><storycode:paste select="#f-b875f66d311d-execView"/></storycode:file>

</storycode:doc>
