<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/json.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d74503e1f6c1"></a>
## `apps/web/src/plugins/languages/json.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d74503e1f6c1-plugin"></a>
### `plugin`

<storycode:copy id="f-d74503e1f6c1-plugin">
import { json } from "@codemirror/lang-json";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "json", grammar: () => json(), aliases: [], comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/json.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/json.ts"><storycode:paste select="#f-d74503e1f6c1-plugin"/></storycode:file>

</storycode:doc>
