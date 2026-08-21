// Keeping one editor's blame column filled and in step with the toggle.
//
// The fetch is deliberately lazy: nothing is asked of git until somebody
// turns the column on. That is most of what makes an off-by-default feature
// actually free — a `git blame` per open file at startup would cost real time
// on a large repository to fill a column nobody is looking at.
//
// It refills when the file is SAVED rather than as you type. Blame describes
// what is committed; re-running it on every keystroke would burn a process
// per character to move a label that has not changed.

import { useEffect } from "react";
import type { EditorView } from "@codemirror/view";

import { api } from "../api/client";
import { setBlame, setBlameShown } from "./blameGutter";
import { loadBlameShown, onBlameChanged } from "../lib/blamePref";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";

export function useBlame(view: EditorView | null, path: string | null): void {
  useEffect(() => {
    if (!view || !path) return;
    let live = true;

    const fill = () => {
      api.blame(path).then(
        (answer) => {
          if (!live || !view.dom.isConnected) return;
          view.dispatch({ effects: setBlame.of(answer.lines) });
        },
        () => {
          // No git, no repository, an untracked file: the column is simply
          // blank. Nothing here is worth interrupting anyone over.
          if (live && view.dom.isConnected) view.dispatch({ effects: setBlame.of([]) });
        },
      );
    };

    const apply = (shown: boolean) => {
      if (!view.dom.isConnected) return;
      view.dispatch({ effects: setBlameShown.of(shown) });
      if (shown) fill();
    };

    apply(loadBlameShown());
    const offToggle = onBlameChanged(apply);
    // A save (or anything else that rewrites files) can change what is
    // committed under this buffer.
    const onFiles = () => {
      if (loadBlameShown()) fill();
    };
    window.addEventListener(FILES_CHANGED_EVENT, onFiles);
    return () => {
      live = false;
      offToggle();
      window.removeEventListener(FILES_CHANGED_EVENT, onFiles);
    };
  }, [view, path]);
}
