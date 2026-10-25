<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/types.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c912e23e9c219992b9f3a5fdfe663a2b74fb0fe5d74acc403a090c184b5b249d`.

<book:entry path="apps/web/src/plugins/languages/types.ts">
import type { Extension } from "@codemirror/state";
import type { RoutedLanguageId } from "../../editor/generated/languages";
export interface LanguageUiPlugin {
  id: RoutedLanguageId;
  grammar?: () => Extension;
  aliases: readonly string[];
  tests?: { attribute?: RegExp; definition: RegExp; group: number };
  comment: readonly [string, string];
}
</book:entry>

</book:doc>
