import { css } from "@codemirror/lang-css";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "css", grammar: () => css(), aliases: [], comment: ["// ", ""] };
