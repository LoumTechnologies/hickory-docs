<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/sql.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-37f4d7263836"></a>
## `apps/web/src/plugins/languages/sql.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-37f4d7263836-plugin"></a>
### `plugin`

<storycode:copy id="f-37f4d7263836-plugin">
import { sql } from "@codemirror/lang-sql";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "sql", grammar: () => sql(), aliases: ["postgres", "postgresql", "mysql", "sqlite", "plsql", "tsql"], comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/sql.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/sql.ts"><storycode:paste select="#f-37f4d7263836-plugin"/></storycode:file>

</storycode:doc>
