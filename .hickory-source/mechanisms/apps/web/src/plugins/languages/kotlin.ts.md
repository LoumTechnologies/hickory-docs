<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/kotlin.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-49dc4212f249"></a>
## `apps/web/src/plugins/languages/kotlin.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-49dc4212f249-plugin"></a>
### `plugin`

<storycode:copy id="f-49dc4212f249-plugin">
import { kotlin } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "kotlin", grammar: () => StreamLanguage.define(kotlin), aliases: ["kt"], comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/kotlin.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/kotlin.ts"><storycode:paste select="#f-49dc4212f249-plugin"/></storycode:file>

</storycode:doc>
