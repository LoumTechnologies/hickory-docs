<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/LineageSettings.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6f1fcaffbc4e"></a>
## `apps/web/src/components/LineageSettings.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Authorship display accepts non-Git providers](../../../../../guarantees/lineage/authorship-display-accepts-non-git-providers/contract.md).

<a id="f-6f1fcaffbc4e-CHANGED"></a>
### `CHANGED`

<storycode:copy id="f-6f1fcaffbc4e-CHANGED">
import { useEffect, useId, useState } from "react";
import { useUiCommands } from "../lib/uiActionBridge";
import { FeatureSettings } from "./FeatureSettings";
import { loadRibbonStyle, saveRibbonStyle, type RibbonStyle } from "../lib/ribbonStyle";
import { loadRibbonVisibility, saveRibbonVisibility, type RibbonVisibility } from "../lib/ribbonVisibility";

import { authorshipColors, setAuthorshipColors } from "../lib/attribution";

const CHANGED = "hickory:ribbon-preferences";
</storycode:copy>
<a id="f-6f1fcaffbc4e-useRibbonPresentation"></a>
### `useRibbonPresentation`

<storycode:copy id="f-6f1fcaffbc4e-useRibbonPresentation">
export function useRibbonPresentation() {
  const [style, setStyle] = useState(loadRibbonStyle);
  const [visibility, setVisibility] = useState(loadRibbonVisibility);
  useEffect(() => {
    const update = () => { setStyle(loadRibbonStyle()); setVisibility(loadRibbonVisibility()); };
    window.addEventListener(CHANGED, update);
    return () => window.removeEventListener(CHANGED, update);
  }, []);
  return { style, visibility };
}

</storycode:copy>
<a id="f-6f1fcaffbc4e-LineageSettings"></a>
### `LineageSettings`

<storycode:copy id="f-6f1fcaffbc4e-LineageSettings">
export function LineageSettings() {
  return <FeatureSettings label="Lineage settings"><LineageOptions /></FeatureSettings>;
}
</storycode:copy>
<a id="f-6f1fcaffbc4e-LineageOptions"></a>
### `LineageOptions`

<storycode:copy id="f-6f1fcaffbc4e-LineageOptions">
function LineageOptions() {
  const instance = useId(); const parent = `lineage-settings:${instance}`;
  const { style, visibility } = useRibbonPresentation();
  const [colors, setColors] = useState(authorshipColors);
  const changeColors = (enabled: boolean) => { setColors(enabled); setAuthorshipColors(enabled); };
  const changeStyle = (value: RibbonStyle) => { saveRibbonStyle(value); window.dispatchEvent(new Event(CHANGED)); };
  const changeVisibility = (value: RibbonVisibility) => { saveRibbonVisibility(value); window.dispatchEvent(new Event(CHANGED)); };
  const read = () => ({ revision: JSON.stringify([style, visibility, colors]), enabled: true, value: { style, visibility, colors, meaning: "Git credit alone does not authenticate human or agent origin" } });
  useUiCommands([
    { key: `${parent}/colors`, command: { label: "Set credited author colors", parent, subject: parent, parameters: { enabled: { type: "boolean" } }, prerequisites: [], read, execute: async args => { changeColors(Boolean(args.enabled)); return { status: "completed", value: null }; } } },
    { key: `${parent}/style`, command: { label: "Set lineage drawing style", parent, subject: parent, parameters: { style: { type: "string", maxLength: 16, choices: ["braces", "bands"] } }, prerequisites: [], read, execute: async args => { changeStyle(args.style as RibbonStyle); return { status: "completed", value: null }; } } },
    { key: `${parent}/visibility`, command: { label: "Set document connection visibility", parent, subject: parent, parameters: { visibility: { type: "string", maxLength: 16, choices: ["caret", "always"] } }, prerequisites: [], read, execute: async args => { changeVisibility(args.visibility as RibbonVisibility); return { status: "completed", value: null }; } } },
  ]);
  return <>
    <label><input type="checkbox" checked={colors} onChange={event => changeColors(event.target.checked)} /> Color credited authors</label>
    <p className="muted">Known human input uses teal; recorded agent text uses purple and a dashed stripe. Other colors distinguish credited authors with unknown origin; imported text uses dots. Git identity alone does not establish human or agent origin. Changed text loses its highlight until saved attribution refreshes.</p>
    <label>Draw connections as<select aria-label="Lineage style" value={style} onChange={event => {
      changeStyle(event.target.value as RibbonStyle);
    }}><option value="braces">Braces</option><option value="bands">Ribbons</option></select></label>
    <label>Document connections<select aria-label="Document lineage visibility" value={visibility} onChange={event => {
      changeVisibility(event.target.value as RibbonVisibility);
    }}><option value="caret">At the caret or on hover</option><option value="always">All connections</option></select></label>
    <p className="muted">Conversation connections follow expanded items. Change collapsed-item visibility in Agent settings.</p>
  </>;
}
</storycode:copy>

### Assemble `apps/web/src/components/LineageSettings.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/LineageSettings.tsx"><storycode:paste select="#f-6f1fcaffbc4e-CHANGED"/><storycode:paste select="#f-6f1fcaffbc4e-useRibbonPresentation"/><storycode:paste select="#f-6f1fcaffbc4e-LineageSettings"/><storycode:paste select="#f-6f1fcaffbc4e-LineageOptions"/></storycode:file>

</storycode:doc>
