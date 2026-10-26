<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/useBlame.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9b02a0106d13"></a>
## `apps/web/src/editor/useBlame.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Review threads preserve the questioned target](../../../../../guarantees/authoring/review-threads-preserve-the-questioned-target/contract.md), [Authorship display accepts non-Git providers](../../../../../guarantees/lineage/authorship-display-accepts-non-git-providers/contract.md), [The Blame Column Is Optional And Honest](../../../../../guarantees/lineage/the-blame-column-is-optional-and-honest/contract.md).

<a id="f-9b02a0106d13-useBlame"></a>
### `useBlame`

Keeping one editor's blame column filled and in step with the toggle.
The fetch is deliberately lazy: nothing is asked of git until somebody
turns the column on. That is most of what makes an off-by-default feature
actually free — a `git blame` per open file at startup would cost real time
on a large repository to fill a column nobody is looking at.
It refills when the file is SAVED rather than as you type. Blame describes
what is committed; re-running it on every keystroke would burn a process
per character to move a label that has not changed.

<storycode:copy id="f-9b02a0106d13-useBlame">
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

import { registerReviewEditor } from "../lib/reviews";
import { useEffect } from "react";
import type { EditorView } from "@codemirror/view";

import { attributionProvider, authorshipColors, ATTRIBUTION_CHANGED } from "../lib/attribution";
import { setBlame, setBlameShown, setAuthorshipShown } from "./blameGutter";
import { loadBlameShown, onBlameChanged } from "../lib/blamePref";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";

export function useBlame(view: EditorView | null, path: string | null): void {
  useEffect(() => {
    if (!view || !path) return;
    const unregisterReview = registerReviewEditor(view, path);
    let live = true;
    let generation = 0;

    const fill = () => {
      const request = ++generation;
      const source = view.state.doc;
      attributionProvider().lines(path).then(
        (answer) => {
          if (!live || !view.dom.isConnected || request !== generation || source !== view.state.doc) return;
          view.dispatch({ effects: setBlame.of(answer.filter(line => line.line > 0 && line.line <= source.lines && line.text === source.line(line.line).text)) });
        },
        () => {
          // No git, no repository, an untracked file: the column is simply
          // blank. Nothing here is worth interrupting anyone over.
          if (live && view.dom.isConnected && request === generation) view.dispatch({ effects: setBlame.of([]) });
        },
      );
    };

    const apply = (shown: boolean) => {
      if (!view.dom.isConnected) return;
      view.dispatch({ effects: setBlameShown.of(shown) });
      if (shown) fill();
    };

    const update = () => {
      view.dispatch({ effects: setAuthorshipShown.of(authorshipColors()) });
      apply(loadBlameShown());
      if (authorshipColors() && !loadBlameShown()) fill();
    };
    update();
    window.addEventListener(ATTRIBUTION_CHANGED, update);
    const offToggle = onBlameChanged(apply);
    // A save (or anything else that rewrites files) can change what is
    // committed under this buffer.
    const onFiles = () => {
      if (loadBlameShown() || authorshipColors()) fill();
    };
    window.addEventListener(FILES_CHANGED_EVENT, onFiles);
    return () => {
      live = false;
      unregisterReview();
      offToggle();
      window.removeEventListener(ATTRIBUTION_CHANGED, update);
      window.removeEventListener(FILES_CHANGED_EVENT, onFiles);
    };
  }, [view, path]);
}
</storycode:copy>

### Assemble `apps/web/src/editor/useBlame.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/useBlame.ts"><storycode:paste select="#f-9b02a0106d13-useBlame"/></storycode:file>

</storycode:doc>
