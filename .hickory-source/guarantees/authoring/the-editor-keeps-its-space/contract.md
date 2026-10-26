<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The editor keeps its space

For: Authors editing executable documents and the files those documents produce.

Given the desktop workspace, when editing a document or plain file, then view
identity, editability and write destination use an information control in that
pane’s existing header. Document-wide Run/Test and plain-file Debug live in its
header’s action menu. Cell actions remain on the right-hand action rail;
collisions still push buttons downward. A document with no rail actions reserves
no empty action column; adding a cell restores its actions. Editor instances, source bytes and
selections are not replaced by moving their controls.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-editor-keeps-its-space.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-b862abcd8a87"></a>
## `apps/web/e2e/editor-density.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](contract.md).

<a id="f-b862abcd8a87-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b862abcd8a87-expression_statement">
import { expect, test, type Locator, type Page } from "@playwright/test";

// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
// Real native engine and CSS geometry; use just test-editor-density-ui.
test.use({ viewport: { width: 1200, height: 900 } });
</storycode:copy>
<a id="f-b862abcd8a87-center"></a>
### `center`

<storycode:copy id="f-b862abcd8a87-center">
const center = (page: Page) => page.locator('[data-spatial-center=true]');
</storycode:copy>
<a id="f-b862abcd8a87-font"></a>
### `font`

<storycode:copy id="f-b862abcd8a87-font">
const font = (element: Locator) => element.evaluate(el => parseFloat(getComputedStyle(el).fontSize));
</storycode:copy>
<a id="f-b862abcd8a87-height"></a>
### `height`

<storycode:copy id="f-b862abcd8a87-height">
const height = (element: Locator) => element.evaluate(el => el.getBoundingClientRect().height);
</storycode:copy>
<a id="f-b862abcd8a87-showFiles"></a>
### `showFiles`

<storycode:copy id="f-b862abcd8a87-showFiles">
async function showFiles(page: Page) {
  const files = page.locator('[data-spatial-files=left]');
  const handle = page.getByRole('button', { name: 'Show left drawer' });
  if (await handle.isVisible()) await handle.click();
  await expect(files).toBeVisible();
}
</storycode:copy>
<a id="f-b862abcd8a87-setting"></a>
### `setting`

<storycode:copy id="f-b862abcd8a87-setting">
async function setting(page: Page, name: string) {
  await page.goto('/#/settings');
  return page.getByRole("checkbox", { name, exact: true });
}
</storycode:copy>
<a id="f-b862abcd8a87-note"></a>
### `note`

<storycode:copy id="f-b862abcd8a87-note">
async function note(page: Page, name: string) {
  const source = '# Dense note\n\n<hick:file path="sample.py">\n' + Array.from({ length: 100 }, (_, i) => `value_${i} = ${i}`).join('\n') + '\n</hick:file>\n<hick:container name="shell" image="alpine:3.20" />\n<hick:exec container="shell">true</hick:exec>\n';
  const response = await page.request.post('/api/projects/local/docs', { data: { path: name, source } });
  expect(response.ok()).toBe(true);
  const doc = await response.json();
  await page.goto(`/#/docs/${doc.id}`);
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name, exact: true }).click();
  await expect(center(page).locator('.cm-content')).toContainText('value_0');
  return { id: doc.id, source, name };
}
</storycode:copy>
<a id="f-b862abcd8a87-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b862abcd8a87-expression_statement-2">
test.beforeEach(async ({ page }) => {
  test.skip(!process.env.HICKORY_SPATIAL_URL, 'Use just test-editor-density-ui.');
  await page.goto(process.env.HICKORY_SPATIAL_URL!);
  await expect(page.getByRole('tablist', { name: 'Center views' })).toBeVisible();
});

</storycode:copy>
<a id="f-b862abcd8a87-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-b862abcd8a87-expression_statement-3">
test('header contains actions and view information; ruler and action rail remain available', async ({ page }) => {
  const doc = await note(page, 'density-header.md');
  const header = center(page).locator('.spatial-view-header');
  await expect(header.getByRole('button', { name: 'More actions for this document' })).toBeVisible();
  await expect(center(page).locator('.debug-block > .surface-info')).toHaveCount(0);
  await expect(center(page).locator('.debug-block > .doc-tab-toolbar')).toHaveCount(0);
  await expect(center(page).getByRole('slider')).toBeVisible();
  await expect(center(page).locator('.cm-right-rail')).toBeVisible();
  await expect(center(page).locator('.cm-card-rail')).toBeVisible();
  await header.getByRole('button', { name: /About Document/ }).click();
  await expect(header.locator('.surface-info p')).toContainText('Edits save automatically');
  await header.getByRole('button', { name: /About Document/ }).click();
  await header.getByRole('button', { name: 'More actions for this document' }).click();
  await expect(page.getByRole('menuitem', { name: 'Run', exact: true })).toBeVisible();
  await expect(page.getByRole('menuitem', { name: 'Test', exact: true })).toBeVisible();
  await page.keyboard.press('Escape');
  const api = await page.request.get(`/api/docs/${doc.id}`);
  expect((await api.json()).source).toBe(doc.source);
  await page.screenshot({ path: '/tmp/hickory-compact-editor.png' });
});

</storycode:copy>
<a id="f-b862abcd8a87-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-b862abcd8a87-expression_statement-4">
test('compact spacing shows more rows without shrinking text; Files touch targets are optional', async ({ page }) => {
  const doc = await note(page, 'density-spacing.md');
  const row = center(page).locator('.cm-line').filter({ hasText: /^value_0 =/ }).first();
  const compact = { height: await height(row), font: await font(row) };
  const topPadding = await center(page).locator('.cm-content').evaluate(el => parseFloat(getComputedStyle(el).paddingTop));
  expect(topPadding).toBeLessThanOrEqual(8);
  await page.goto('/#/settings');
  await page.getByRole('button', { name: 'Comfortable', exact: true }).click();
  await page.goto(`/#/docs/${doc.id}`);
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: doc.name, exact: true }).click();
  await expect(row).toBeVisible();
  expect(await height(row)).toBeGreaterThan(compact.height);
  expect(await font(row)).toBe(compact.font);
  await page.goto('/#/settings');
  await page.getByRole('button', { name: 'Compact', exact: true }).click();
  await page.goto(`/#/docs/${doc.id}`);
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: doc.name, exact: true }).click();
  await showFiles(page);
  const file = page.locator('[data-spatial-files=left] .filesystem-editor .cm-line').first();
  expect(await height(file)).toBeLessThan(28);
  await (await setting(page, 'Large Files touch targets')).check();
  await page.goto(`/#/docs/${doc.id}`);
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: doc.name, exact: true }).click();
  await showFiles(page);
  expect(await height(file)).toBeGreaterThanOrEqual(36);
});

