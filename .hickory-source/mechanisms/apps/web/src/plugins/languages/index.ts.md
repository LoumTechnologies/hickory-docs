<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/index.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-84adcd7173b4"></a>
## `apps/web/src/plugins/languages/index.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-84adcd7173b4-UI_LANGUAGE_PLUGINS"></a>
### `UI_LANGUAGE_PLUGINS`

<storycode:copy id="f-84adcd7173b4-UI_LANGUAGE_PLUGINS">
import { plugin as less } from "./less";
import { plugin as scss } from "./scss";
import { plugin as dockerfile } from "./dockerfile";
import { plugin as terraform } from "./terraform";
import { plugin as haskell } from "./haskell";
import { plugin as elixir } from "./elixir";
// UI adapters supply rendering and textual detection; supported capabilities
// come from the native plugin registry through generated metadata.
import type { Extension } from "@codemirror/state";
import type { RoutedLanguageId } from "../../editor/generated/languages";
import { LANGUAGE_CAPABILITIES, EXTENSION_TO_LANGUAGE } from "../../editor/generated/languages";
import type { LanguageUiPlugin } from "./types";
import { plugin as python } from "./python";
import { plugin as javascript } from "./javascript";
import { plugin as javascriptreact } from "./javascriptreact";
import { plugin as typescript } from "./typescript";
import { plugin as typescriptreact } from "./typescriptreact";
import { plugin as rust } from "./rust";
import { plugin as json } from "./json";
import { plugin as markdown } from "./markdown";
import { plugin as html } from "./html";
import { plugin as css } from "./css";
import { plugin as sql } from "./sql";
import { plugin as php } from "./php";
import { plugin as shellscript } from "./shellscript";
import { plugin as csharp } from "./csharp";
import { plugin as xml } from "./xml";
import { plugin as go } from "./go";
import { plugin as java } from "./java";
import { plugin as c } from "./c";
import { plugin as cpp } from "./cpp";
import { plugin as kotlin } from "./kotlin";
import { plugin as scala } from "./scala";
import { plugin as dart } from "./dart";
import { plugin as ruby } from "./ruby";
import { plugin as swift } from "./swift";
import { plugin as lua } from "./lua";
import { plugin as groovy } from "./groovy";
import { plugin as r } from "./r";
import { plugin as fsharp } from "./fsharp";
import { plugin as vb } from "./vb";
import { plugin as toml } from "./toml";
import { plugin as yaml } from "./yaml";
import { plugin as nix } from "./nix";
import { plugin as zig } from "./zig";

export const UI_LANGUAGE_PLUGINS: readonly LanguageUiPlugin[] = [elixir, haskell, terraform, dockerfile, scss, less, python, javascript, javascriptreact, typescript, typescriptreact, rust, json, markdown, html, css, sql, php, shellscript, csharp, xml, go, java, c, cpp, kotlin, scala, dart, ruby, swift, lua, groovy, r, fsharp, vb, toml, yaml, nix, zig];
</storycode:copy>
<a id="f-84adcd7173b4-UI_LANGUAGE_ALIASES"></a>
### `UI_LANGUAGE_ALIASES`

<storycode:copy id="f-84adcd7173b4-UI_LANGUAGE_ALIASES">
export const UI_LANGUAGE_ALIASES: Record<string, RoutedLanguageId> = {};
</storycode:copy>
<a id="f-84adcd7173b4-UI_LANGUAGE_GRAMMARS"></a>
### `UI_LANGUAGE_GRAMMARS`

<storycode:copy id="f-84adcd7173b4-UI_LANGUAGE_GRAMMARS">
export const UI_LANGUAGE_GRAMMARS: Partial<Record<RoutedLanguageId, () => Extension>> = {};
</storycode:copy>
<a id="f-84adcd7173b4-UI_LANGUAGE_BY_ID"></a>
### `UI_LANGUAGE_BY_ID`

<storycode:copy id="f-84adcd7173b4-UI_LANGUAGE_BY_ID">
export const UI_LANGUAGE_BY_ID = new Map(UI_LANGUAGE_PLUGINS.map(plugin => [plugin.id, plugin]));
</storycode:copy>
<a id="f-84adcd7173b4-for_in_statement"></a>
### `for_in_statement`

<storycode:copy id="f-84adcd7173b4-for_in_statement">
for (const plugin of UI_LANGUAGE_PLUGINS) {
  if (plugin.grammar) UI_LANGUAGE_GRAMMARS[plugin.id] = plugin.grammar;
  for (const alias of plugin.aliases) {
    if (UI_LANGUAGE_ALIASES[alias]) throw new Error(`Conflicting language alias: ${alias}`);
    UI_LANGUAGE_ALIASES[alias] = plugin.id;
  }
  if (plugin.tests && !LANGUAGE_CAPABILITIES[plugin.id]?.includes("TestRunner")) {
    throw new Error(`Test detection has no native runner: ${plugin.id}`);
  }
}

</storycode:copy>
<a id="f-84adcd7173b4-languageComment"></a>
### `languageComment`

<storycode:copy id="f-84adcd7173b4-languageComment">
export function languageComment(language: string, text: string): string {
  const key = UI_LANGUAGE_ALIASES[language] ?? EXTENSION_TO_LANGUAGE[language] ?? language;
  const [prefix, suffix] = UI_LANGUAGE_BY_ID.get(key as RoutedLanguageId)?.comment ?? ["// ", ""];
  return `${prefix}${text}${suffix}\n`;
}
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/index.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/index.ts"><storycode:paste select="#f-84adcd7173b4-UI_LANGUAGE_PLUGINS"/><storycode:paste select="#f-84adcd7173b4-UI_LANGUAGE_ALIASES"/><storycode:paste select="#f-84adcd7173b4-UI_LANGUAGE_GRAMMARS"/><storycode:paste select="#f-84adcd7173b4-UI_LANGUAGE_BY_ID"/><storycode:paste select="#f-84adcd7173b4-for_in_statement"/><storycode:paste select="#f-84adcd7173b4-languageComment"/></storycode:file>

</storycode:doc>
