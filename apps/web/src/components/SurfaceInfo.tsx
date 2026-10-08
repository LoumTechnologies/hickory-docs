/** The same three questions for every working surface: identity, editability,
 * and where a write lands. Presentation never grants a capability. */
export function SurfaceInfo({ name, editable, destination }: {
  name: string; editable: boolean; destination: string;
}) {
  return <div className="surface-info" aria-label="About this view">
    <strong>{name}</strong><span>{editable ? "Editable" : "Read-only"}</span>
    <span>{destination}</span>
  </div>;
}
