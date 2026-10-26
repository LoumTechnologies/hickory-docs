<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/javascriptreact.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-20969e290b60"></a>
## `apps/web/src/plugins/languages/javascriptreact.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-20969e290b60-plugin"></a>
### `plugin`

<storycode:copy id="f-20969e290b60-plugin">
import { javascript } from "@codemirror/lang-javascript";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "javascriptreact", grammar: () => javascript({ jsx: true }), aliases: ["jsx", "javascript-react"], tests: { definition: /^\s*(?:it|test)(?:\.(?:only|skip|each))?\s*\(\s*(["'`])((?:\\.|(?!\1).)*)\1/, group: 2 }, comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/javascriptreact.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/javascriptreact.ts"><storycode:paste select="#f-20969e290b60-plugin"/></storycode:file>

</storycode:doc>
