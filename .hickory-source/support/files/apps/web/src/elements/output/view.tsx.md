<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/elements/output/view.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `28029f09e2f96c236bdb8f60ccb633d336887a2e0220cd3a8968cff689af79a6`.

<book:entry path="apps/web/src/elements/output/view.tsx">
import { notebookOutputs } from "../../lib/notebookOutput";
import { NotebookOutputs } from "../../components/NotebookOutputs";
import type { ElementView } from "../types";

/** A derived execution result. Its source remains copyable but this view gives
 * it no editing affordance: changing it would invalidate its `hash`. */
export const outputView: ElementView = {
  kind: "output",
  draws: (block) => block.name === "output",
  render(slot) {
    const rich = notebookOutputs(slot.text);
    return (
      <section className="rounded border border-slate-200 bg-slate-50 p-3" data-hick-output>
        <div className="mb-2 text-xs font-medium uppercase tracking-wide text-slate-500">
          Output
        </div>
        <pre className="m-0 overflow-auto whitespace-pre-wrap font-mono text-sm">{rich.text}</pre>
        <NotebookOutputs outputs={rich.outputs} />
      </section>
    );
  },
};
</book:entry>

</book:doc>
