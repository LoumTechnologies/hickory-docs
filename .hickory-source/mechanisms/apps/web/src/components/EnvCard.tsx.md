<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/EnvCard.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f5a0c599fb99"></a>
## `apps/web/src/components/EnvCard.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f5a0c599fb99-EnvCardProps"></a>
### `EnvCardProps`

<storycode:copy id="f-f5a0c599fb99-EnvCardProps">
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

</storycode:copy>
<a id="f-f5a0c599fb99-EnvCard"></a>
### `EnvCard`

The executor-resolution note rendered INLINE at the end of a
`hick:container` declaration line: where this environment's commands
actually run. Everything else about the environment — name, image, access
rules — is the declaration's own source text, already on numbered lines
directly here; repeating it in chrome would only add rows the gutter
cannot number. The summary survives as a hover title.

<storycode:copy id="f-f5a0c599fb99-EnvCard">
/**
 * The executor-resolution note rendered INLINE at the end of a
 * `hick:container` declaration line: where this environment's commands
 * actually run. Everything else about the environment — name, image, access
 * rules — is the declaration's own source text, already on numbered lines
 * directly here; repeating it in chrome would only add rows the gutter
 * cannot number. The summary survives as a hover title.
 */
export function EnvCard({ name, image, rules, executor }: EnvCardProps) {
  let resolution: { text: string; warn: boolean } | null = null;
  if (executor?.kind === "local") {
    resolution = {
      text: "runs on the host — image recorded, not enforced",
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
  if (!resolution) return null;

  const summary = [name, image, ...rules].filter(Boolean).join(" · ");
  return (
    <span
      className={`env-resolution${resolution.warn ? " warn" : ""}`}
      data-testid={`env-card-${name}`}
      data-tip={summary}
    >
      {resolution.text}
    </span>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/EnvCard.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/EnvCard.tsx"><storycode:paste select="#f-f5a0c599fb99-EnvCardProps"/><storycode:paste select="#f-f5a0c599fb99-EnvCard"/></storycode:file>

</storycode:doc>
