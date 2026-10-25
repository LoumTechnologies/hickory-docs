import { python } from "@codemirror/lang-python";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "python", grammar: () => python(), aliases: ["py"], tests: { definition: /^\s*(?:async\s+)?def\s+(test_[A-Za-z0-9_]*)\s*\(/, group: 1 }, comment: ["# ", ""] };
