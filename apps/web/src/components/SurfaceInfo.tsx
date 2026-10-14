import { PaneHeaderContent } from "../shell/PaneHeader";
import "./SurfaceInfo.css";

/** The same three questions for every working surface: identity, editability,
 * and where a write lands. Presentation never grants a capability. */
export function SurfaceInfo({ name, editable, destination }: {
  name: string; editable: boolean; destination: string;
}) {
  return <PaneHeaderContent part="info"><details className="surface-info" aria-label="About this view">
    <summary role="button" aria-label={`About ${name} · ${editable ? "Editable" : "Read-only"}`} data-tip={`${name} · ${editable ? "Editable" : "Read-only"}. ${destination}`}>
      <strong>{name}</strong><span>{editable ? "Editable" : "Read-only"}</span>
    </summary>
    <p><strong>{name} · {editable ? "Editable" : "Read-only"}</strong><br />{destination}</p>
  </details></PaneHeaderContent>;
}
