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
    if (slot.outputOnly && !block?.transcript.length && slot.recordedOutput === undefined && !block?.runError && !running) return null;
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
