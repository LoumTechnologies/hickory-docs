<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/cpp.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f8d461817d64"></a>
## `apps/web/src/plugins/languages/cpp.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f8d461817d64-plugin"></a>
### `plugin`

<storycode:copy id="f-f8d461817d64-plugin">
import { cpp } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "cpp", grammar: () => StreamLanguage.define(cpp), aliases: ["c++"], comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/cpp.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/cpp.ts"><storycode:paste select="#f-f8d461817d64-plugin"/></storycode:file>

</storycode:doc>
