<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/php.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c864455c795d"></a>
## `apps/web/src/plugins/languages/php.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c864455c795d-plugin"></a>
### `plugin`

<storycode:copy id="f-c864455c795d-plugin">
import { php } from "@codemirror/lang-php";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "php", grammar: () => php(), aliases: [], comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/php.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/php.ts"><storycode:paste select="#f-c864455c795d-plugin"/></storycode:file>

</storycode:doc>
