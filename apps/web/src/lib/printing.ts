// Printing, which is harder than `window.print()` for one specific reason.
//
// CodeMirror virtualises: only the lines near the viewport are in the DOM at
// all. Printing the window therefore prints the screenful someone happens to
// be looking at, silently, with no indication that the other four hundred
// lines were dropped — which is a worse failure than refusing to print, since
// the paper looks fine.
//
// So printing does not use the editor's DOM. The full text is written into a
// dedicated element that only exists during the print, and `@media print` in
// styles.css hides everything else on the page. What comes out is what the
// buffer holds, all of it.
//
// The document TITLE matters more than it looks: every browser and webview
// puts it in the page header and uses it as the default filename for
// "Save as PDF". Leaving it as the app's own title would name every printed
// note "Hickory Docs".

/** Where the printable copy is written. Created on demand, removed after. */
const HOST_ID = "hickory-print-sheet";

export interface PrintJob {
  /** Names the page header, and the PDF a reader saves. */
  title: string;
  /** The whole buffer. Not a viewport, not a selection. */
  text: string;
}

/**
 * Print `job`, then put the page back exactly as it was.
 *
 * `print()` is synchronous in every engine we ship on — it blocks until the
 * dialog is dismissed — so the cleanup below runs after the dialog closes.
 * The `finally` is not decoration: an engine that throws here (a headless
 * build, a webview with printing disabled) must not leave the app wearing a
 * printout's title and a hidden element full of somebody's notes.
 */
export function printText(job: PrintJob, win: Window = window): void {
  const doc = win.document;
  const previousTitle = doc.title;
  const host = doc.createElement("div");
  host.id = HOST_ID;
  host.className = "print-sheet";

  const heading = doc.createElement("h1");
  heading.className = "print-sheet__title";
  heading.textContent = job.title;
  const body = doc.createElement("pre");
  body.className = "print-sheet__body";
  // textContent, never innerHTML: this is somebody's file, and a note
  // containing `<script>` is a note about scripts.
  body.textContent = job.text;
  host.append(heading, body);
  doc.body.appendChild(host);
  win.document.title = job.title;

  try {
    win.print();
  } finally {
    host.remove();
    win.document.title = previousTitle;
  }
}

/** The last path segment, which is what a reader calls the file. */
export function printTitleFor(path: string): string {
  const name = path.split("/").filter(Boolean).pop();
  return name && name.length > 0 ? name : path;
}
