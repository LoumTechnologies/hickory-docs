<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/r.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9edb56648a7d"></a>
## `apps/web/src/plugins/languages/r.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-9edb56648a7d-plugin"></a>
### `plugin`

<storycode:copy id="f-9edb56648a7d-plugin">
import { r } from "@codemirror/legacy-modes/mode/r";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "r", grammar: () => StreamLanguage.define(r), aliases: [], comment: ["# ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/r.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/r.ts"><storycode:paste select="#f-9edb56648a7d-plugin"/></storycode:file>

</storycode:doc>
