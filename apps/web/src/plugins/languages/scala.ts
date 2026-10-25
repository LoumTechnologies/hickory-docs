import { scala } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "scala", grammar: () => StreamLanguage.define(scala), aliases: [], comment: ["// ", ""] };
