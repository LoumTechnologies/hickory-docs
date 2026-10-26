<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/markdown.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-05005d9f0128"></a>
## `apps/web/src/plugins/languages/markdown.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/plugins/languages/markdown.ts">
import { markdown } from "@codemirror/lang-markdown";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "markdown", grammar: () => markdown(), aliases: ["md"], comment: ["<!-- ", " -->"] };
</storycode:entry>

</storycode:doc>
