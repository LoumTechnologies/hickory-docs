import { r } from "@codemirror/legacy-modes/mode/r";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "r", grammar: () => StreamLanguage.define(r), aliases: [], comment: ["# ", ""] };
