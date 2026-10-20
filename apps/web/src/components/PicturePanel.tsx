import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
// A `<hick:file>` that writes a chart, shown as the chart.
//
// The block's source is forty lines of R or matplotlib; what it MEANS is one
// picture. So the file block renders as its result, the same way an exec cell
// and a diagram do, and the rail's ▨ icon is the way back to the code that
// drew it.
//
// The picture is loaded from the file on disk — the same `/api/asset` URL the
// markdown `![…](chart.svg)` beneath it uses — not from the cell's transcript.
// That is deliberate: this panel then shows exactly what a reader of the
// repository sees, so a file that is stale, missing, or holding `[never run]`
// LOOKS wrong here instead of looking fine while the disk disagrees. That
// disagreement is the bug this whole area exists to make visible.

import { useEffect, useId, useRef, useState } from "react";

export interface PicturePanelProps {
  /** Root-relative path of the file, already resolved against the document. */
  src: string | null;
  /** The path as the document spells it — what a message about it should say. */
  path: string;
}

/** A file whose bytes are the never-run marker rather than a picture. */
const NEVER_RUN = "[never run]";

export function PicturePanel({ src, path }: PicturePanelProps) {
  const image = useRef<HTMLImageElement | null>(null);
  const [epoch, setEpoch] = useState(0);
  const version = useRef(0);
  const currentSource = useRef(src); currentSource.current = src;
  const [captureError, setCaptureError] = useState<string | null>(null);
  const parent = `picture:${useId()}`;
  const [status, setStatus] = useState<"loading" | "ok" | "missing" | "unrecorded">("loading");

  // A never-run marker is a 14-byte text file with an image's name: the
  // browser draws a broken icon and says nothing useful. Reading the bytes
  // first is what lets the panel say which of the two things went wrong —
  // the cell has no recording, or the file is not there at all.
  useEffect(() => {
    version.current++; setCaptureError(null);
    if (!src) {
      setStatus("missing");
      return;
    }
    let live = true;
    setStatus("loading");
    fetch(src)
      .then(async (response) => {
        if (!response.ok) return "missing" as const;
        const head = (await response.text()).slice(0, 200);
        return head.includes(NEVER_RUN) ? ("unrecorded" as const) : ("ok" as const);
      })
      .catch(() => "missing" as const)
      .then((next) => {
        if (live) setStatus(next);
      });
    return () => {
      live = false;
    };
  }, [src]);

  const inline = src?.startsWith("data:");
  const resource = !src ? null : { kind: inline ? "inline_image" : "image_resource",
    media_type: inline ? src.slice(5, 133).split(/[;,]/)[0] : null,
    location: inline ? null : src.split(/[?#]/)[0].slice(0, 4096), query_present: !inline && src.includes("?") };
  const revision = useUiRevision(() => [src, path, status, epoch, version.current, image.current?.currentSrc, image.current?.naturalWidth, image.current?.naturalHeight, captureError]);
  const ready = () => !!src && status === "ok" && !!image.current?.isConnected && image.current.complete && image.current.currentSrc === new URL(src, window.location.href).href && image.current.naturalWidth > 0;
  useUiCommands([{ key: parent, command: { parent, subject: parent, label: `Read displayed image ${path.slice(0, 200)}`,
    parameters: { max_dimension: { type: "number", minimum: 256, maximum: 2048, integer: true } }, prerequisites: ["Loaded image in the current panel; capture is a scaled PNG of displayed pixels, not the original file bytes"],
    read: () => ({ revision: revision(), enabled: ready(), reason: ready() ? undefined : "Wait for a recorded, successfully loaded picture", value: { path, resource, status, natural_width: image.current?.naturalWidth ?? null, natural_height: image.current?.naturalHeight ?? null, capture_error: captureError } }),
    execute: async (args, _actor, signal) => {
      const element = image.current, capturedVersion = version.current;
      if (!ready() || !element || signal.aborted) return { status: "unavailable", reason: "Picture is not available for capture" };
      try {
        const scale = Math.min(1, (args.max_dimension as number) / Math.max(element.naturalWidth, element.naturalHeight));
        const canvas = document.createElement("canvas");
        canvas.width = Math.max(1, Math.round(element.naturalWidth * scale)); canvas.height = Math.max(1, Math.round(element.naturalHeight * scale));
        const context = canvas.getContext("2d");
        if (!context) throw new Error("This browser has no image canvas context");
        context.drawImage(element, 0, 0, canvas.width, canvas.height);
        const blob = await new Promise<Blob>((resolve, reject) => canvas.toBlob(value => value ? resolve(value) : reject(new Error("PNG encoding unavailable")), "image/png"));
        if (blob.size > 393216) return { status: "unavailable", reason: "PNG exceeds 384 KiB; read again with a smaller max_dimension" };
        const bytes = new Uint8Array(await blob.arrayBuffer());
        const digest = [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))].map(byte => byte.toString(16).padStart(2, "0")).join("");
        if (signal.aborted || image.current !== element || currentSource.current !== src || version.current !== capturedVersion || !element.isConnected) return { status: "unavailable", reason: "Displayed image changed or closed while capturing; read the current picture again" };
        let binary = ""; for (const byte of bytes) binary += String.fromCharCode(byte);
        setCaptureError(null);
        return { status: "completed", value: { path, resource, original_width: element.naturalWidth, original_height: element.naturalHeight, width: canvas.width, height: canvas.height, ui_image: { media_type: "image/png", data: btoa(binary), digest, byte_length: bytes.length }, scope: "Actual captured raster pixels, potentially downscaled; not source-file equality or a verification verdict" } };
      } catch (error) {
        const reason = error instanceof Error ? error.message : String(error); setCaptureError(reason);
        return { status: "requires_input", request: `${parent}:capture-access`, reason: `Image capture unavailable: ${reason}. Use an accessible same-origin image with browser CORS access in a secure browser context, then read the current image again.` };
      }
    },
  } }]);
  useUiReading("picture rendering status", { path, resource, status, capture_error: captureError, scope: "Image identity and load state only; Read displayed image sends captured pixels explicitly" });

  if (status === "unrecorded") {
    return (
      <div className="rendered-picture rendered-picture--empty">
        <strong>{path}</strong> has no recording yet — the cell that draws it has not run
        in this checkout. Run the cell (▶ on the rail) to produce it.
      </div>
    );
  }
  if (status === "missing" || !src) {
    return (
      <div className="rendered-picture rendered-picture--empty">
        <strong>{path}</strong> is not on disk yet. Run the cell (▶ on the rail) to write it.
      </div>
    );
  }
  return (
    <div className="rendered-picture">
      <img ref={image} src={src} alt={path} onLoad={() => { version.current++; setEpoch(value => value + 1); }} onError={() => { version.current++; setStatus("missing"); }} />
      {captureError && <p role="alert">Image capture request {parent}:capture-access · {captureError}. Use a same-origin image or enable its CORS access, then retry.</p>}
    </div>
  );
}
