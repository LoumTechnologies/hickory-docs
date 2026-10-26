<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/html.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-afbb6157c0aa"></a>
## `apps/web/src/plugins/languages/html.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/plugins/languages/html.ts">
import { html } from "@codemirror/lang-html";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "html", grammar: () => html(), aliases: ["htm"], comment: ["<!-- ", " -->"] };
</storycode:entry>

</storycode:doc>
