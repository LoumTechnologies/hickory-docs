import { json } from "@codemirror/lang-json";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "json", grammar: () => json(), aliases: [], comment: ["// ", ""] };
