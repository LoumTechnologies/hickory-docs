/** Editing preference, persisted per browser profile like word navigation. */
export const UNWRAP_PARAGRAPHS_KEY = "hickory.unwrapParagraphs";

export function loadUnwrapParagraphs(): boolean {
  return typeof localStorage === "undefined" ||
    localStorage.getItem(UNWRAP_PARAGRAPHS_KEY) !== "false";
}

export function saveUnwrapParagraphs(on: boolean): void {
  localStorage.setItem(UNWRAP_PARAGRAPHS_KEY, String(on));
}
