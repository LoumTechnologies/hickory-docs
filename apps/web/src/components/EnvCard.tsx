import type { ExecutorInfo } from "../api/types";

export interface EnvCardProps {
  /** Container name (`hick:container name=`). */
  name: string;
  /** Image ref (`image=`), e.g. "alpine:3.20". */
  image?: string;
  /** Summarised hick:allow/hick:deny children, e.g. "network: github.com:443". */
  rules: string[];
  /** GET /api/executor result; null while loading / unavailable. */
  executor: ExecutorInfo | null;
}

/**
 * The environment card rendered above a `hick:container` declaration in the
 * Document view: what the container is (name + image ref + access rules) and
 * where its commands actually run (the /api/executor resolution line).
 */
export function EnvCard({ name, image, rules, executor }: EnvCardProps) {
  let resolution: { text: string; warn: boolean } | null = null;
  if (executor?.kind === "local") {
    resolution = {
      text: "local executor — commands run on the host; image is recorded, not enforced",
      warn: false,
    };
  } else if (executor?.kind === "canopy") {
    const mapped = image ? executor.images?.[image] : undefined;
    resolution = mapped
      ? { text: `canopy — runs in ${mapped}`, warn: false }
      : {
          text: image
            ? `canopy — image "${image}" is not mapped to a sandbox image`
            : "canopy — no image declared",
          warn: true,
        };
  }

  return (
    <div className="env-card" data-testid={`env-card-${name}`}>
      <div className="env-card-bar">
        <span className="env-card-label">environment</span>
        <span className="env-card-name">{name}</span>
        {image && <span className="env-card-image">{image}</span>}
        {rules.length > 0 && <span className="env-card-rules">{rules.join(" · ")}</span>}
      </div>
      {resolution && (
        <div className={`env-card-resolution${resolution.warn ? " warn" : ""}`}>
          {resolution.text}
        </div>
      )}
    </div>
  );
}
