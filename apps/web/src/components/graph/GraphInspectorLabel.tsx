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
