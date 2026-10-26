<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/visibleElementRect.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-335f501f03a1"></a>
## `apps/web/src/lib/visibleElementRect.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-335f501f03a1-visibleElementRect"></a>
### `visibleElementRect`

Visible part of an element, clipped to ancestor scrollports (excluding scrollbars).

<storycode:copy id="f-335f501f03a1-visibleElementRect">
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
</storycode:copy>

### Assemble `apps/web/src/lib/visibleElementRect.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/visibleElementRect.ts"><storycode:paste select="#f-335f501f03a1-visibleElementRect"/></storycode:file>

</storycode:doc>
