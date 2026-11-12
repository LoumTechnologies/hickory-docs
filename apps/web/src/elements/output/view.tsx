import { InlineTerminal } from "../../terminal/WatchingTerminal";
import { notebookOutputs } from "../../lib/notebookOutput";
import { NotebookOutputs } from "../../components/NotebookOutputs";
import type { ElementView } from "../types";

/** A derived execution result. Its source remains copyable but this view gives
 * it no editing affordance: changing it would invalidate its `hash`. */
export const outputView: ElementView = {
  kind: "output",
  draws: (block) => block.name === "output" || block.name === "expect",
  render(slot) {
    const rich = notebookOutputs(slot.text);
    return (
      <section data-hick-output>
        <InlineTerminal text={slot.sourceElement === "expect" ? slot.text.replace(/^\r?\n/, "") : rich.text} label={slot.sourceElement === "expect" ? "Expected output terminal" : "Output terminal"} />
        {slot.sourceElement !== "expect" && <NotebookOutputs outputs={rich.outputs} />}
      </section>
    );
  },
};
