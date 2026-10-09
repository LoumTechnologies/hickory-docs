/** Visible part of an element, clipped to ancestor scrollports (excluding scrollbars). */
export function visibleElementRect(element: HTMLElement): DOMRect | null {
  let { left, right, top, bottom } = element.getBoundingClientRect();
  for (let parent = element.parentElement; parent; parent = parent.parentElement) {
    const style = getComputedStyle(parent);
    const clips = (overflow: string) => /^(auto|scroll|hidden|clip)$/.test(overflow);
    const clipX = clips(style.overflowX || style.overflow);
    const clipY = clips(style.overflowY || style.overflow);
    if (!clipX && !clipY) continue;
    const box = parent.getBoundingClientRect();
    const x = box.left + parent.clientLeft, y = box.top + parent.clientTop;
    if (clipX) { left = Math.max(left, x); right = Math.min(right, x + parent.clientWidth); }
    if (clipY) { top = Math.max(top, y); bottom = Math.min(bottom, y + parent.clientHeight); }
  }
  return right > left && bottom > top ? new DOMRect(left, top, right - left, bottom - top) : null;
}
