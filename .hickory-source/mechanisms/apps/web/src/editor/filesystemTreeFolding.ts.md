<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Fold an indented filesystem buffer

The app's Files editor and the published read-only browser show the same indented path projection. Share the directory fold service instead of teaching each host a different subtree boundary. A directory ends at the last following line with greater indentation; folding hides that range without removing any paths from the buffer. Host policy decides which directories start open and how activation works.

<static:copy id="static-apps-web-src-editor-filesystemTreeFolding-ts-0">
import { codeFolding, foldGutter, foldService } from "@codemirror/language";
import type { Extension } from "@codemirror/state";

export function filesystemTreeFolding(): Extension {
  return [codeFolding(), foldGutter(), foldService.of((state, from) => {
    const line = state.doc.lineAt(from);
    const spaces = line.text.length - line.text.trimStart().length;
    if (!line.text.trimEnd().endsWith("/")) return null;
    let end = line.to;
    for (let number = line.number + 1; number <= state.doc.lines; number += 1) {
      const child = state.doc.line(number);
      const childSpaces = child.text.length - child.text.trimStart().length;
      if (child.text.trim() && childSpaces <= spaces) break;
      end = child.to;
    }
    return end > line.to ? { from: line.to, to: end } : null;
  })];
}
</static:copy>

<static:file path="apps/web/src/editor/filesystemTreeFolding.ts"><static:paste select="#static-apps-web-src-editor-filesystemTreeFolding-ts-0"/></static:file>
</static:doc>
