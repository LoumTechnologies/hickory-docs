<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/visibleElementRect.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a8efff446660694884179118bab2f1f3ed614d7d21806c7e07e21020b6fcadce`.

<book:entry path="apps/web/src/lib/visibleElementRect.ts">
/** Visible part of an element, clipped to ancestor scrollports (excluding scrollbars). */
export function visibleElementRect(element: HTMLElement): DOMRect | null {
  if (getComputedStyle(element).visibility === "hidden") return null;
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
</book:entry>

</book:doc>
