import { useEffect, useRef } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { editorChrome } from "../editor/chrome";
import { justRecipeGutter, launchJustRecipe } from "../editor/justRecipeGutter";

/** A root needs a real gutter row too; it isn't a file in the editable buffer. */
export function RecipeFolderHeader({ name, paths, revision, onOpenTerminal, onError }: {
  name: string; paths: string[]; revision: unknown; onOpenTerminal?: (id: string) => void; onError: (reason: string) => void;
}) {
  const host = useRef<HTMLDivElement>(null);
  const callbacks = useRef({ onOpenTerminal, onError }); callbacks.current = { onOpenTerminal, onError };
  const version = JSON.stringify(paths);
  useEffect(() => {
    if (!host.current) return;
    let alive = true, pending = false;
    const view = new EditorView({ parent: host.current, state: EditorState.create({ doc: name, extensions: [
      editorChrome("code"), EditorState.readOnly.of(true), EditorView.editable.of(false),
      EditorView.contentAttributes.of({ "aria-label": "Workspace folder" }),
      justRecipeGutter(paths.map(path => ({ path, line: 1 })), () => alive, (path, recipe) => {
        if (pending) return;
        pending = true;
        void launchJustRecipe(path, recipe).then(session => { if (alive && session) callbacks.current.onOpenTerminal?.(session.id); })
          .catch(error => { if (alive) callbacks.current.onError(error instanceof Error ? error.message : String(error)); })
          .finally(() => { pending = false; });
      }, reason => callbacks.current.onError(reason)),
      EditorView.theme({
        "&": { overflow: "visible" },
        "&:has(.cm-gutter-overflow)": { zIndex: "201" },
        ".cm-scroller": { overflow: "visible" },
        ".cm-scroller > .cm-content": { padding: "0" },
        ".cm-scroller .cm-line": { padding: "0" },
        ".cm-gutters-after": { boxSizing: "border-box", flex: "0 0 20px", width: "20px" },
        ".cm-just-gutter": { minWidth: "0", width: "0" },
      }),
    ] }) });
    return () => { alive = false; view.destroy(); };
  }, [name, version, revision]);
  return <div ref={host} style={{ flex: 1, minWidth: 0 }} />;
}
