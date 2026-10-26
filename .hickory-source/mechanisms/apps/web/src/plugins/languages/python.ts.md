<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/python.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-97caed4f3fef"></a>
## `apps/web/src/plugins/languages/python.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-97caed4f3fef-plugin"></a>
### `plugin`

<storycode:copy id="f-97caed4f3fef-plugin">
import { python } from "@codemirror/lang-python";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "python", grammar: () => python(), aliases: ["py"], tests: { definition: /^\s*(?:async\s+)?def\s+(test_[A-Za-z0-9_]*)\s*\(/, group: 1 }, comment: ["# ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/python.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/python.ts"><storycode:paste select="#f-97caed4f3fef-plugin"/></storycode:file>

</storycode:doc>
