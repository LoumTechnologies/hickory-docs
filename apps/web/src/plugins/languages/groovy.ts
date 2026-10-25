import { groovy } from "@codemirror/legacy-modes/mode/groovy";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "groovy", grammar: () => StreamLanguage.define(groovy), aliases: [], comment: ["// ", ""] };