</storycode:copy>
<a id="f-b862abcd8a87-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-b862abcd8a87-expression_statement-5">
test('palette takes no resting space and Escape preserves text, selection and focus', async ({ page }) => {
  await note(page, 'density-palette.md');
  const content = center(page).locator('.cm-content');
  await content.click(); await content.press('ControlOrMeta+Home');
  await content.press('ArrowRight'); await content.press('Shift+ArrowRight');
  const selected = await page.evaluate(() => window.getSelection()?.toString());
  const before = await center(page).boundingBox();
  await expect(page.getByRole('search')).toHaveCount(0);
  await content.press('ControlOrMeta+p');
  const query = page.getByRole('textbox', { name: 'Search open files and folder' });
  await expect(query).toBeFocused();
  await query.press('Escape');
  await expect(page.getByRole('search')).toHaveCount(0);
  await expect(content).toBeFocused();
  expect(await page.evaluate(() => window.getSelection()?.toString())).toBe(selected);
  expect(await center(page).boundingBox()).toEqual(before);
  await center(page).getByRole('button', { name: 'Open search' }).click();
  await expect(query).toBeFocused();
  await query.fill('>Settings');
  await page.getByRole('option', { name: /Settings/ }).click();
  await expect(page.getByRole('group', { name: 'Editor density' })).toBeVisible();
});

</storycode:copy>
<a id="f-b862abcd8a87-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-b862abcd8a87-expression_statement-6">
test('tab zoom scales document and code text but leaves the header fixed; right numbers remember their toggle', async ({ page }) => {
  const doc = await note(page, 'density-zoom.md');
  const content = center(page).locator('.cm-content');
  const initial = await font(content), headerHeight = await height(center(page).locator('.spatial-view-header'));
  await content.click(); await content.press('ControlOrMeta+Alt+-');
  await expect.poll(() => font(content)).toBeLessThan(initial);
  expect(await height(center(page).locator('.spatial-view-header'))).toBe(headerHeight);
  await content.press('ControlOrMeta+Alt+0');
  await expect.poll(() => font(content)).toBe(initial);
  await (await setting(page, 'Right-hand line numbers')).uncheck();
  await page.goto(`/#/docs/${doc.id}`);
  await page.getByRole("tablist", { name: "Center views" }).getByRole("tab", { name: doc.name, exact: true }).click();
  await expect(center(page).locator('.cm-right-rail')).toBeHidden();
  await expect(center(page).locator('.cm-card-rail')).toBeVisible();
  await expect(center(page).getByRole('slider')).toBeVisible();
  await page.reload();
  await expect(center(page).locator('.cm-right-rail')).toBeHidden();
  expect((await (await page.request.get(`/api/docs/${doc.id}`)).json()).source).toBe(doc.source);
  // The same tab zoom applies to an ordinary code file, not only documents.
  const created = await page.request.post('/api/files/op', { data: { op: 'create', path: 'zoom-code.py' } });
  expect(created.ok()).toBe(true);
  const saved = await page.request.put('/api/file', { data: { path: 'zoom-code.py', content: 'print(42)\n', force: true } });
  expect(saved.ok()).toBe(true);
  await page.reload();
  await showFiles(page);
  await page.locator('[data-spatial-files=left] [data-tree-path="zoom-code.py"]').dblclick();
  await page.getByRole('tablist', { name: 'Center views' }).getByRole('tab', { name: 'zoom-code.py', exact: true }).click();
  const code = center(page).locator('.cm-content');
  await expect(code).toContainText('print(42)');
  await center(page).getByLabel('More actions for this file').click();
  await expect(center(page).getByRole('button', { name: /^Debug / })).toBeVisible();
  await center(page).getByLabel('More actions for this file').click();
  const original = await font(code);
  await code.click(); await code.press('ControlOrMeta+Alt+-');
  await expect.poll(() => font(code)).toBeLessThan(original);
});

</storycode:copy>
<a id="f-b862abcd8a87-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-b862abcd8a87-expression_statement-7">
test('an ordinary note reserves no empty action column and adding a cell restores its actions', async ({ page }) => {
  const response = await page.request.post('/api/projects/local/docs', { data: { path: 'empty-rail.md', source: '# Ordinary note\n' } });
  expect(response.ok()).toBe(true);
  const doc = await response.json();
  await page.goto(`/#/docs/${doc.id}`);
  await page.getByRole('tablist', { name: 'Center views' }).getByRole('tab', { name: 'empty-rail.md', exact: true }).click();
  const rail = center(page).locator('.cm-card-rail');
  await expect(rail).toHaveAttribute('data-empty', 'true');
  expect((await rail.boundingBox())?.width).toBe(0);
  await expect(center(page).locator('.cm-right-rail')).toBeVisible();
  const content = center(page).locator('.cm-content');
  await content.click(); await content.press('ControlOrMeta+End');
  await content.pressSequentially('\n<hick:container name="shell" image="alpine:3.20" />\n<hick:exec container="shell">true</hick:exec>');
  await expect(rail).toHaveAttribute('data-empty', 'false');
  expect((await rail.boundingBox())!.width).toBeGreaterThan(0);
  await expect(rail.getByRole('button', { name: /click to run it/ })).toBeVisible();
});
</storycode:copy>

### Assemble `apps/web/e2e/editor-density.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/editor-density.spec.ts"><storycode:paste select="#f-b862abcd8a87-expression_statement"/><storycode:paste select="#f-b862abcd8a87-center"/><storycode:paste select="#f-b862abcd8a87-font"/><storycode:paste select="#f-b862abcd8a87-height"/><storycode:paste select="#f-b862abcd8a87-showFiles"/><storycode:paste select="#f-b862abcd8a87-setting"/><storycode:paste select="#f-b862abcd8a87-note"/><storycode:paste select="#f-b862abcd8a87-expression_statement-2"/><storycode:paste select="#f-b862abcd8a87-expression_statement-3"/><storycode:paste select="#f-b862abcd8a87-expression_statement-4"/><storycode:paste select="#f-b862abcd8a87-expression_statement-5"/><storycode:paste select="#f-b862abcd8a87-expression_statement-6"/><storycode:paste select="#f-b862abcd8a87-expression_statement-7"/></storycode:file>

<a id="output-a4f9707ac652"></a>
## `apps/web/src/editor/chrome.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](contract.md).

<a id="f-a4f9707ac652-ChromeKind"></a>
### `ChromeKind`

