import "./SurfaceInfo.css";

/** The same three questions for every working surface: identity, editability,
 * and where a write lands. Presentation never grants a capability. */
export function SurfaceInfo({ name, editable, destination }: {
  name: string; editable: boolean; destination: string;
}) {
  return <details className="surface-info" aria-label="About this view">
    <summary data-tip={`${name} · ${editable ? "Editable" : "Read-only"}. ${destination}`}>
      <strong>{name}</strong><span>{editable ? "Editable" : "Read-only"}</span>
    </summary>
    <p>{destination}</p>
  </details>;
}
