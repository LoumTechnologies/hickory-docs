<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/ruby.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1f24c6bef142"></a>
## `apps/web/src/plugins/languages/ruby.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-1f24c6bef142-plugin"></a>
### `plugin`

<storycode:copy id="f-1f24c6bef142-plugin">
import { ruby } from "@codemirror/legacy-modes/mode/ruby";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "ruby", grammar: () => StreamLanguage.define(ruby), aliases: ["rb"], comment: ["# ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/ruby.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/ruby.ts"><storycode:paste select="#f-1f24c6bef142-plugin"/></storycode:file>

</storycode:doc>
