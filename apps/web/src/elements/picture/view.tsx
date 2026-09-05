import { PicturePanel } from "../../components/PicturePanel";
import { assetUrl } from "../../editor/mdLinks";
import { isPicturePath } from "../../editor/hickDoc";
import { resolveTarget } from "../../lib/mdLinks";
import type { ElementView } from "../types";

/** A `<hick:file>` that writes a picture: shown as the picture it writes. */
export const pictureView: ElementView = {
  kind: "picture",
  draws: (block) => block.name === "file" && isPicturePath(block.attrs.path),
  render(slot, cx) {
    const target = resolveTarget(cx.path, slot.picture?.path ?? "");
    return <PicturePanel src={target ? assetUrl(target) : null} path={slot.picture?.path ?? ""} />;
  },
};
