import type { ElementView } from "../types";

/** A derived execution result. Its source remains copyable but this view gives
 * it no editing affordance: changing it would invalidate its `hash`. */
export const outputView: ElementView = {
  kind: "output",
  draws: (block) => block.name === "output",
  render(slot) {
    return (
      <section className="rounded border border-slate-200 bg-slate-50 p-3" data-hick-output>
        <div className="mb-2 text-xs font-medium uppercase tracking-wide text-slate-500">
          Output
        </div>
        <pre className="m-0 overflow-auto whitespace-pre-wrap font-mono text-sm">{slot.text}</pre>
      </section>
    );
  },
};
