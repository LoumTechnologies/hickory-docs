<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/elements/exec/view.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `8d8b9041c45489ddd89d7459f17bc45447b1248f64b8614765eae59ace0bd4f5`.

<book:entry path="apps/web/src/elements/exec/view.tsx">
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
</book:entry>

</book:doc>
