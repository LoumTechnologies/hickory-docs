import { c } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "c", grammar: () => StreamLanguage.define(c), aliases: ["objective-c"], comment: ["// ", ""] };
