import { fSharp } from "@codemirror/legacy-modes/mode/mllike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "fsharp", grammar: () => StreamLanguage.define(fSharp), aliases: ["f#"], comment: ["// ", ""] };
