import { php } from "@codemirror/lang-php";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "php", grammar: () => php(), aliases: [], comment: ["// ", ""] };
