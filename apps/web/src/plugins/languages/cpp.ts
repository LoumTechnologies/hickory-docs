import { cpp } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "cpp", grammar: () => StreamLanguage.define(cpp), aliases: ["c++"], comment: ["// ", ""] };
