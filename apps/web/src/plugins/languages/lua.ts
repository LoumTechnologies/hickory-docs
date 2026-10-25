import { lua } from "@codemirror/legacy-modes/mode/lua";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "lua", grammar: () => StreamLanguage.define(lua), aliases: [], comment: ["// ", ""] };
