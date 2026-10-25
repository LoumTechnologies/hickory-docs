import { toml } from "@codemirror/legacy-modes/mode/toml";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "toml", grammar: () => StreamLanguage.define(toml), aliases: [], comment: ["# ", ""] };
