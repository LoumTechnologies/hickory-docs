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

import { useEffect, useState } from "react";

export interface PicturePanelProps {
  /** Root-relative path of the file, already resolved against the document. */
  src: string | null;
  /** The path as the document spells it — what a message about it should say. */
  path: string;
}

/** A file whose bytes are the never-run marker rather than a picture. */
const NEVER_RUN = "[never run]";

export function PicturePanel({ src, path }: PicturePanelProps) {
  const [status, setStatus] = useState<"loading" | "ok" | "missing" | "never-run">("loading");

  // A never-run marker is a 14-byte text file with an image's name: the
  // browser draws a broken icon and says nothing useful. Reading the bytes
  // first is what lets the panel say which of the two things went wrong —
  // the cell has no recording, or the file is not there at all.
  useEffect(() => {
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
        return head.includes(NEVER_RUN) ? ("never-run" as const) : ("ok" as const);
      })
      .catch(() => "missing" as const)
      .then((next) => {
        if (live) setStatus(next);
      });
    return () => {
      live = false;
    };
  }, [src]);

  if (status === "never-run") {
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
      <img src={src} alt={path} />
    </div>
  );
}
