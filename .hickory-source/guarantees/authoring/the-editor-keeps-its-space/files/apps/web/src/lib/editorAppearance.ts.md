<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/editorAppearance.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The editor keeps its space](../../../../../contract.md)

Captured SHA-256: `024be7f8ecfa68d05515271500e8879334675bbb0e826db291ae1425f09fd6e9`.

<book:entry path="apps/web/src/lib/editorAppearance.ts">
import "../editor/density.css";

export type EditorDensity = "compact" | "comfortable";
export interface EditorAppearance { density: EditorDensity; touchTargets: boolean; rightLineNumbers: boolean }
export const EDITOR_APPEARANCE_KEY = "hickory.editorAppearance";

export function loadEditorAppearance(storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage): EditorAppearance {
  try {
    const value = JSON.parse(storage?.getItem(EDITOR_APPEARANCE_KEY) ?? "null");
    return { density: value?.density === "comfortable" ? "comfortable" : "compact", touchTargets: value?.touchTargets === true, rightLineNumbers: value?.rightLineNumbers !== false };
  } catch { return { density: "compact", touchTargets: false, rightLineNumbers: true }; }
}
export function applyEditorAppearance(value: EditorAppearance, root: HTMLElement = document.documentElement) {
  root.dataset.editorDensity = value.density;
  root.dataset.rightLineNumbers = String(value.rightLineNumbers);
  root.dataset.touchTargets = String(value.touchTargets);
}
export function saveEditorAppearance(value: EditorAppearance) {
  try { localStorage.setItem(EDITOR_APPEARANCE_KEY, JSON.stringify(value)); } catch { /* Optional presentation storage. */ }
  applyEditorAppearance(value);
}
</book:entry>

</book:doc>
