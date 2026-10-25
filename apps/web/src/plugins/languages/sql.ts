import { sql } from "@codemirror/lang-sql";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "sql", grammar: () => sql(), aliases: ["postgres", "postgresql", "mysql", "sqlite", "plsql", "tsql"], comment: ["// ", ""] };
