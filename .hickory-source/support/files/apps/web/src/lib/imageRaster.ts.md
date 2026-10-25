<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/imageRaster.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `79e7ccfb82c06bef310d5bcf01f0db1525732a96f10312dde185cece47da1528`.

<book:entry path="apps/web/src/lib/imageRaster.ts">
/** Capture the actual browser-loaded pixels. Producer/source equality and a
 * successful program execution are separate from this bounded visual reading. */
export async function imageRaster(element: HTMLImageElement, maxDimension: number, signal: AbortSignal, current: () => boolean) {
  const valid = () => !signal.aborted && element.isConnected && current();
  if (!valid() || !element.complete || !element.naturalWidth) return { ok: false as const, reason: "Displayed image is not currently available" };
  const scale = Math.min(1, maxDimension / Math.max(element.naturalWidth, element.naturalHeight));
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(element.naturalWidth * scale)); canvas.height = Math.max(1, Math.round(element.naturalHeight * scale));
  const context = canvas.getContext("2d");
  if (!context) throw Error("This browser has no image canvas context");
  context.drawImage(element, 0, 0, canvas.width, canvas.height);
  const blob = await new Promise<Blob>((resolve, reject) => canvas.toBlob(value => value ? resolve(value) : reject(Error("PNG encoding unavailable")), "image/png"));
  if (blob.size > 393216) return { ok: false as const, reason: "PNG exceeds 384 KiB; read again with a smaller max_dimension" };
  const bytes = new Uint8Array(await blob.arrayBuffer());
  const digest = [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))].map(byte => byte.toString(16).padStart(2, "0")).join("");
  if (!valid()) return { ok: false as const, reason: "Displayed image changed or closed while capturing; read the current picture again" };
  let binary = ""; for (const byte of bytes) binary += String.fromCharCode(byte);
  return { ok: true as const, capture: { original_width: element.naturalWidth, original_height: element.naturalHeight, width: canvas.width, height: canvas.height,
    ui_image: { media_type: "image/png", data: btoa(binary), digest, byte_length: bytes.length } } };
}
</book:entry>

</book:doc>