One set of editor chrome, for every editor in the app.
This exists because of a bug that was really a structural mistake. The
gutters, the cursor, the selection, and the find panel were styled by rules
scoped to `.document-editor` — so a `.md` document had a dark gutter with
light numbers, and every OTHER editor (a `.py` file, a `.md` file, a woven
output) silently fell through to CodeMirror's built-in theme, which is
light. A white gutter with dark numbers, in a dark window.
Scoping the chrome to one editor was the mistake; the white gutter was only
how it showed up. So the chrome is an EXTENSION now, not a stylesheet
selector: every editor loads it, and an editor that forgets to has no
chrome at all rather than quietly inheriting somebody else's.
## Why a CodeMirror theme rather than more CSS
CodeMirror's base theme is injected into the document with its own
specificity, and a plain rule in styles.css loses to it in the places that
matter most (`.cm-panels`, `.cm-tooltip`, the selection layer). A theme
extension is emitted through the same mechanism at the same specificity, so
it wins where a rule would not — and it travels with the extension list,
which is the property that makes "every editor looks the same" true by
construction instead of by discipline.
## The two kinds
A `.md` document is prose first: proportional type, generous leading,
because most of what is in it is sentences. Everything else is code, and
it is drawn the way the SAME code looks when it is inside a document — the
sunken ground and the accent rule of a `hick:file` body (see `.cm-file-line`
in styles.css). That is the point: opening `orders.py` and reading the
`<hick:file path="orders.py">` block that writes it should not feel like
two different programs.

<storycode:copy id="f-a4f9707ac652-ChromeKind">
// One set of editor chrome, for every editor in the app.
//
// This exists because of a bug that was really a structural mistake. The
// gutters, the cursor, the selection, and the find panel were styled by rules
// scoped to `.document-editor` — so a `.md` document had a dark gutter with
// light numbers, and every OTHER editor (a `.py` file, a `.md` file, a woven
// output) silently fell through to CodeMirror's built-in theme, which is
// light. A white gutter with dark numbers, in a dark window.
//
// Scoping the chrome to one editor was the mistake; the white gutter was only
// how it showed up. So the chrome is an EXTENSION now, not a stylesheet
// selector: every editor loads it, and an editor that forgets to has no
// chrome at all rather than quietly inheriting somebody else's.
//
// ## Why a CodeMirror theme rather than more CSS
//
// CodeMirror's base theme is injected into the document with its own
// specificity, and a plain rule in styles.css loses to it in the places that
// matter most (`.cm-panels`, `.cm-tooltip`, the selection layer). A theme
// extension is emitted through the same mechanism at the same specificity, so
// it wins where a rule would not — and it travels with the extension list,
// which is the property that makes "every editor looks the same" true by
// construction instead of by discipline.
//
// ## The two kinds
//
// A `.md` document is prose first: proportional type, generous leading,
// because most of what is in it is sentences. Everything else is code, and
// it is drawn the way the SAME code looks when it is inside a document — the
// sunken ground and the accent rule of a `hick:file` body (see `.cm-file-line`
// in styles.css). That is the point: opening `orders.py` and reading the
// `<hick:file path="orders.py">` block that writes it should not feel like
// two different programs.

import { EditorView } from "@codemirror/view";
import type { Extension } from "@codemirror/state";

/** Which of the two looks an editor wears. */
export type ChromeKind = "document" | "code";

</storycode:copy>
<a id="f-a4f9707ac652-editorChrome"></a>
### `editorChrome`

The shared chrome.
Everything here reads a token, never a literal colour, so all three themes
follow without a second definition.

<storycode:copy id="f-a4f9707ac652-editorChrome">
/**
 * The shared chrome.
 *
 * Everything here reads a token, never a literal colour, so all three themes
 * follow without a second definition.
 */
export function editorChrome(kind: ChromeKind): Extension {
  const code = kind === "code";
  return EditorView.theme({
    "&": {
      color: "var(--fg)",
      backgroundColor: "transparent",
      // The height belongs to the pane; an editor that sized itself would
      // fight the shell's grid.
      height: "100%",
    },
    "&.cm-focused": { outline: "none" },
    ".cm-scroller": {
      fontFamily: code
        ? 'ui-monospace, "SF Mono", Menlo, Consolas, monospace'
        : 'system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", sans-serif',
      lineHeight: code ? "var(--editor-code-leading, 1.4)" : "var(--editor-document-leading, 1.45)",
    },
    ".cm-content": { "--cm-pad-x": "var(--editor-pad-x, .75rem)", caretColor: "var(--fg)", padding: "var(--editor-pad-top, .5rem) var(--editor-pad-x, .75rem) var(--editor-pad-bottom, .75rem)" },

    // -- the gutters, which is where this started ---------------------------
    ".cm-gutters": {
      backgroundColor: "transparent",
      color: "var(--fg-muted)",
      border: "none",
      // The numbers are always monospace, even beside proportional prose:
      // a column of digits that does not line up is not a column.
      fontFamily: 'ui-monospace, "SF Mono", Menlo, Consolas, monospace',
      fontSize: "0.78em",
    },
    ".cm-lineNumbers .cm-gutterElement": { padding: "0 0.5em 0 var(--editor-number-pad, .5em)" },
    ".cm-activeLineGutter": {
      backgroundColor: "transparent",
      color: "var(--fg)",
    },
    ".cm-foldGutter .cm-gutterElement": {
      minWidth: "1.5rem",
      padding: "0 0.25rem 0 0.6rem",
    },

    // -- caret, selection, active line --------------------------------------
    ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--fg)" },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection": {
      backgroundColor: "var(--select-bg, rgba(110, 160, 255, 0.28))",
    },
    ".cm-activeLine": { backgroundColor: "transparent" },
    ".cm-selectionMatch": {
      backgroundColor: "var(--select-bg, rgba(110, 160, 255, 0.18))",
    },

    // -- the find panel, which was the other place the light theme leaked ---
    ".cm-panels": {
      backgroundColor: "var(--bg-raised)",
      color: "var(--fg)",
      borderBottom: "1px solid var(--border)",
      fontFamily: 'system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
      fontSize: "0.8rem",
    },
    ".cm-panels.cm-panels-bottom": {
      borderTop: "1px solid var(--border)",
      borderBottom: "none",
    },
    ".cm-panel input, .cm-panel button, .cm-panel select": {
      backgroundColor: "var(--bg)",
      color: "var(--fg)",
      border: "1px solid var(--border)",
      borderRadius: "4px",
      padding: "0.1rem 0.35rem",
      font: "inherit",
    },
    ".cm-panel button:hover": { borderColor: "var(--accent)" },
    ".cm-panel label": { color: "var(--fg-muted)" },
    ".cm-searchMatch": {
      backgroundColor: "var(--search-hit, rgba(217, 161, 63, 0.32))",
    },
    ".cm-searchMatch.cm-searchMatch-selected": {
      backgroundColor: "var(--search-hit-on, rgba(217, 161, 63, 0.6))",
    },

    // -- tooltips and completion --------------------------------------------
    ".cm-tooltip": {
      backgroundColor: "var(--bg-raised)",
      color: "var(--fg)",
      border: "1px solid var(--border)",
      borderRadius: "6px",
    },
    ".cm-tooltip.cm-tooltip-autocomplete > ul > li[aria-selected]": {
      backgroundColor: "var(--accent)",
      color: "var(--bg)",
    },
  });
}
</storycode:copy>

