<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/scss.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-daedcd15ec1c"></a>
## `apps/web/src/plugins/languages/scss.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-daedcd15ec1c-plugin"></a>
### `plugin`

<storycode:copy id="f-daedcd15ec1c-plugin">
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "scss", aliases: [], comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/scss.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/scss.ts"><storycode:paste select="#f-daedcd15ec1c-plugin"/></storycode:file>

</storycode:doc>
