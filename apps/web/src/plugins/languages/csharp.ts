import { csharp } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "csharp", grammar: () => StreamLanguage.define(csharp), aliases: ["c#", "cs", "dotnet"], tests: { definition: /^\s*(?:public\s+|private\s+|internal\s+|static\s+|async\s+|virtual\s+|override\s+)*[A-Za-z_<>\[\],. ]+?\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/, group: 1, attribute: /^\s*\[\s*(?:Fact|Theory|Test|TestMethod|TestCase)\b/ }, comment: ["// ", ""] };
