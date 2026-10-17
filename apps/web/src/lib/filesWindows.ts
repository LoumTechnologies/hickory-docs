import { useEffect, useState } from "react";

export type FilesWindows = boolean;
export const FILES_WINDOWS_KEY = "hickory.filesWindows";
const event = "hickory-files-windows";
const defaults = (): FilesWindows => false;

export function loadFilesWindows(storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage): FilesWindows {
  try {
    const stored = JSON.parse(storage?.getItem(FILES_WINDOWS_KEY) ?? "null");
    return stored === true;
  } catch { return defaults(); }
}
export function saveFilesWindows(enabled: FilesWindows) {
  try { localStorage.setItem(FILES_WINDOWS_KEY, JSON.stringify(enabled)); } catch { /* Presentation storage is optional. */ }
  window.dispatchEvent(new CustomEvent(event, { detail: enabled }));
}
export function useFilesWindows() {
  const [enabled, setEnabled] = useState(loadFilesWindows);
  useEffect(() => {
    const local = (change: Event) => setEnabled((change as CustomEvent<FilesWindows>).detail);
    const otherWindow = (change: StorageEvent) => { if (!change.key || change.key === FILES_WINDOWS_KEY) setEnabled(loadFilesWindows()); };
    window.addEventListener(event, local); window.addEventListener("storage", otherWindow);
    return () => { window.removeEventListener(event, local); window.removeEventListener("storage", otherWindow); };
  }, []);
  return enabled;
}
