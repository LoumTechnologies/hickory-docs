<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/graph/GraphInspectorLabel.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `429ae73b6d7876a42ec23b6dea0630734affac33fc8fce0ed61261f47588b48f`.

<book:entry path="apps/web/src/components/graph/GraphInspectorLabel.tsx">
import type { UiActor } from "../../lib/uiActions";
import { useGraphLabelDraft } from "./useGraphLabelDraft";

export function GraphInspectorLabel({ identity, value, commit }: {
  identity: string; value: string; commit: (value: string, actor?: UiActor) => boolean | void;
}) {
  const label = useGraphLabelDraft({ identity: `inspector:${identity}`, value, initiallyOpen: true, commit });
  return <>
    {label.error && <span role="alert">{label.error}</span>}
    <input aria-label="Line label" className="graph-inspector__label" placeholder="label…"
      value={label.value} onFocus={() => { if (!label.editing) label.open(); }}
      onChange={event => label.change(event.target.value)} onBlur={() => label.finish(true)}
      onKeyDown={event => {
        if (event.key === "Enter") event.currentTarget.blur();
        if (event.key === "Escape") { event.stopPropagation(); label.finish(false); }
      }} />
  </>;
}
</book:entry>

</book:doc>
