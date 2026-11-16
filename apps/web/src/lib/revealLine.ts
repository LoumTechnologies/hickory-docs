// "Open that file, at that line."
//
// The awkward part is timing: the request is made before the pane that will
// answer it exists. A find hit opens a file, and the editor for it is built a
// tick or two later — so an event alone would be shouted into an empty room.
//
// So the request is REMEMBERED, keyed by path, and a pane claims it when it
// mounts as well as when the event arrives. A claim consumes it, which means
// opening the same file again later does not silently jump to a line somebody
// asked about two minutes ago.

export const REVEAL_LINE_EVENT = "hickory:reveal-line";

/** Outstanding requests, by path. At most one per file: the newest wins,
 * because it is the one somebody just clicked. */
const pending = new Map<string, { line: number; character?: number;guard?:{source:string;current:()=>boolean} }>();

/** Ask for `path` to be shown with the caret on `line` (1-based). */
export function revealLine(path: string, line: number, character?: number,guard?:{source:string;current:()=>boolean}): void {
  if(guard&&(!guard.current()||!Number.isInteger(line)||line<1||character!==undefined&&(!Number.isInteger(character)||character<0)))return;
  pending.set(path, { line: Math.max(1, Math.floor(line)), ...(character === undefined ? {} : { character: Math.max(0, Math.floor(character)) }),...(guard?{guard}:{}) });
  window.dispatchEvent(new CustomEvent(REVEAL_LINE_EVENT, { detail: path }));
}

/**
 * Take the request for `path`, if there is one.
 *
 * Consuming rather than peeking: a pane that scrolls to the line has answered
 * the question, and re-answering it on the next re-render would fight the
 * reader for the cursor.
 */
export function claimReveal(path: string): number | null {
  const line = pending.get(path);
  if (line === undefined) return null;
  pending.delete(path);
  if(line.guard)return null;
  return line.line;
}
/** Consume the same one-shot request while retaining its optional UTF-16 column. */
export function claimRevealPosition(path: string,source?:string): { line: number; character?: number } | null {
  const location = pending.get(path);
  if (!location) return null;
  pending.delete(path);if(location.guard&&(source!==location.guard.source||!location.guard.current()))return null;return {line:location.line,...(location.character===undefined?{}:{character:location.character})};
}

/** Listen for requests. The handler is called with the path asked for; it
 * still has to `claimReveal` to take it. Returns the unsubscribe. */
export function onRevealLine(handler: (path: string) => void): () => void {
  const listener = (event: Event) => {
    const path = (event as CustomEvent).detail;
    if (typeof path === "string") handler(path);
  };
  window.addEventListener(REVEAL_LINE_EVENT, listener);
  return () => window.removeEventListener(REVEAL_LINE_EVENT, listener);
}

// "Open that file, at that line" — from somewhere that cannot open a tab.
//
// A definition can land in another file, and the editor that asked has no
// way to open one: tabs belong to the workspace. So the request is announced,
// the workspace opens (or raises) the tab, and the reveal above does the
// rest. Paths are workspace-relative, as everywhere else; `line` is 1-based.

export const OPEN_LOCATION_EVENT = "hickory:open-location";

export interface OpenLocation {
  path: string;
  line: number;
  character?: number;
  recordHistory?: boolean;
  expectedSource?: string;
  capture?:{hash:string|null;current:()=>boolean;sourceReference?:number|null;positionKnown?:boolean};
}

/** Ask the workspace to open `path` and put the caret on `line`. */
export function diagramFilePath(href: string, source: string): string | null {
  let path: string;
  try { path = decodeURIComponent(href); } catch { return null; }
  if (!path || /[:\\\r\n]/.test(path) || path.startsWith("/")) return null;
  const parts = source.split("/").slice(0, -1);
  for (const part of path.split("/")) {
    if (part === "..") { if (!parts.length) return null; parts.pop(); }
    else if (part && part !== ".") parts.push(part);
  }
  return parts.length ? parts.join("/") : null;
}
export function openLocation(path: string, line: number, character?: number, recordHistory = true, expectedSource?: string): void {
  window.dispatchEvent(
    new CustomEvent<OpenLocation>(OPEN_LOCATION_EVENT, { detail: { path, line, ...(character === undefined ? {} : { character }), ...(recordHistory ? {} : { recordHistory: false }), ...(expectedSource === undefined ? {} : { expectedSource }) } }),
  );
}

/** Unknown captured source identities open text without asserting any line position. */
export function openCapturedLocation(path:string,line:number|null,hash:string|null,current:()=>boolean,sourceReference?:number|null){
 const positionKnown=line!==null&&Number.isInteger(line)&&line>=1;
 window.dispatchEvent(new CustomEvent<OpenLocation>(OPEN_LOCATION_EVENT,{detail:{path,line:positionKnown?line!:1,character:0,recordHistory:false,capture:{hash:positionKnown&&!(sourceReference&&sourceReference>0)&&hash&&/^[0-9a-f]{64}$/.test(hash)?hash:null,current,positionKnown,...(sourceReference===undefined?{}:{sourceReference})}}}));
}

/** The workspace's side. Returns the unsubscribe. */
export function onOpenLocation(handler: (location: OpenLocation) => void,supportsCaptured=false): () => void {
  const listener = (event: Event) => {
    const detail = (event as CustomEvent<OpenLocation>).detail;
    if (detail && typeof detail.path === "string"&&(!detail.capture||supportsCaptured)) handler(detail);
  };
  window.addEventListener(OPEN_LOCATION_EVENT, listener);
  return () => window.removeEventListener(OPEN_LOCATION_EVENT, listener);
}

/** A `hick:///<path>` URI's path, or null for any other scheme. */
export function pathOfDocUri(uri: string): string | null {
  if (!uri.startsWith("hick:///")) return null;
  try { return decodeURIComponent(new URL(uri).pathname.replace(/^\//,"")); } catch { return null; }
}

// "Show that terminal" — from a pane that started a session and cannot open
// a tab. A test run, say.

export const SHOW_TERMINAL_EVENT = "hickory:show-terminal";

export function showTerminalRequest(id: string, title: string): void {
  window.dispatchEvent(new CustomEvent(SHOW_TERMINAL_EVENT, { detail: { id, title } }));
}

export function onShowTerminal(handler: (session: { id: string; title: string }) => void): () => void {
  const listener = (event: Event) => {
    const detail = (event as CustomEvent<{ id: string; title: string }>).detail;
    if (detail && typeof detail.id === "string") handler(detail);
  };
  window.addEventListener(SHOW_TERMINAL_EVENT, listener);
  return () => window.removeEventListener(SHOW_TERMINAL_EVENT, listener);
}

/** Test seam: forget every outstanding request. */
export function resetReveals(): void {
  pending.clear();
}
