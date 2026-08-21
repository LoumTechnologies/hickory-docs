// Whether the blame column is showing.
//
// A display preference of this machine, like the theme and the zoom: it
// follows the reader's current job — reviewing, bisecting, working out
// whether a passage came from the agent — not the project. So localStorage,
// beside the other appearance preferences.
//
// Off by default, deliberately. A blame column is a permanent indent on every
// line of every file, answering a question nobody asks most of the time.

export const BLAME_KEY = "hickory.blameColumn";

/** The event a toggle fires so every open editor turns together. Two panes
 * disagreeing about whether the column is up would look like a bug. */
export const BLAME_CHANGED_EVENT = "hickory:blame-changed";

export function loadBlameShown(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined"
    ? null
    : localStorage,
): boolean {
  return storage?.getItem(BLAME_KEY) === "1";
}

export function saveBlameShown(
  shown: boolean,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined"
    ? null
    : localStorage,
): void {
  storage?.setItem(BLAME_KEY, shown ? "1" : "0");
}

/** Flip it, persist it, and tell every editor. Returns the new value. */
export function toggleBlameShown(): boolean {
  const next = !loadBlameShown();
  saveBlameShown(next);
  window.dispatchEvent(new CustomEvent(BLAME_CHANGED_EVENT, { detail: next }));
  return next;
}

/** Hear about a change. Returns the unsubscribe. */
export function onBlameChanged(handler: (shown: boolean) => void): () => void {
  const listener = (event: Event) => {
    const shown = (event as CustomEvent).detail;
    if (typeof shown === "boolean") handler(shown);
  };
  window.addEventListener(BLAME_CHANGED_EVENT, listener);
  return () => window.removeEventListener(BLAME_CHANGED_EVENT, listener);
}
