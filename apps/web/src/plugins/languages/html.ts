import { html } from "@codemirror/lang-html";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "html", grammar: () => html(), aliases: ["htm"], comment: ["<!-- ", " -->"] };
