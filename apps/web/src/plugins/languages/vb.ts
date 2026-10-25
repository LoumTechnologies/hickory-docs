import { vb } from "@codemirror/legacy-modes/mode/vb";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "vb", grammar: () => StreamLanguage.define(vb), aliases: ["visual-basic", "vbnet"], comment: ["// ", ""] };
