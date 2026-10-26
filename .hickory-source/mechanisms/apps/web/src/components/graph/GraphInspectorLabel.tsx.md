<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/graph/GraphInspectorLabel.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-82ef23445734"></a>
## `apps/web/src/components/graph/GraphInspectorLabel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-82ef23445734-GraphInspectorLabel"></a>
### `GraphInspectorLabel`

<storycode:copy id="f-82ef23445734-GraphInspectorLabel">
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
</storycode:copy>

### Assemble `apps/web/src/components/graph/GraphInspectorLabel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/graph/GraphInspectorLabel.tsx"><storycode:paste select="#f-82ef23445734-GraphInspectorLabel"/></storycode:file>

</storycode:doc>
