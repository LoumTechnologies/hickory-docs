import { dart } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "dart", grammar: () => StreamLanguage.define(dart), aliases: [], comment: ["// ", ""] };
