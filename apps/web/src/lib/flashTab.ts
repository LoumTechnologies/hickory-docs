// "It's already open — HERE." Opening a file that is already open activates
// its tab, but activation alone is invisible when the tab was already active
// or sits in a busy strip: the flash is what moves the eye to it. One event,
// answered by the shell, works for every place a tab renders — the top
// strip, the side tab tree, and a collapsed pane's icon strip — because they
// all carry the same data attributes the ribbon overlay measures by.

export const FLASH_TAB_EVENT = "hickory:flash-tab";
export const FLASH_CLASS = "shell-tab--flash";

/** Ask the shell to pulse the tab showing `target`. */
export function flashTab(kind: "document" | "generated" | "file" | "tree", target: string): void {
  window.dispatchEvent(new CustomEvent(FLASH_TAB_EVENT, { detail: { kind, target } }));
}

/** A value quoted into an attribute selector. Paths may hold anything. */
function attr(value: string): string {
  return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

/**
 * Listen for flash requests and pulse the matching tab under `root`.
 * The lookup runs a frame later on purpose: the request is usually
 * dispatched in the same tick as the activation that re-renders (and may
 * re-parent) the tab element, and a class added to the old element would
 * vanish with it.
 */
export function attachFlashListener(root: HTMLElement): () => void {
  const frames = new Set<number>();
  const onFlash = (e: Event) => {
    const detail = (e as CustomEvent<{ kind?: string; target?: string }>).detail;
    if (!detail?.kind || typeof detail.target !== "string") return;
    // `let` + assign, not `const`: a synchronous rAF (tests) runs the
    // callback during the call, and a `const` frame would still be in its
    // temporal dead zone there.
    let frame = 0;
    frame = requestAnimationFrame(() => {
      frames.delete(frame);
      const el = root.querySelector(
        `[data-shell-tab-kind=${attr(detail.kind!)}][data-shell-tab-target=${attr(detail.target!)}]`,
      );
      if (!(el instanceof HTMLElement)) return;
      // Restarting an in-flight flash needs the class re-applied from
      // scratch, or the animation never re-fires.
      el.classList.remove(FLASH_CLASS);
      void el.offsetWidth;
      el.classList.add(FLASH_CLASS);
      el.addEventListener("animationend", () => el.classList.remove(FLASH_CLASS), {
        once: true,
      });
    });
    frames.add(frame);
  };
  window.addEventListener(FLASH_TAB_EVENT, onFlash);
  return () => {
    window.removeEventListener(FLASH_TAB_EVENT, onFlash);
    for (const frame of frames) cancelAnimationFrame(frame);
  };
}
