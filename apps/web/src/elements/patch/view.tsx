import { PatchEditor } from "./Editor";
import type { ElementView } from "../types";

export const patchView: ElementView = {
  kind: "patch",
  draws: block => block.name === "patch" && !!block.attrs.id,
  render: (slot, cx) => <PatchEditor slot={slot} cx={cx} />,
};
