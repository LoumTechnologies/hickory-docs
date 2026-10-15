import { expect, it } from "vitest";
import { freeform, open, panes, tab } from "../shell/layout";
import { normalizeUi } from "../lib/uiState";
import { restoreFolderLayout } from "./restoreFolderLayout";

// Guarantee: docs/guarantees/authoring/a-folder-remembers-its-window.md
it("restores the folder while retaining an edited unnamed note", () => {
  const draft = tab("untitled", "untitled", "My draft");
  const current = open(freeform(), draft);
  const stored = open(freeform(), tab("file", "notes.txt"));
  const restored = restoreFolderLayout(current, stored, true);
  expect(panes(restored.root).flatMap(pane => pane.tabs)).toEqual([panes(stored.root)[0].tabs[0], draft]);
});

it("replaces only the untouched introduction and preserves the saved focus", () => {
  const intro = tab("untitled", "untitled");
  const stored = open(freeform(), tab("file", "notes.txt"));
  expect(restoreFolderLayout(open(freeform(), intro), stored, true, intro.id)).toBe(stored);
});

it("retains unnamed recovery identities when normalizing a stored layout", () => {
  const draft = tab("untitled", "untitled");
  const stored = open(freeform(), draft);
  const normalized = normalizeUi({ version: 1, layout: stored }).layout!;
  expect(panes(normalized.root)[0].tabs[0].id).toBe(draft.id);
});
