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
