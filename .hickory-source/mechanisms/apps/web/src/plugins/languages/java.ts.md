<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/java.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-84c503c186fc"></a>
## `apps/web/src/plugins/languages/java.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-84c503c186fc-plugin"></a>
### `plugin`

<storycode:copy id="f-84c503c186fc-plugin">
import { java } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "java", grammar: () => StreamLanguage.define(java), aliases: [], comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/java.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/java.ts"><storycode:paste select="#f-84c503c186fc-plugin"/></storycode:file>

</storycode:doc>
