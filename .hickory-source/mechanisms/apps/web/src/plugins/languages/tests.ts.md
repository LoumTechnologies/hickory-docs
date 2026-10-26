<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/tests.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8853a6b18e45"></a>
## `apps/web/src/plugins/languages/tests.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-8853a6b18e45-TestMark"></a>
### `TestMark`

<storycode:copy id="f-8853a6b18e45-TestMark">
import { UI_LANGUAGE_BY_ID, UI_LANGUAGE_ALIASES } from "./index";
import { EXTENSION_TO_LANGUAGE, type RoutedLanguageId } from "../../editor/generated/languages";
export interface TestMark { line: number; name: string }
</storycode:copy>
<a id="f-8853a6b18e45-findTests"></a>
### `findTests`

<storycode:copy id="f-8853a6b18e45-findTests">
export function findTests(language: string, text: string): TestMark[] {
  const key = UI_LANGUAGE_ALIASES[language] ?? EXTENSION_TO_LANGUAGE[language] ?? language;
  const shape = UI_LANGUAGE_BY_ID.get(key as RoutedLanguageId)?.tests;
  if (!shape) return [];
  const lines = text.split("\n"), marks: TestMark[] = [];
  if (shape.attribute) {
    for (let i = 0; i < lines.length; i++) {
      if (!shape.attribute.test(lines[i])) continue;
      for (let j = i + 1; j < lines.length && j <= i + 6; j++) {
        if (lines[j].trim() === "" || /^\s*(#\[|\[)/.test(lines[j])) continue;
        const found = shape.definition.exec(lines[j]);
        if (found) marks.push({ line: j + 1, name: found[shape.group] });
        break;
      }
    }
  } else {
    lines.forEach((line, i) => {
      const found = shape.definition.exec(line);
      if (found) marks.push({ line: i + 1, name: found[shape.group] });
    });
  }
  return marks;
}
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/tests.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/tests.ts"><storycode:paste select="#f-8853a6b18e45-TestMark"/><storycode:paste select="#f-8853a6b18e45-findTests"/></storycode:file>

</storycode:doc>
