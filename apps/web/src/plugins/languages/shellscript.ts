import { shell } from "@codemirror/legacy-modes/mode/shell";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "shellscript", grammar: () => StreamLanguage.define(shell), aliases: ["bash", "sh", "shell", "zsh", "console"], comment: ["# ", ""] };
