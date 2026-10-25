import { xml } from "@codemirror/legacy-modes/mode/xml";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "xml", grammar: () => StreamLanguage.define(xml), aliases: [], comment: ["<!-- ", " -->"] };
