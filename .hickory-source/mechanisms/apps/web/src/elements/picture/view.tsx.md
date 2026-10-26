<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/elements/picture/view.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6b4c127c4350"></a>
## `apps/web/src/elements/picture/view.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-6b4c127c4350-pictureView"></a>
### `pictureView`

<storycode:copy id="f-6b4c127c4350-pictureView">
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
</storycode:copy>

### Assemble `apps/web/src/elements/picture/view.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/picture/view.tsx"><storycode:paste select="#f-6b4c127c4350-pictureView"/></storycode:file>

</storycode:doc>
