<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/typescript.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e388b3254607"></a>
## `apps/web/src/plugins/languages/typescript.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-e388b3254607-plugin"></a>
### `plugin`

<storycode:copy id="f-e388b3254607-plugin">
import { javascript } from "@codemirror/lang-javascript";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "typescript", grammar: () => javascript({ typescript: true }), aliases: ["ts"], tests: { definition: /^\s*(?:it|test)(?:\.(?:only|skip|each))?\s*\(\s*(["'`])((?:\\.|(?!\1).)*)\1/, group: 2 }, comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/typescript.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/typescript.ts"><storycode:paste select="#f-e388b3254607-plugin"/></storycode:file>

</storycode:doc>
