import { swift } from "@codemirror/legacy-modes/mode/swift";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "swift", grammar: () => StreamLanguage.define(swift), aliases: [], comment: ["// ", ""] };
