import type { Extension } from "@codemirror/state";
import type { RoutedLanguageId } from "../../editor/generated/languages";
export interface LanguageUiPlugin {
  id: RoutedLanguageId;
  grammar?: () => Extension;
  aliases: readonly string[];
  tests?: { attribute?: RegExp; definition: RegExp; group: number };
  comment: readonly [string, string];
}
