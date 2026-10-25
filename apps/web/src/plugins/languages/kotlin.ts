import { kotlin } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "kotlin", grammar: () => StreamLanguage.define(kotlin), aliases: ["kt"], comment: ["// ", ""] };
