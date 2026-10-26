<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/elements/math/view.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-43875ae288d1"></a>
## `apps/web/src/elements/math/view.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-43875ae288d1-mathView"></a>
### `mathView`

<storycode:copy id="f-43875ae288d1-mathView">
import { MathPanel } from "../../components/MathPanel";
import type { ElementView } from "../types";

/** `<hick:math>`: a formula, typeset. */
export const mathView: ElementView = {
  kind: "math",
  draws: (block) => block.name === "math",
  render: (slot) => (
    <div className="rendered-math">
      <MathPanel source={slot.text} />
    </div>
  ),
};
</storycode:copy>

### Assemble `apps/web/src/elements/math/view.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/math/view.tsx"><storycode:paste select="#f-43875ae288d1-mathView"/></storycode:file>

</storycode:doc>
