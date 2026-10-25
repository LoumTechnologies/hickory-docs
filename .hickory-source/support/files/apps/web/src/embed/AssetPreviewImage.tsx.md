<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/AssetPreviewImage.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `7e555f1617fa84ec98326a76b5c629a5796450f4f44621ec57531e85cfeaef14`.

<book:entry path="apps/web/src/embed/AssetPreviewImage.tsx">
import { useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { imageRaster } from "../lib/imageRaster";
/** A host-supplied preview stays read-only; pixel disclosure is explicit. */
export function AssetPreviewImage({ path, url }: { path: string; url: string }) {
  const parent = `embedded-asset:${useId()}`;
  const image = useRef<HTMLImageElement | null>(null), currentUrl = useRef(JSON.stringify([path, url])), version = useRef(0);
  currentUrl.current = JSON.stringify([path, url]);
  const [status, setStatus] = useState<"loading" | "loaded" | "missing">("loading");
  const [error, setError] = useState("");
  const revision = useUiRevision(() => [path, url, status, version.current, image.current?.currentSrc, image.current?.naturalWidth, image.current?.naturalHeight, error]);
  const ready = () => status === "loaded" && !!image.current?.isConnected && image.current.complete && image.current.naturalWidth > 0 && image.current.currentSrc === new URL(url, location.href).href;
  useUiCommands([{ key: parent, command: {
    label: `Read embedded image ${path.slice(0, 200)}`, parent, subject: parent, parameters: { max_dimension: { type: "number", minimum: 256, maximum: 2048, integer: true } }, prerequisites: ["Current loaded host image; explicit scaled PNG capture, subject to browser CORS and secure-context restrictions"],
    read: () => ({ revision: revision(), enabled: ready(), value: { path, status, width: image.current?.naturalWidth ?? null, height: image.current?.naturalHeight ?? null, capture_error: error }, reason: "Wait for the current host image to load" }),
    execute: async (args, _actor, signal) => {
      const element = image.current, captured = version.current;
      if (!element || !ready()) return { status: "unavailable", reason: "The host image is not currently loaded" };
      const current = () => image.current === element && currentUrl.current === JSON.stringify([path, url]) && version.current === captured && element.isConnected;
      try {
        const result = await imageRaster(element, args.max_dimension as number, signal, current);
        if (!result.ok) return { status: "unavailable", reason: result.reason };
        setError("");
        return { status: "completed", value: { path, ...result.capture, scope: "Actual displayed host pixels, possibly scaled; not original bytes or execution evidence" } };
      } catch (cause) {
        if (!current() || signal.aborted) return { status: "unavailable", reason: "The host image owner changed or capture was cancelled" };
        const reason = cause instanceof Error ? cause.message : String(cause); setError(reason);
        return { status: "requires_input", request: `${parent}:capture-access`, reason: `Image capture unavailable: ${reason}. Supply a same-origin or host-created blob/data image with browser canvas access in a secure context, then retry.` };
      }
    },
  } }]);
  useUiReading("embedded image status", { path, status, error, width: image.current?.naturalWidth ?? null, height: image.current?.naturalHeight ?? null, pixels: "Use the explicit image reading action" });
  return <span><img ref={image} src={url} alt={path} onLoad={() => { version.current++; setStatus("loaded"); }} onError={() => { version.current++; setStatus("missing"); }} />{status === "missing" && <span role="alert">{path}: host image failed to load</span>}{error && <span role="alert">{error}</span>}</span>;
}
</book:entry>

</book:doc>
