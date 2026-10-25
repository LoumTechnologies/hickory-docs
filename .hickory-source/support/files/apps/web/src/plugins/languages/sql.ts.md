<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/sql.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `54bebf0f96791e11b64889737c9d33b1f805b9b2eb9911a6159c9a999acdebac`.

<book:entry path="apps/web/src/plugins/languages/sql.ts">
import { sql } from "@codemirror/lang-sql";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "sql", grammar: () => sql(), aliases: ["postgres", "postgresql", "mysql", "sqlite", "plsql", "tsql"], comment: ["// ", ""] };
</book:entry>

</book:doc>
