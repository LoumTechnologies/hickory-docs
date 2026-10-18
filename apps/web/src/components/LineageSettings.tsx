import { useEffect, useState } from "react";
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
  const { style, visibility } = useRibbonPresentation();
  const [colors, setColors] = useState(authorshipColors);
  return <FeatureSettings label="Lineage settings">
    <label><input type="checkbox" checked={colors} onChange={event => { setColors(event.target.checked); setAuthorshipColors(event.target.checked); }} /> Color credited authors</label>
    <p className="muted">Known human input uses teal; recorded agent text uses purple and a dashed stripe. Other colors distinguish credited authors with unknown origin; imported text uses dots. Git identity alone does not establish human or agent origin. Changed text loses its highlight until saved attribution refreshes.</p>
    <label>Draw connections as<select aria-label="Lineage style" value={style} onChange={event => {
      saveRibbonStyle(event.target.value as RibbonStyle); window.dispatchEvent(new Event(CHANGED));
    }}><option value="braces">Braces</option><option value="bands">Ribbons</option></select></label>
    <label>Document connections<select aria-label="Document lineage visibility" value={visibility} onChange={event => {
      saveRibbonVisibility(event.target.value as RibbonVisibility); window.dispatchEvent(new Event(CHANGED));
    }}><option value="caret">At the caret or on hover</option><option value="always">All connections</option></select></label>
    <p className="muted">Conversation connections follow expanded items. Change collapsed-item visibility in Agent settings.</p>
  </FeatureSettings>;
}
