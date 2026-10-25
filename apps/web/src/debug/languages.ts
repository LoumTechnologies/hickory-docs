// Debug availability comes from the same language plugins as the native host.
import { DEBUGGABLE_LANGUAGE_IDS } from "../editor/generated/languages";

export const DEBUGGABLE_LANGUAGES = new Set(DEBUGGABLE_LANGUAGE_IDS);

export function isDebuggable(language: string | null | undefined): boolean {
  return !!language && DEBUGGABLE_LANGUAGES.has(language);
}
