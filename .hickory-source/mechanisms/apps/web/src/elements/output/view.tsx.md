<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/elements/output/view.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5c5466e7bfa2"></a>
## `apps/web/src/elements/output/view.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-5c5466e7bfa2-outputView"></a>
### `outputView`

<storycode:copy id="f-5c5466e7bfa2-outputView">
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
</storycode:copy>

### Assemble `apps/web/src/elements/output/view.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/output/view.tsx"><storycode:paste select="#f-5c5466e7bfa2-outputView"/></storycode:file>

</storycode:doc>
