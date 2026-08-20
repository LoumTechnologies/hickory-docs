// The tree's right-click menu: the items treeMenu.ts computed, at the pointer.
//
// Deliberately small. It closes on Escape, on a click anywhere else, and on a
// scroll or resize that would leave it floating over the wrong row; it never
// traps focus, because a menu you cannot escape is worse than one you have to
// aim at twice.

import { useEffect, useRef } from "react";

import type { TreeMenuItem } from "./treeMenu";

export interface TreeContextMenuProps {
  /** Viewport coordinates of the click that opened it. */
  x: number;
  y: number;
  /** What this row offers. */
  items: readonly TreeMenuItem[];
  /** The row it belongs to, for the heading that says what will be acted on. */
  subject: string;
  onPick: (item: TreeMenuItem) => void;
  onClose: () => void;
}

export function TreeContextMenu({ x, y, items, subject, onPick, onClose }: TreeContextMenuProps) {
  const box = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    // The first item takes focus so the whole menu is reachable by keyboard
    // from the Windows/Linux context-menu key, which opens it with no pointer
    // involved at all.
    box.current?.querySelector("button")?.focus();
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

  // Kept inside the window: a right-click near the bottom edge otherwise
  // opens a menu whose last item is unreachable.
  const height = items.length * 26 + 34;
  const width = 220;
  const left = Math.max(4, Math.min(x, window.innerWidth - width - 4));
  const top = Math.max(4, Math.min(y, window.innerHeight - height - 4));

  return (
    <div
      ref={box}
      className="tree-menu"
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
          className={`tree-menu__item${item.group ? " tree-menu__item--group" : ""}`}
          onClick={() => onPick(item)}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}
