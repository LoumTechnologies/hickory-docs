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
