import { UI_LANGUAGE_BY_ID, UI_LANGUAGE_ALIASES } from "./index";
import { EXTENSION_TO_LANGUAGE, type RoutedLanguageId } from "../../editor/generated/languages";
export interface TestMark { line: number; name: string }
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
