<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/elements/patch/view.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Patch elements use the shared diff renderer](../../../../../../../guarantees/literate-projects/patch-view-uses-the-shared-diff-renderer/contract.md)

Captured SHA-256: `53d207239001c10aaa907e82c83bdc665c7d405851ee488a2e9f0ebf6c886a74`.

<book:entry path="apps/web/src/elements/patch/view.tsx">
import { PatchEditor } from "./Editor";
import type { ElementView } from "../types";

export const patchView: ElementView = {
  kind: "patch",
  draws: block => block.name === "patch",
  render: (slot, cx) => <PatchEditor slot={slot} cx={cx} />,
};
</book:entry>

</book:doc>
