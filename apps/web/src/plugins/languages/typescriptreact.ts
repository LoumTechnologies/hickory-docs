import { javascript } from "@codemirror/lang-javascript";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "typescriptreact", grammar: () => javascript({ typescript: true, jsx: true }), aliases: ["tsx", "typescript-react"], tests: { definition: /^\s*(?:it|test)(?:\.(?:only|skip|each))?\s*\(\s*(["'`])((?:\\.|(?!\1).)*)\1/, group: 2 }, comment: ["// ", ""] };
