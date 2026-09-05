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
