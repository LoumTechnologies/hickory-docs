import { yaml } from "@codemirror/legacy-modes/mode/yaml";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "yaml", grammar: () => StreamLanguage.define(yaml), aliases: ["yml"], comment: ["# ", ""] };