### Assemble `apps/web/src/editor/chrome.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/chrome.ts"><storycode:paste select="#f-a4f9707ac652-ChromeKind"/><storycode:paste select="#f-a4f9707ac652-editorChrome"/></storycode:file>

<a id="output-572d4c31cabc"></a>
## `apps/web/src/editor/density.css`

Density changes spacing, never source bytes or the chosen font size.

Related contracts: [The editor keeps its space](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/editor/density.css">
/* Density changes spacing, never source bytes or the chosen font size. */
:root {
  --editor-code-leading: 1.4;
  --editor-document-leading: 1.45;
  --editor-pad-top: .5rem;
  --editor-pad-x: .75rem;
  --editor-pad-bottom: .75rem;
  --editor-number-pad: .5em;
  --workspace-padding: .25rem;
}
:root[data-editor-density=comfortable] {
  --editor-code-leading: 1.5;
  --editor-document-leading: 1.65;
  --editor-pad-top: 2rem;
  --editor-pad-x: 2.5rem;
  --editor-pad-bottom: 3rem;
  --editor-number-pad: 1em;
  --workspace-padding: 1rem;
}
:root[data-touch-targets=true] .spatial-shell .spatial-files .filesystem-editor .cm-line,
:root[data-touch-targets=true] .spatial-shell .spatial-files .filesystem-editor .cm-gutterElement { min-height: 36px; line-height: 36px; }

:root[data-right-line-numbers=false] .cm-right-rail { display: none; }

:root .doc-page.wide-mode .doc-main { padding: var(--workspace-padding, .25rem); }

.cm-card-rail[data-empty=true] { width: 0; border-left: 0; }
</storycode:entry>

<a id="output-329924238378"></a>
## `apps/web/src/lib/editorAppearance.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](contract.md).

<a id="f-329924238378-EditorDensity"></a>
### `EditorDensity`

<storycode:copy id="f-329924238378-EditorDensity">
import "../editor/density.css";

export type EditorDensity = "compact" | "comfortable";
</storycode:copy>
<a id="f-329924238378-EditorAppearance"></a>
### `EditorAppearance`

<storycode:copy id="f-329924238378-EditorAppearance">
export interface EditorAppearance { density: EditorDensity; touchTargets: boolean; rightLineNumbers: boolean }
</storycode:copy>
<a id="f-329924238378-EDITOR_APPEARANCE_KEY"></a>
### `EDITOR_APPEARANCE_KEY`

<storycode:copy id="f-329924238378-EDITOR_APPEARANCE_KEY">
export const EDITOR_APPEARANCE_KEY = "hickory.editorAppearance";

</storycode:copy>
<a id="f-329924238378-loadEditorAppearance"></a>
### `loadEditorAppearance`

<storycode:copy id="f-329924238378-loadEditorAppearance">
export function loadEditorAppearance(storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage): EditorAppearance {
  try {
    const value = JSON.parse(storage?.getItem(EDITOR_APPEARANCE_KEY) ?? "null");
    return { density: value?.density === "comfortable" ? "comfortable" : "compact", touchTargets: value?.touchTargets === true, rightLineNumbers: value?.rightLineNumbers !== false };
  } catch { return { density: "compact", touchTargets: false, rightLineNumbers: true }; }
}
</storycode:copy>
<a id="f-329924238378-applyEditorAppearance"></a>
### `applyEditorAppearance`

<storycode:copy id="f-329924238378-applyEditorAppearance">
export function applyEditorAppearance(value: EditorAppearance, root: HTMLElement = document.documentElement) {
  root.dataset.editorDensity = value.density;
  root.dataset.rightLineNumbers = String(value.rightLineNumbers);
  root.dataset.touchTargets = String(value.touchTargets);
}
</storycode:copy>
<a id="f-329924238378-saveEditorAppearance"></a>
### `saveEditorAppearance`

<storycode:copy id="f-329924238378-saveEditorAppearance">
export function saveEditorAppearance(value: EditorAppearance) {
  try { localStorage.setItem(EDITOR_APPEARANCE_KEY, JSON.stringify(value)); } catch { /* Optional presentation storage. */ }
  applyEditorAppearance(value);
}
</storycode:copy>

### Assemble `apps/web/src/lib/editorAppearance.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/editorAppearance.ts"><storycode:paste select="#f-329924238378-EditorDensity"/><storycode:paste select="#f-329924238378-EditorAppearance"/><storycode:paste select="#f-329924238378-EDITOR_APPEARANCE_KEY"/><storycode:paste select="#f-329924238378-loadEditorAppearance"/><storycode:paste select="#f-329924238378-applyEditorAppearance"/><storycode:paste select="#f-329924238378-saveEditorAppearance"/></storycode:file>

<a id="output-9a0b2ebc41a8"></a>
## `apps/web/src/shell/CommandBar.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Settings preserves the open workspace](../settings-preserves-the-open-workspace/contract.md), [The editor keeps its space](contract.md).

<a id="f-9a0b2ebc41a8-OPEN_COMMAND_BAR"></a>
### `OPEN_COMMAND_BAR`

The search box across the top of the window.
It is one field, and it is deliberately not four. A window that grows a
separate control for "open a file", "find a symbol", "search the folder",
and "run a command" has four things to learn and four places to look; one
field that understands a prefix has one, and the prefixes are the same ones
every editor already taught people.
(nothing)   — search open files and folder contents, ranked
`>`         — a command
`?`         — search open files and folder contents, ranked (an explicit spelling
for people who learned the earlier prefix)
`:`         — a line number in the file that is open
The bar itself is dumb: it parses the prefix, asks whoever mounted it for
candidates, and reports the pick. What any of those things MEAN belongs to
the workspace, which is the only thing that knows what is open.

<storycode:copy id="f-9a0b2ebc41a8-OPEN_COMMAND_BAR">
// The search box across the top of the window.
//
// It is one field, and it is deliberately not four. A window that grows a
// separate control for "open a file", "find a symbol", "search the folder",
// and "run a command" has four things to learn and four places to look; one
// field that understands a prefix has one, and the prefixes are the same ones
// every editor already taught people.
//
//   (nothing)   — search open files and folder contents, ranked
//   `>`         — a command
//   `?`         — search open files and folder contents, ranked (an explicit spelling
//                 for people who learned the earlier prefix)
//   `:`         — a line number in the file that is open
//
// The bar itself is dumb: it parses the prefix, asks whoever mounted it for
// candidates, and reports the pick. What any of those things MEAN belongs to
// the workspace, which is the only thing that knows what is open.

import { isAction } from "../lib/keymap";
import "./commandPalette.css";
import { useEffect, useId, useLayoutEffect, useMemo, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField } from "../lib/uiActions";

const OPEN_COMMAND_BAR = "hickory-open-command-bar";
</storycode:copy>
<a id="f-9a0b2ebc41a8-openCommandBar"></a>
### `openCommandBar`

<storycode:copy id="f-9a0b2ebc41a8-openCommandBar">
export function openCommandBar(commands = false) {
  window.dispatchEvent(new CustomEvent(OPEN_COMMAND_BAR, { detail: commands }));
}

</storycode:copy>
<a id="f-9a0b2ebc41a8-CommandMode"></a>
### `CommandMode`

<storycode:copy id="f-9a0b2ebc41a8-CommandMode">
/** What a query is asking for. */
export type CommandMode = "files" | "command" | "content" | "line";

</storycode:copy>
<a id="f-9a0b2ebc41a8-CommandItem"></a>
### `CommandItem`

<storycode:copy id="f-9a0b2ebc41a8-CommandItem">
export interface CommandItem {
  id: string;
  label: string;
  /** The dim second line — a path, a match, a keyboard shortcut. */
  detail?: string;
  run: () => void;
  /** Command-mode callbacks need explicit policy review; navigation is separate. */
  agentAllowed?: boolean;
}

</storycode:copy>
<a id="f-9a0b2ebc41a8-parseQuery"></a>
### `parseQuery`

The mode a raw query is in, and the query with its prefix removed.

<storycode:copy id="f-9a0b2ebc41a8-parseQuery">
/** The mode a raw query is in, and the query with its prefix removed. */
export function parseQuery(raw: string): { mode: CommandMode; term: string } {
  const trimmed = raw.trimStart();
  if (trimmed.startsWith(">")) return { mode: "command", term: trimmed.slice(1).trim() };
  if (trimmed.startsWith("?")) return { mode: "content", term: trimmed.slice(1).trim() };
  if (trimmed.startsWith(":")) return { mode: "line", term: trimmed.slice(1).trim() };
  return { mode: "content", term: trimmed.trim() };
}

</storycode:copy>
<a id="f-9a0b2ebc41a8-modeHint"></a>
### `modeHint`

What the bar says it will do, so the prefixes are discoverable without a
help page.

<storycode:copy id="f-9a0b2ebc41a8-modeHint">
/** What the bar says it will do, so the prefixes are discoverable without a
 * help page. */
export function modeHint(mode: CommandMode): string {
  switch (mode) {
    case "command":
      return "Commands";
    case "content":
      return "Search open files and folder";
    case "line":
      return "Go to line";
    case "files":
      return "Files";
  }
}

</storycode:copy>
<a id="f-9a0b2ebc41a8-CommandBar"></a>
### `CommandBar`

<storycode:copy id="f-9a0b2ebc41a8-CommandBar">
export function CommandBar({
  candidates,
  placeholder = "Search",
  overlay = false,
}: {
  /** Answers for a query. Async because searching contents is a request. */
  candidates: (mode: CommandMode, term: string) => Promise<CommandItem[]> | CommandItem[];
  placeholder?: string;
  overlay?: boolean;
}) {
  const [raw, setRaw] = useState("");
  const [open, setOpen] = useState(false);
  const [results, setResults] = useState<{ raw: string; items: CommandItem[] } | null>(null);
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listId = useId();
  const items = results?.raw === raw ? results.items : [];
  const loading = results?.raw !== raw;
  const previousFocus = useRef<HTMLElement | null>(null);
  const openRef = useRef(open); openRef.current = open;
  const { mode, term } = useMemo(() => parseQuery(raw), [raw]);
  const reveal = (commands: boolean) => {
      if (!openRef.current) previousFocus.current = document.activeElement as HTMLElement | null;
      setRaw(commands ? ">" : "");
      setOpen(true);
      inputRef.current?.focus();
      inputRef.current?.select();
  };
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (inputRef.current?.closest("[hidden]:not(.command-palette)")) return;
      const commands = isAction(event, "search.commands");
      if (!commands && !isAction(event, "search.quickOpen")) return;
      event.preventDefault();
      reveal(commands);
    };
    const onOpen = (event: Event) => reveal((event as CustomEvent<boolean>).detail === true);
    window.addEventListener("keydown", onKey);
    window.addEventListener(OPEN_COMMAND_BAR, onOpen);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener(OPEN_COMMAND_BAR, onOpen);
    };
  }, []);
  useLayoutEffect(() => {
    if (overlay && open) { inputRef.current?.focus(); inputRef.current?.select(); }
  }, [overlay, open]);
  const dismiss = () => {
    setOpen(false);
    setRaw("");
    if (overlay && previousFocus.current?.isConnected) previousFocus.current.focus();
    else inputRef.current?.blur();
  };

  useEffect(() => {
    if (!open) return;
    setResults(null);
    setSelected(0);
    let live = true;
    const timer = window.setTimeout(() => {
      void Promise.resolve(candidates(mode, term)).then(
        (found) => {
          if (!live) return;
          setResults({ raw, items: found });
          setSelected(0);
        },
        () => live && setResults({ raw, items: [] }),
      );
      // A keystroke is not a question; a pause is. Short, because this is the
      // control people type into fastest.
    }, 120);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
  }, [open, raw, mode, term, candidates]);

  useEffect(() => {
    if (open) document.getElementById(`${listId}-${selected}`)?.scrollIntoView?.({ block: "nearest" });
  }, [open, selected, results, listId]);

  const choose = (item: CommandItem | undefined) => {
    if (!item) return;
    dismiss();
    return item.run();
  };
  const parent = `command-search:${listId}`;
  const revision = useUiRevision(() => [raw, open, selected, results?.raw, items.map(item => [item.id, item.label, item.detail, item.agentAllowed])]);
  const commands: { key: string; command: UiCommand }[] = [];
  function add(id: string, label: string, parameters: Record<string, UiField>, execute: UiCommand["execute"], enabled = true, group = parent) {
    commands.push({ key: `${parent}/${id}`, command: { label, parent: group, subject: parent, parameters, prerequisites: ["Current query and returned candidate identity; command-mode policy applies"], read: () => ({ revision: revision(), enabled, reason: enabled ? undefined : "Wait for the current search response" }), execute } });
  }
  add("visibility", "Set command search visibility", { open: { type: "boolean" }, commands: { type: "boolean" } }, async args => { if (args.open) reveal(Boolean(args.commands)); else dismiss(); return { status: "completed", value: { open: Boolean(args.open) } }; }, true, "command-search-disclosures");
  if (open) {
    add("input", "Set command search query", { query: { type: "string", maxLength: 8192 } }, async args => { setRaw(String(args.query)); return { status: "completed", value: { dispatch: "query requested", result: "Observe the current candidates before choosing" } }; });
    add("read", "Read command search bytes", { offset: { type: "number", minimum: 0, maximum: 100000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ query: raw, mode, loading, selected, items: items.map(item => ({ id: item.id, label: item.label, detail: item.detail, agent_allowed: mode !== "command" || item.agentAllowed === true })) }));
      const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds current command search" };
      return { status: "completed", value: { encoding: "JSON encoded as UTF-8 bytes", bytes: [...bytes.slice(offset, end)], span: [offset, end], total_bytes: bytes.length, next: end < bytes.length ? end : null } };
    });
    add("select", "Select command search candidate", { index: { type: "number", minimum: 0, maximum: 1000000, integer: true } }, async args => { if (!items[Number(args.index)]) return { status: "unavailable", reason: "Index does not belong to this query's results" }; setSelected(Number(args.index)); return { status: "completed", value: { selected: args.index } }; }, !loading);
    add("choose", "Choose command search candidate", { id: { type: "string", maxLength: 8192 } }, async (args, actor) => {
      const item = items.find(item => item.id === args.id);
      if (!item) return { status: "unavailable", reason: "Candidate does not belong to this query's current result" };
      if (actor.kind === "agent" && mode === "command" && item.agentAllowed !== true) return { status: "unavailable", reason: "This palette callback has no agent policy adapter. For document runs use the source-bound Run/Verify saved document controls; other command adapters remain integration work" };
      await choose(item); return { status: "completed", value: { dispatch: "requested", item: item.id, result: "Observe the actual route, dialog or source-bound job receipt" } };
    }, !loading);
  }
  useUiCommands(commands);

  return (
    <div className={overlay ? "command-palette" : "command-bar"} hidden={overlay && !open} role="search"
      onMouseDown={event => { if (overlay && event.target === event.currentTarget) dismiss(); }}>
      <div className={overlay ? "command-bar command-bar--overlay" : undefined}>
      <input
        ref={inputRef}
        className="command-bar__input"
        type="text"
        value={raw}
        placeholder={placeholder}
        aria-label={modeHint(mode)}
        aria-expanded={open}
        aria-controls={open ? listId : undefined}
        aria-activedescendant={items[selected] ? `${listId}-${selected}` : undefined}
        onFocus={() => setOpen(true)}
        // Result buttons keep the input focused on mousedown. An overlay
        // closes immediately when focus leaves it, so reopening cannot be
        // cancelled by a timeout from its previous dismissal.
        onBlur={(event) => {
          if (overlay) {
            if (!event.currentTarget.parentElement?.contains(event.relatedTarget as Node | null)) setOpen(false);
          } else window.setTimeout(() => setOpen(false), 120);
        }}
        onChange={(event) => {
          setRaw(event.target.value);
          setOpen(true);
        }}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.preventDefault();
            dismiss();
          } else if (event.key === "ArrowDown") {
            event.preventDefault();
            setSelected((i) => Math.max(0, Math.min(i + 1, items.length - 1)));
          } else if (event.key === "ArrowUp") {
            event.preventDefault();
            setSelected((i) => Math.max(i - 1, 0));
          } else if (event.key === "Enter") {
            event.preventDefault();
            choose(items[selected]);
          }
        }}
      />
      {open && (
        <div className="command-bar__panel">
          <p className="command-bar__hint">{modeHint(mode)}</p>
          {items.length === 0 ? (
            <p className="command-bar__empty muted">
              {loading ? "Searching…" : term ? "Nothing matches." : "Start typing."}
            </p>
          ) : (
            <ul className="command-bar__list" role="listbox" id={listId} aria-label={modeHint(mode)}>
              {items.map((item, index) => (
                <li key={item.id}>
                  <button
                    type="button"
                    role="option"
                    id={`${listId}-${index}`}
                    aria-selected={index === selected}
                    className={`command-bar__item${index === selected ? " command-bar__item--on" : ""}`}
                    onMouseEnter={() => setSelected(index)}
                    onMouseDown={(event) => event.preventDefault()}
                    onClick={() => choose(item)}
                  >
                    <span className="command-bar__label">{item.label}</span>
                    {item.detail && (
                      <span className="command-bar__detail mono">{item.detail}</span>
                    )}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
      </div>
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/shell/CommandBar.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/CommandBar.tsx"><storycode:paste select="#f-9a0b2ebc41a8-OPEN_COMMAND_BAR"/><storycode:paste select="#f-9a0b2ebc41a8-openCommandBar"/><storycode:paste select="#f-9a0b2ebc41a8-CommandMode"/><storycode:paste select="#f-9a0b2ebc41a8-CommandItem"/><storycode:paste select="#f-9a0b2ebc41a8-parseQuery"/><storycode:paste select="#f-9a0b2ebc41a8-modeHint"/><storycode:paste select="#f-9a0b2ebc41a8-CommandBar"/></storycode:file>

<a id="output-abce6d0421a3"></a>
## `apps/web/src/shell/PaneHeader.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](contract.md).

<a id="f-abce6d0421a3-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-abce6d0421a3-expression_statement">
import { afterEach, expect, it } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { PaneHeaderProvider, PaneHeaderSlots, PaneHeaderContent } from "./PaneHeader";
import { SurfaceInfo } from "../components/SurfaceInfo";

afterEach(cleanup);
</storycode:copy>
<a id="f-abce6d0421a3-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-abce6d0421a3-expression_statement-2">
// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
it("keeps info and actions in the header without recreating the editing surface", () => {
  function Pane() {
    const [label, setLabel] = useState("old");
    return <PaneHeaderProvider><header data-testid="header"><PaneHeaderSlots /></header><main>
      <SurfaceInfo name="Note" editable destination="Saves to note.md" />
      <PaneHeaderContent part="actions"><button onClick={() => setLabel("new")}>{label}</button></PaneHeaderContent>
      <textarea aria-label="Source" defaultValue="unchanged" />
    </main></PaneHeaderProvider>;
  }
  render(<Pane />);
  const source = screen.getByRole("textbox") as HTMLTextAreaElement;
  source.setSelectionRange(1, 4);
  expect(screen.getByTestId("header").contains(screen.getByRole("button", { name: "old" }))).toBe(true);
  expect(screen.getByTestId("header").querySelector(".surface-info")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "old" }));
  expect(screen.getByRole("textbox")).toBe(source);
  expect(source.value).toBe("unchanged");
  expect([source.selectionStart, source.selectionEnd]).toEqual([1, 4]);
});
</storycode:copy>

### Assemble `apps/web/src/shell/PaneHeader.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/PaneHeader.test.tsx"><storycode:paste select="#f-abce6d0421a3-expression_statement"/><storycode:paste select="#f-abce6d0421a3-expression_statement-2"/></storycode:file>

<a id="output-376a93f02516"></a>
## `apps/web/src/shell/PaneHeader.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](contract.md).

<a id="f-376a93f02516-Part"></a>
### `Part`

<storycode:copy id="f-376a93f02516-Part">
import { createContext, useContext, useMemo, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import "./paneHeader.css";

type Part = "info" | "actions";
</storycode:copy>
<a id="f-376a93f02516-HeaderSlots"></a>
### `HeaderSlots`

<storycode:copy id="f-376a93f02516-HeaderSlots">
interface HeaderSlots {
  info: HTMLElement | null;
  actions: HTMLElement | null;
  setInfo: (node: HTMLDivElement | null) => void;
  setActions: (node: HTMLDivElement | null) => void;
}
</storycode:copy>
<a id="f-376a93f02516-HeaderContext"></a>
### `HeaderContext`

<storycode:copy id="f-376a93f02516-HeaderContext">
const HeaderContext = createContext<HeaderSlots | null>(null);

</storycode:copy>
<a id="f-376a93f02516-PaneHeaderProvider"></a>
### `PaneHeaderProvider`

Move only chrome into a pane's existing header. Its editor stays mounted.

<storycode:copy id="f-376a93f02516-PaneHeaderProvider">
/** Move only chrome into a pane's existing header. Its editor stays mounted. */
export function PaneHeaderProvider({ children }: { children: ReactNode }) {
  const [info, setInfo] = useState<HTMLDivElement | null>(null);
  const [actions, setActions] = useState<HTMLDivElement | null>(null);
  const slots = useMemo(() => ({ info, actions, setInfo, setActions }), [info, actions]);
  return <HeaderContext.Provider value={slots}>{children}</HeaderContext.Provider>;
}

</storycode:copy>
<a id="f-376a93f02516-PaneHeaderSlots"></a>
### `PaneHeaderSlots`

<storycode:copy id="f-376a93f02516-PaneHeaderSlots">
export function PaneHeaderSlots() {
  const header = useContext(HeaderContext);
  return <div className="pane-header-slots" onPointerDown={event => event.stopPropagation()}>
    <div className="pane-header-info" ref={header?.setInfo} />
    <div className="pane-header-actions" ref={header?.setActions} />
  </div>;
}

</storycode:copy>
<a id="f-376a93f02516-PaneHeaderContent"></a>
### `PaneHeaderContent`

Standalone/embedded surfaces retain their own chrome if there is no shell.

<storycode:copy id="f-376a93f02516-PaneHeaderContent">
/** Standalone/embedded surfaces retain their own chrome if there is no shell. */
export function PaneHeaderContent({ part, children }: { part: Part; children: ReactNode }) {
  const header = useContext(HeaderContext);
  return header?.[part] ? createPortal(children, header[part]!) : <>{children}</>;
}
</storycode:copy>

### Assemble `apps/web/src/shell/PaneHeader.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/PaneHeader.tsx"><storycode:paste select="#f-376a93f02516-Part"/><storycode:paste select="#f-376a93f02516-HeaderSlots"/><storycode:paste select="#f-376a93f02516-HeaderContext"/><storycode:paste select="#f-376a93f02516-PaneHeaderProvider"/><storycode:paste select="#f-376a93f02516-PaneHeaderSlots"/><storycode:paste select="#f-376a93f02516-PaneHeaderContent"/></storycode:file>

<a id="output-c43c71cf0fa9"></a>
## `apps/web/src/shell/commandPalette.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/shell/commandPalette.css">
.command-palette[hidden] { display: none; }
.command-palette { position: fixed; inset: 0; z-index: 500; display: flex; justify-content: center; align-items: flex-start; padding-top: min(12vh, 100px); background: rgba(0, 0, 0, .18); }
.command-bar--overlay { width: min(38rem, calc(100vw - 24px)); padding: 8px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg-raised); box-shadow: var(--shadow); }
.command-bar--overlay .command-bar__input { box-sizing: border-box; text-align: left; }
.command-bar--overlay .command-bar__panel { left: 0; right: 0; }
</storycode:entry>

<a id="output-c09b7ea67dd0"></a>
## `apps/web/src/shell/paneHeader.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/shell/paneHeader.css">
.pane-header-slots { display: flex; align-items: center; flex: none; gap: 4px; }
.pane-header-info, .pane-header-actions { display: flex; align-items: center; min-width: 0; }
.spatial-view-header:has(.pane-header-slots) { overflow: visible; }
.pane-header-slots .doc-tab-toolbar { padding: 0; background: transparent; border: 0; font-size: 12px; }
.pane-header-slots .doc-tab-toolbar .btn { padding: 2px 5px; font-size: 12px; }
.pane-header-slots .surface-info { position: relative; padding: 0; border: 0; font-size: 12px; }
.pane-header-slots .surface-info > summary { padding: 4px; }
.pane-header-slots .surface-info > summary::before { content: "ⓘ"; }
.pane-header-slots .surface-info[open] > summary::before { transform: none; }
.pane-header-slots .surface-info > summary > strong,
.pane-header-slots .surface-info > summary > span { display: none; }
.pane-header-slots .surface-info > p { position: absolute; z-index: 30; right: 0; top: 24px; width: min(320px, 75vw); box-sizing: border-box; margin: 0; padding: 10px; background: var(--bg-raised); color: var(--fg); border: 1px solid var(--border); border-radius: 4px; box-shadow: var(--shadow); white-space: normal; }
.pane-header-slots .save-state { max-width: 8rem; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
</storycode:entry>

<a id="output-721476aea697"></a>
## `apps/web/src/views/useFileTabMenu.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](contract.md).

<a id="f-721476aea697-folder"></a>
### `folder`

<storycode:copy id="f-721476aea697-folder">
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { installMockHandler } from "../api/client";
import type { FolderTree } from "../shell/FolderTreePane";
import { treeMenuItems } from "../shell/treeMenu";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useFileTabMenu } from "./useFileTabMenu";
import type { Tab } from "../shell/layout";

const folder: FolderTree = { root: "notes", root_path: "/notes", file_manager: "Finder", tree: [
  { name: "note.md", path: "note.md", dir: false, doc_id: "doc1" },
  { name: "input.txt", path: "input.txt", dir: false },
  { name: "output.txt", path: "output.txt", dir: false, generated_by: "doc1" },
] };
</storycode:copy>
<a id="f-721476aea697-opened"></a>
### `opened`

<storycode:copy id="f-721476aea697-opened">
const opened = vi.fn();
</storycode:copy>
<a id="f-721476aea697-Harness"></a>
### `Harness`

<storycode:copy id="f-721476aea697-Harness">
function Harness({ kind = "file", target = "input.txt" }: { kind?: Tab["kind"]; target?: string }) {
  const { onTabContextMenu, fileTabMenu } = useFileTabMenu([folder], "doc1", new Map(), opened, [{ id: "tab1", kind, target }]);
  return <><button onContextMenu={event => onTabContextMenu(event, { id: "tab1", kind, target })}>File tab</button>{fileTabMenu}</>;
}
</storycode:copy>
<a id="f-721476aea697-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-721476aea697-expression_statement">
afterEach(() => { cleanup(); vi.clearAllMocks(); });
</storycode:copy>
<a id="f-721476aea697-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-721476aea697-expression_statement-2">
it("offers every Files menu action and acts on the right-clicked tab", async () => {
  const calls: unknown[] = [];
  installMockHandler(async (method, path, body) => { calls.push([method, path, body]); return {}; });
  render(<Harness />);
  fireEvent.contextMenu(screen.getByText("File tab"));
  const expected = treeMenuItems({ rootPath: "/notes", fileManager: "Finder" }, "input.txt", false, { plainText: true, activeDoc: { path: "note.md", name: "note.md" } });
  expect(screen.getAllByRole("menuitem").map(item => item.textContent)).toEqual(expected.map(item => item.label));
  fireEvent.click(screen.getByRole("menuitem", { name: "Delete" }));
  expect(calls).toEqual([]);
  fireEvent.click(screen.getByRole("button", { name: "Delete" }));
  await waitFor(() => expect(calls).toEqual([["POST", "/api/files/op", { op: "delete", path: "input.txt" }]]));
});
</storycode:copy>
<a id="f-721476aea697-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-721476aea697-expression_statement-3">
it("uses the shared adoption action and opens its resulting document", async () => {
  installMockHandler(async (method, path) => {
    if (method === "POST" && path === "/api/adopt") return { doc_id: "adopted" };
    throw new Error(`unexpected ${method} ${path}`);
  });
  render(<Harness />);
  fireEvent.contextMenu(screen.getByText("File tab"));
  fireEvent.click(screen.getByRole("menuitem", { name: /Make literate/ }));
  await waitFor(() => expect(opened).toHaveBeenCalledWith("adopted"));
});
</storycode:copy>
<a id="f-721476aea697-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-721476aea697-expression_statement-4">
it.each([["document", "note.md"], ["generated", "output.txt"], ["file", "output.txt"]] as const)("does not offer adoption for %s %s", (kind, target) => {
  render(<Harness kind={kind} target={target} />);
  fireEvent.contextMenu(screen.getByText("File tab"));
  expect(screen.queryByRole("menuitem", { name: /Make literate/ })).toBeNull();
  expect(screen.getByRole("menuitem", { name: "Rename…" })).toBeTruthy();
});
</storycode:copy>
<a id="f-721476aea697-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-721476aea697-expression_statement-5">
it("does not give file actions to unsaved or tool tabs", () => {
  render(<Harness kind="untitled" target="draft" />);
  fireEvent.contextMenu(screen.getByText("File tab"));
  expect(screen.queryByRole("menu")).toBeNull();
});

</storycode:copy>
<a id="f-721476aea697-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-721476aea697-expression_statement-6">
// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
it("discloses current file-tab targets, opens the real menu and retires the former tab owner", async () => {
  const registry = new UiActions({ workspace: "p", window: "file-menu" }, () => true, async () => {});
  const mounted = render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "acp:tabs", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
  expect(actions().some(action => action.label === "Delete")).toBe(false);
  expect(await run("Read file tab menu targets", { first: 0, count: 64 })).toMatchObject({ outcome: { value: { tabs: [{ id: "tab1", path: "input.txt" }] } } });
  expect(await run("Open file tab context menu", { tab: "missing" })).toMatchObject({ outcome: { status: "unavailable" } });
  expect(await run("Open file tab context menu", { tab: "tab1" })).toMatchObject({ outcome: { status: "completed" } });
  expect(screen.getByRole("menuitem", { name: "Rename…" })).toBeTruthy();
  const old = actions().find(action => action.label === "Delete")!;
  mounted.rerender(<UiActionContext.Provider value={registry}><Harness target="output.txt" /></UiActionContext.Provider>);
  expect(screen.queryByRole("menu")).toBeNull();
  expect(await registry.act(registry.scope, { id: "acp:tabs", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old" })).not.toMatchObject({ outcome: { status: "completed" } });
  await run("Open file tab context menu", { tab: "tab1" });
  expect(screen.queryByRole("menuitem", { name: /Make literate/ })).toBeNull();
});
</storycode:copy>

### Assemble `apps/web/src/views/useFileTabMenu.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useFileTabMenu.test.tsx"><storycode:paste select="#f-721476aea697-folder"/><storycode:paste select="#f-721476aea697-opened"/><storycode:paste select="#f-721476aea697-Harness"/><storycode:paste select="#f-721476aea697-expression_statement"/><storycode:paste select="#f-721476aea697-expression_statement-2"/><storycode:paste select="#f-721476aea697-expression_statement-3"/><storycode:paste select="#f-721476aea697-expression_statement-4"/><storycode:paste select="#f-721476aea697-expression_statement-5"/><storycode:paste select="#f-721476aea697-expression_statement-6"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/components/SurfaceInfo.tsx](../../../mechanisms/apps/web/src/components/SurfaceInfo.tsx.md#output-6c34dd749b33)
- [apps/web/src/editor/CardRail.tsx](../../../mechanisms/apps/web/src/editor/CardRail.tsx.md#output-f6019c4774f8)
- [apps/web/src/editor/chrome.ts](contract.md#output-a4f9707ac652)
- [apps/web/src/editor/density.css](contract.md#output-572d4c31cabc)
- [apps/web/src/lib/cardRail.ts](../../../mechanisms/apps/web/src/lib/cardRail.ts.md#output-996aefac1e60)
- [apps/web/src/lib/editorAppearance.ts](contract.md#output-329924238378)
- [apps/web/src/lib/keymap.ts](../../../mechanisms/apps/web/src/lib/keymap.ts.md#output-4d0aa846a85d)
- [apps/web/src/shell/CommandBar.tsx](contract.md#output-9a0b2ebc41a8)
- [apps/web/src/shell/PaneHeader.tsx](contract.md#output-376a93f02516)
- [apps/web/src/shell/SpatialShellView.tsx](../../../mechanisms/apps/web/src/shell/SpatialShellView.tsx.md#output-64456f67b44e)
- [apps/web/src/shell/commandPalette.css](contract.md#output-c43c71cf0fa9)
- [apps/web/src/shell/paneHeader.css](contract.md#output-c09b7ea67dd0)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)
- [apps/web/src/views/SettingsView.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-38894484eb8f)
- [apps/web/src/views/workspaceTabs.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-b5a48fd337ae)

### Tests and fixtures

- [apps/web/e2e/editor-density.spec.ts](contract.md#output-b862abcd8a87)
- [apps/web/src/editor/DocumentEditor.test.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.test.tsx.md#output-71bbcfe029bb)
- [apps/web/src/shell/CommandBar.test.tsx](../../../mechanisms/apps/web/src/shell/CommandBar.test.tsx.md#output-f9af94fac1a9)
- [apps/web/src/shell/PaneHeader.test.tsx](contract.md#output-abce6d0421a3)
- [apps/web/src/shell/views.test.tsx](../../../mechanisms/apps/web/src/shell/views.tsx.md#output-003396968e5c)
- [apps/web/src/views/SettingsView.test.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-5bfac230ae5d)
- [apps/web/src/views/useFileTabMenu.test.tsx](contract.md#output-721476aea697)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
