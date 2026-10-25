import { rust } from "@codemirror/lang-rust";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "rust", grammar: () => rust(), aliases: ["rs"], tests: { definition: /^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/, group: 1, attribute: /^\s*#\[\s*(?:tokio::|async_std::)?test\b/ }, comment: ["// ", ""] };
