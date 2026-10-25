import { markdown } from "@codemirror/lang-markdown";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "markdown", grammar: () => markdown(), aliases: ["md"], comment: ["<!-- ", " -->"] };
