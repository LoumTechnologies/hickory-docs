// What Ctrl+← and Ctrl+→ count as one word.
//
// "Whole words" is what every editor does by default: a run of word
// characters, stopping at space and punctuation. "Subwords" also stops inside
// an identifier — `PascalCase`, `camelCase`, `snake_case`, `XMLHttpRequest` —
// which is the motion you want in code and the wrong one in prose. Since a
// `.md` document is both, this cannot be inferred from the file; it is a
// preference about the person, so it is one.
//
// An editing preference rather than a presentation one, but it persists the
// same way for the same reason: there is no account to hang it on, so it is
// localStorage, per browser, with a hard default for anything unset or
// unrecognisable. Mirrors lib/tabStyle.ts.

export type WordMotion = "word" | "subword";

export const WORD_MOTION_KEY = "hickory.wordMotion";

const isMotion = (value: unknown): value is WordMotion =>
  value === "word" || value === "subword";

/** The persisted motion, defaulting to "word" on anything unset or invalid. */
export function loadWordMotion(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined"
    ? null
    : localStorage,
): WordMotion {
  const stored = storage?.getItem(WORD_MOTION_KEY);
  return isMotion(stored) ? stored : "word";
}

export function saveWordMotion(
  motion: WordMotion,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined"
    ? null
    : localStorage,
): void {
  storage?.setItem(WORD_MOTION_KEY, motion);
}
