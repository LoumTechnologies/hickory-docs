<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/elements/math/view.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `55aaa7051e0bf2577f32cbd1269689118eee3cb7d27a6e0919d293d7f67222a3`.

<book:entry path="apps/web/src/elements/math/view.tsx">
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
</book:entry>

</book:doc>
