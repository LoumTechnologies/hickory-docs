import { ruby } from "@codemirror/legacy-modes/mode/ruby";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "ruby", grammar: () => StreamLanguage.define(ruby), aliases: ["rb"], comment: ["# ", ""] };
