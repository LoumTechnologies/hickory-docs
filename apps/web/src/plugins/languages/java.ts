import { java } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "java", grammar: () => StreamLanguage.define(java), aliases: [], comment: ["// ", ""] };
