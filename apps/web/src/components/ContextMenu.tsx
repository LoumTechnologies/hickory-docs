// One right-click menu, for everything in the app that has one.
//
// Extracted from the folder tree's menu (shell/TreeContextMenu.tsx), which is
// now a caller rather than the only implementation. The behaviour is the part
// worth having exactly once: it closes on Escape, on a click anywhere else,
// and on a scroll or resize that would leave it floating over the wrong
// thing; it takes focus on the first item so the Windows/Linux context-menu
// key reaches the whole menu with no pointer involved; and it keeps itself
// inside the window, because a menu opened near the bottom edge otherwise has
// a last item nobody can click.
//
// It never traps focus. A menu you cannot escape is worse than one you have
// to aim at twice.

import { useEffect, useRef } from "react";

export interface ContextMenuItem {
  /** Stable across renders and platforms; what a test clicks by. */
  id: string;
  label: string;
  /** Offered but not available right now, with the tooltip saying why. An
   * item that vanishes teaches nobody where it went. */
  disabled?: boolean;
  /** Why it is disabled, or what it will do. */
  tip?: string;
  /** A separator line is drawn above this item. */
  group?: boolean;
  /** Removes something. Drawn so the hand slows down before it lands. */
  danger?: boolean;
}

export interface ContextMenuProps {
  /** Viewport coordinates of the click that opened it. */
  x: number;
  y: number;
  items: readonly ContextMenuItem[];
  /** What is being acted on, as a heading. */
  subject: string;
  onPick: (id: string) => void;
  onClose: () => void;
  /** Extra class on the box, for a caller with its own look. */
  className?: string;
}

/** Roughly how tall the menu will be, for keeping it on screen. Measured
 * against the CSS in styles.css (`.tree-menu__item`); a few pixels out only
 * moves the box a few pixels, which is why an estimate is enough. */
export function menuBox(
  items: readonly ContextMenuItem[],
  x: number,
  y: number,
  viewport: { width: number; height: number },
): { left: number; top: number; width: number } {
  const width = 220;
  const height = items.length * 26 + items.filter((i) => i.group).length * 5 + 34;
  return {
    left: Math.max(4, Math.min(x, viewport.width - width - 4)),
    top: Math.max(4, Math.min(y, viewport.height - height - 4)),
    width,
  };
}

export function ContextMenu({
  x,
  y,
  items,
  subject,
  onPick,
  onClose,
  className,
}: ContextMenuProps) {
  const box = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    box.current?.querySelector<HTMLButtonElement>("button:not([disabled])")?.focus();
  }, []);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        onClose();
      }
    };
    const onPointer = (event: MouseEvent) => {
      if (!box.current?.contains(event.target as Node)) onClose();
    };
    window.addEventListener("keydown", onKey, true);
    window.addEventListener("mousedown", onPointer, true);
    window.addEventListener("contextmenu", onPointer, true);
    window.addEventListener("resize", onClose);
    window.addEventListener("scroll", onClose, true);
    return () => {
      window.removeEventListener("keydown", onKey, true);
      window.removeEventListener("mousedown", onPointer, true);
      window.removeEventListener("contextmenu", onPointer, true);
      window.removeEventListener("resize", onClose);
      window.removeEventListener("scroll", onClose, true);
    };
  }, [onClose]);

  const { left, top, width } = menuBox(items, x, y, {
    width: window.innerWidth,
    height: window.innerHeight,
  });

  return (
    <div
      ref={box}
      className={`tree-menu${className ? ` ${className}` : ""}`}
      role="menu"
      aria-label={`Actions for ${subject}`}
      style={{ left: `${left}px`, top: `${top}px`, width: `${width}px` }}
    >
      <p className="tree-menu__title mono" data-tip={subject}>
        {subject}
      </p>
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          role="menuitem"
          data-menu-item={item.id}
          disabled={item.disabled}
          data-tip={item.tip}
          className={`tree-menu__item${item.group ? " tree-menu__item--group" : ""}${
            item.danger ? " tree-menu__item--danger" : ""
          }`}
          // The grid takes its selection on mousedown; a press on a menu item
          // must not move it out from under the action about to run.
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => onPick(item.id)}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}
