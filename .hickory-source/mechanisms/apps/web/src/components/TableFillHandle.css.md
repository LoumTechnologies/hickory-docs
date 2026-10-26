<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/TableFillHandle.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-21a63153e47e"></a>
## `apps/web/src/components/TableFillHandle.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/components/TableFillHandle.css">
.table-fill-handle {
  position: absolute;
  right: 0;
  bottom: 0;
  z-index: 4;
  width: 14px;
  height: 14px;
  padding: 0;
  border: 1px solid var(--bg);
  border-radius: 2px;
  background: var(--accent);
  color: var(--bg);
  font: bold 13px/12px ui-monospace, monospace;
  cursor: crosshair;
}
.table-fill-handle:hover { filter: brightness(1.15); }
.table-panel__grid td.table-fill-preview {
  background: var(--select-bg);
  outline: 1px dashed var(--accent);
  outline-offset: -1px;
}
.table-fill-range {
  position: absolute;
  right: 0;
  bottom: 16px;
  z-index: 5;
  white-space: nowrap;
  pointer-events: none;
  padding: 2px 5px;
  border: 1px solid var(--accent);
  background: var(--bg);
  color: var(--fg);
  font-size: 0.7rem;
}
</storycode:entry>

</storycode:doc>
