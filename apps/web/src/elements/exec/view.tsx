import { CellPanel } from "../../components/CellPanel";
import { matchExecBlock } from "../../lib/blockMatch";
import type { ElementView } from "../types";

/** `<hick:exec>`: a cell, drawn as its command, its status and its output. */
export const execView: ElementView = {
  kind: "exec",
  draws: (block) => block.name === "exec",
  render(slot, cx) {
    const block = matchExecBlock({ span: slot.span, index: slot.index }, cx.execBlocks);
    return (
      <CellPanel
        block={block}
        running={block ? cx.runningCells.has(block.id) : false}
        command={slot.text}
        replay={cx.replaying.includes(slot.at)}
      />
    );
  },
};
