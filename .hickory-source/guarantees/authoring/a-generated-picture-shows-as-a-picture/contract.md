<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Generated Picture Shows As A Picture

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:file>` block whose `path` names something a browser can draw
(`.svg`, `.png`, `.jpg`/`.jpeg`, `.gif`, `.webp`, `.avif`), when the document
is opened in the app, then the block shows **the picture the file holds**, and
a `▨` icon on the action rail toggles it back to the source that draws it —
the plotting code, the cell, and the tags around them.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-generated-picture-shows-as-a-picture.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-d39df8cacf2f"></a>
## `apps/web/src/components/PicturePanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Generated Picture Shows As A Picture](contract.md).

<a id="f-d39df8cacf2f-PicturePanelProps"></a>
### `PicturePanelProps`

<storycode:copy id="f-d39df8cacf2f-PicturePanelProps">
import { imageRaster } from "../lib/imageRaster";
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

</storycode:copy>
<a id="f-d39df8cacf2f-NEVER_RUN"></a>
### `NEVER_RUN`

A file whose bytes are the never-run marker rather than a picture.

<storycode:copy id="f-d39df8cacf2f-NEVER_RUN">
/** A file whose bytes are the never-run marker rather than a picture. */
const NEVER_RUN = "[never run]";

</storycode:copy>
<a id="f-d39df8cacf2f-PicturePanel"></a>
### `PicturePanel`

<storycode:copy id="f-d39df8cacf2f-PicturePanel">
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
        const result = await imageRaster(element, args.max_dimension as number, signal, () => image.current === element && currentSource.current === src && version.current === capturedVersion);
        if (!result.ok) return { status: "unavailable", reason: result.reason };
        setCaptureError(null);
        return { status: "completed", value: { path, resource, ...result.capture, scope: "Actual captured raster pixels, potentially downscaled; not source-file equality or a verification verdict" } };
      } catch (error) {
        if (signal.aborted || image.current !== element || currentSource.current !== src || version.current !== capturedVersion || !element.isConnected) return { status: "unavailable", reason: "Displayed image changed or closed while capturing" };
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
</storycode:copy>

### Assemble `apps/web/src/components/PicturePanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/PicturePanel.tsx"><storycode:paste select="#f-d39df8cacf2f-PicturePanelProps"/><storycode:paste select="#f-d39df8cacf2f-NEVER_RUN"/><storycode:paste select="#f-d39df8cacf2f-PicturePanel"/></storycode:file>

## Test the behavior: `hick_doc`

This is the maintained test fragment from `crates/hick-literate/tests/picture_weave.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e26fcf8c737f-hick_doc"></a>
### `hick_doc`

A file block that writes a picture weaves as the picture.
Found by dogfooding: showing the block as its chart in the editor made the
document show each chart twice — once for the block, once for the
hand-written `!…` beneath it. That line existed because the
weave had no other way to put a picture in the markdown, and fencing an
SVG puts forty kilobytes of markup in the middle of a document.

<storycode:copy id="f-e26fcf8c737f-hick_doc">
//! A file block that writes a picture weaves as the picture.
//!
//! Guarantee: docs/guarantees/authoring/a-generated-picture-shows-as-a-picture.md
//!
//! Found by dogfooding: showing the block as its chart in the editor made the
//! document show each chart twice — once for the block, once for the
//! hand-written `![…](chart.svg)` beneath it. That line existed because the
//! weave had no other way to put a picture in the markdown, and fencing an
//! SVG puts forty kilobytes of markup in the middle of a document.

fn hick_doc(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
{body}
</hick:doc>"#
    )
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/PicturePanel.tsx](contract.md#output-d39df8cacf2f)
- [apps/web/src/editor/CardRail.tsx](../../../mechanisms/apps/web/src/editor/CardRail.tsx.md#output-f6019c4774f8)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/cards.ts](../../../mechanisms/apps/web/src/editor/cards.ts.md#output-84385185ee44)
- [apps/web/src/editor/hickDoc.ts](../../../mechanisms/apps/web/src/editor/hickDoc.ts.md#output-c2f7bc821e36)
- [apps/web/src/editor/rendered.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-4f5444d21411)
- [apps/web/src/lib/railActions.ts](../../../mechanisms/apps/web/src/lib/railActions.ts.md#output-499155452cf6)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)

### Tests and fixtures

- [apps/web/src/editor/cards.test.ts](../../../mechanisms/apps/web/src/editor/cards.ts.md#output-66013e5cdf76)
- [apps/web/src/editor/rendered.test.ts](../../../mechanisms/apps/web/src/editor/rendered.ts.md#output-1e1e27143ad4)
- [apps/web/src/lib/railActions.test.ts](../../../mechanisms/apps/web/src/lib/railActions.ts.md#output-49410ea38e0d)
- [crates/hick-literate/tests/picture_weave.rs](../../../mechanisms/crates/hick-literate/tests/picture_weave.rs.md#output-e26fcf8c737f)
- [crates/hickory-cli/tests/weave_keeps_artifacts.rs](../../../mechanisms/crates/hickory-cli/tests/weave_keeps_artifacts.rs.md#output-b0471d375879)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
