import { useEffect, useId, useState } from "react";
import { useUiCommands } from "../lib/uiActionBridge";
import { FeatureSettings } from "./FeatureSettings";
import { loadRibbonStyle, saveRibbonStyle, type RibbonStyle } from "../lib/ribbonStyle";
import { loadRibbonVisibility, saveRibbonVisibility, type RibbonVisibility } from "../lib/ribbonVisibility";

import { authorshipColors, setAuthorshipColors } from "../lib/attribution";

const CHANGED = "hickory:ribbon-preferences";
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

export function LineageSettings() {
  return <FeatureSettings label="Lineage settings"><LineageOptions /></FeatureSettings>;
}
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
