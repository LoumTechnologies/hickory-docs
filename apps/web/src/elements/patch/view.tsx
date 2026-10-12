import { DiffView } from "../../components/DiffView";
import type { ElementView } from "../types";

/** Saved patch elements use the change-review renderer. Project actions select
 * completed endpoints; these lines describe the local transformation only. */
export const patchView: ElementView = {
  kind: "patch",
  draws: block => block.name === "patch",
  render(slot) {
    return <DiffView path="Project patch" diff={slot.text.replace(/^\r?\n/, "")}
      binary={false} staged={false} label="New · completed project; Old · scope input" />;
  },
};
