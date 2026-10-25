<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/elements/picture/view.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `36f9be1f72b1829174f30e2980dc0010af8eff4d3698f2f91edd999d7600aa18`.

<book:entry path="apps/web/src/elements/picture/view.tsx">
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
</book:entry>

</book:doc>
