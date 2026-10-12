import { PatchEditor } from "./Editor";
import type { ElementView } from "../types";

export const patchView: ElementView = {
  kind: "patch",
  draws: block => block.name === "patch",
  render: (slot, cx) => <PatchEditor slot={slot} cx={cx} />,
};
