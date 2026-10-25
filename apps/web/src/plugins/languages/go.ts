import { go } from "@codemirror/legacy-modes/mode/go";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "go", grammar: () => StreamLanguage.define(go), aliases: ["golang"], tests: { definition: /^\s*func\s+(Test[A-Za-z0-9_]*)\s*\(\s*\w+\s+\*testing\.T\s*\)/, group: 1 }, comment: ["// ", ""] };
