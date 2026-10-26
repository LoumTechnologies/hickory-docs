<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/types.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8083f5eff818"></a>
## `apps/web/src/plugins/languages/types.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-8083f5eff818-LanguageUiPlugin"></a>
### `LanguageUiPlugin`

<storycode:copy id="f-8083f5eff818-LanguageUiPlugin">
import type { Extension } from "@codemirror/state";
import type { RoutedLanguageId } from "../../editor/generated/languages";
export interface LanguageUiPlugin {
  id: RoutedLanguageId;
  grammar?: () => Extension;
  aliases: readonly string[];
  tests?: { attribute?: RegExp; definition: RegExp; group: number };
  comment: readonly [string, string];
}
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/types.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/types.ts"><storycode:paste select="#f-8083f5eff818-LanguageUiPlugin"/></storycode:file>

</storycode:doc>
