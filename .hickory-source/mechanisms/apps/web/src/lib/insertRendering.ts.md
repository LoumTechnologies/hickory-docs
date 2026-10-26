<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/insertRendering.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-39a9040eedce"></a>
## `apps/web/src/lib/insertRendering.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-39a9040eedce-FieldValues"></a>
### `FieldValues`

<storycode:copy id="f-39a9040eedce-FieldValues">
import type { InsertElement } from "./insertCatalog";

// ---------------------------------------------------------------------------
// Values
// ---------------------------------------------------------------------------

export type FieldValues = Readonly<Record<string, string>>;

</storycode:copy>
<a id="f-39a9040eedce-initialValues"></a>
### `initialValues`

The form's starting values: each field's `value`, else empty.

<storycode:copy id="f-39a9040eedce-initialValues">
/** The form's starting values: each field's `value`, else empty. */
export function initialValues(element: InsertElement): Record<string, string> {
  const values: Record<string, string> = {};
  for (const field of element.fields) {
    values[field.name] = field.value ?? (field.choices ? field.choices[0] : "");
  }
  return values;
}

</storycode:copy>
<a id="f-39a9040eedce-normalize"></a>
### `normalize`

Tidy a value the moment it is used, so the person does not have to know the
spelling. Only ever additive — never a silent change of meaning.

<storycode:copy id="f-39a9040eedce-normalize">
/**
 * Tidy a value the moment it is used, so the person does not have to know the
 * spelling. Only ever additive — never a silent change of meaning.
 */
export function normalize(element: InsertElement, name: string, raw: string): string {
  const value = raw.trim();
  if (!value) return value;
  // A selector without its `#` selects nothing and says nothing about why.
  if ((element.id === "paste" && name === "select") || (element.id === "diagram" && name === "asserts")) {
    return value.startsWith("#") || value.startsWith(".") ? value : `#${value}`;
  }
  return value;
}

</storycode:copy>
<a id="f-39a9040eedce-validate"></a>
### `validate`

Why this form cannot be inserted yet, per field. Empty means it can.

<storycode:copy id="f-39a9040eedce-validate">
/** Why this form cannot be inserted yet, per field. Empty means it can. */
export function validate(element: InsertElement, values: FieldValues): Record<string, string> {
  const problems: Record<string, string> = {};
  for (const field of element.fields) {
    const value = normalize(element, field.name, values[field.name] ?? "");
    if (field.required && !value) {
      const what = field.label.toLowerCase();
      const article = /^[aeiou]/.test(what) ? "an" : "a";
      problems[field.name] = `${element.title} needs ${article} ${what}.`;
      continue;
    }
    // hick has no entity escaping, on purpose: what you type is what the file
    // holds. An attribute can be quoted with " or ', so a value may contain
    // either — but not both, and there is no third quoting to fall back on.
    if (value.includes('"') && value.includes("'")) {
      problems[field.name] =
        "hick attributes are never escaped, so a value cannot contain both \" and '. " +
        "Move the text into the element's body, or drop one of the quotes.";
    }
  }
  return problems;
}

</storycode:copy>
<a id="f-39a9040eedce-attribute"></a>
### `attribute`

The text
The `name="value"` for one attribute, quoted so the value survives whole.

<storycode:copy id="f-39a9040eedce-attribute">
// ---------------------------------------------------------------------------
// The text
// ---------------------------------------------------------------------------

/** The `name="value"` for one attribute, quoted so the value survives whole. */
function attribute(name: string, value: string): string {
  const quote = value.includes('"') ? "'" : '"';
  return ` ${name}=${quote}${value}${quote}`;
}

</storycode:copy>
<a id="f-39a9040eedce-attributes"></a>
### `attributes`

The open tag's attribute list, in the catalogue's field order, skipping
every field left empty — an attribute nobody filled in is not an
attribute, and `show=""` is not the same as no `show`.

<storycode:copy id="f-39a9040eedce-attributes">
/** The open tag's attribute list, in the catalogue's field order, skipping
 * every field left empty — an attribute nobody filled in is not an
 * attribute, and `show=""` is not the same as no `show`. */
function attributes(element: InsertElement, values: FieldValues): string {
  let out = "";
  for (const field of element.fields) {
    const value = normalize(element, field.name, values[field.name] ?? "");
    if (value) out += attribute(field.name, value);
  }
  return out;
}

</storycode:copy>
<a id="f-39a9040eedce-render"></a>
### `render`

The element on its own, with no regard for what surrounds it, alongside
where its body begins — found by construction rather than by searching the
result, because a body that repeats the open tag would fool an indexOf.

<storycode:copy id="f-39a9040eedce-render">
/** The element on its own, with no regard for what surrounds it, alongside
 * where its body begins — found by construction rather than by searching the
 * result, because a body that repeats the open tag would fool an indexOf. */
function render(
  element: InsertElement,
  values: FieldValues,
  body: string,
  extras: Readonly<Record<string, string>> = {},
): { text: string; bodyAt: number } {
  const known = new Set(element.fields.map((field) => field.name));
  const preserved = Object.entries(extras)
    .filter(([name]) => !known.has(name))
    .map(([name, value]) => attribute(name, value))
    .join("");
  const open = `<hick:${element.tag}${attributes(element, values)}${preserved}`;
  if (element.body === "none") {
    const text = `${open} />`;
    return { text, bodyAt: text.length };
  }
  const close = `</hick:${element.tag}>`;
  if (element.body === "inline") {
    return { text: `${open}>${body}${close}`, bodyAt: open.length + 1 };
  }
  return { text: `${open}>\n${body}\n${close}`, bodyAt: open.length + 2 };
}

</storycode:copy>
<a id="f-39a9040eedce-renderElement"></a>
### `renderElement`

The element on its own. Exported for the panel's preview, which shows
exactly the bytes an insert would write.

<storycode:copy id="f-39a9040eedce-renderElement">
/** The element on its own. Exported for the panel's preview, which shows
 * exactly the bytes an insert would write. */
export function renderElement(
  element: InsertElement,
  values: FieldValues,
  body = "",
): string {
  return render(element, values, body).text;
}

</storycode:copy>
<a id="f-39a9040eedce-renderExistingElement"></a>
### `renderExistingElement`

Render an existing element after editing its catalogue fields. Attributes
unknown to this version of the catalogue survive the edit rather than being
silently erased.

<storycode:copy id="f-39a9040eedce-renderExistingElement">
/** Render an existing element after editing its catalogue fields. Attributes
 * unknown to this version of the catalogue survive the edit rather than being
 * silently erased. */
export function renderExistingElement(
  element: InsertElement,
  values: FieldValues,
  body: string,
  attrs: Readonly<Record<string, string>>,
): string {
  return render(element, values, body, attrs).text;
}

</storycode:copy>
<a id="f-39a9040eedce-defaultBody"></a>
### `defaultBody`

What the body input starts out holding: whatever was selected in the
buffer, so choosing Copy with a paragraph highlighted wraps that
paragraph — otherwise the catalogue's own starter text, which the insert
leaves selected so the first keystroke replaces it.

<storycode:copy id="f-39a9040eedce-defaultBody">
/**
 * What the body input starts out holding: whatever was selected in the
 * buffer, so choosing Copy with a paragraph highlighted wraps that
 * paragraph — otherwise the catalogue's own starter text, which the insert
 * leaves selected so the first keystroke replaces it.
 */
export function defaultBody(
  element: InsertElement,
  selected: string,
  values?: FieldValues,
): string {
  if (element.body === "none") return "";
  if (selected.trim().length > 0) return selected;
  const by = element.bodyPlaceholderBy;
  const chosen = by && values ? by.bodies[values[by.field] ?? ""] : undefined;
  return chosen ?? element.bodyPlaceholder ?? "";
}

</storycode:copy>
<a id="f-39a9040eedce-EditContext"></a>
### `EditContext`

What the caret sits in, and what already surrounds it.

<storycode:copy id="f-39a9040eedce-EditContext">
/** What the caret sits in, and what already surrounds it. */
export interface EditContext {
  /** Everything before the insertion point. */
  before: string;
  /** Everything after it. */
  after: string;
  /** The selection the insertion replaces — becomes the element's body when
   * it has one, so selecting a paragraph and inserting Copy wraps it. */
  selected: string;
}

</storycode:copy>
<a id="f-39a9040eedce-Insertion"></a>
### `Insertion`

<storycode:copy id="f-39a9040eedce-Insertion">
export interface Insertion {
  /** The text to put in place of the selection. */
  text: string;
  /** Where the selection lands afterwards, as offsets into `text`. When the
   * body was written from a placeholder these span it, so typing replaces
   * it; otherwise both are the caret. */
  selectFrom: number;
  selectTo: number;
}

</storycode:copy>
<a id="f-39a9040eedce-trailingBreaks"></a>
### `trailingBreaks`

How many blank-line-ish newlines already sit at the end of `before`.

<storycode:copy id="f-39a9040eedce-trailingBreaks">
/** How many blank-line-ish newlines already sit at the end of `before`. */
function trailingBreaks(text: string): number {
  if (text.length === 0) return 2; // The start of a document needs no lead-in.
  const match = /\n*$/.exec(text);
  return match ? match[0].length : 0;
}

</storycode:copy>
<a id="f-39a9040eedce-leadingBreaks"></a>
### `leadingBreaks`

<storycode:copy id="f-39a9040eedce-leadingBreaks">
function leadingBreaks(text: string): number {
  if (text.length === 0) return 2;
  const match = /^\n*/.exec(text);
  return match ? match[0].length : 0;
}

</storycode:copy>
<a id="f-39a9040eedce-currentIndent"></a>
### `currentIndent`

The indentation of the line the caret is on, reused for a child element
so a rule lands under its container rather than at column zero.

<storycode:copy id="f-39a9040eedce-currentIndent">
/** The indentation of the line the caret is on, reused for a child element
 * so a rule lands under its container rather than at column zero. */
function currentIndent(before: string): string {
  const line = before.slice(before.lastIndexOf("\n") + 1);
  const match = /^[ \t]*/.exec(line);
  return match ? match[0] : "";
}

</storycode:copy>
<a id="f-39a9040eedce-buildInsertion"></a>
### `buildInsertion`

Build the exact edit: the text to insert, and where the selection goes.
The surrounding bytes decide the padding, which is the whole reason this
takes a context rather than returning a bare snippet. A block element
inserted mid-paragraph opens its own paragraph; the same element inserted
into an empty document adds nothing to trim.

<storycode:copy id="f-39a9040eedce-buildInsertion">
/**
 * Build the exact edit: the text to insert, and where the selection goes.
 *
 * The surrounding bytes decide the padding, which is the whole reason this
 * takes a context rather than returning a bare snippet. A block element
 * inserted mid-paragraph opens its own paragraph; the same element inserted
 * into an empty document adds nothing to trim.
 */
export function buildInsertion(
  element: InsertElement,
  values: FieldValues,
  context: EditContext,
  bodyOverride?: string,
): Insertion {
  const body = element.body === "none" ? "" : (bodyOverride ?? defaultBody(element, context.selected));

  const { text: rendered, bodyAt } = render(element, values, body);

  let lead = "";
  let trail = "";
  let indent = "";
  if (element.placement === "block") {
    lead = "\n".repeat(Math.max(0, 2 - trailingBreaks(context.before)));
    trail = "\n".repeat(Math.max(0, 2 - leadingBreaks(context.after)));
  } else if (element.placement === "child") {
    // A rule belongs under its parent, so it always starts a line — and
    // keeps whatever indentation that line already carries, falling back to
    // one step in when the caret sits at column zero.
    const atLineStart = trailingBreaks(context.before) > 0;
    lead = atLineStart ? "" : "\n";
    indent = (atLineStart ? "" : currentIndent(context.before)) || "  ";
    trail = leadingBreaks(context.after) > 0 ? "" : "\n";
  }

  // A multi-line child (an expect inside an exec) indents every line it
  // writes, not just the first; a block element is already at column zero.
  const placed =
    indent && rendered.includes("\n")
      ? rendered
          .split("\n")
          .map((line) => (line.length > 0 ? indent + line : line))
          .join("\n")
      : indent + rendered;

  const text = lead + placed + trail;
  // The body lands one indent further along than the bare render says, and
  // an indented multi-line body pushes every line after the first as well.
  const shift = indent ? indent.length * (1 + countBreaks(rendered.slice(0, bodyAt))) : 0;
  const selectFrom = lead.length + shift + bodyAt;
  return { text, selectFrom, selectTo: selectFrom + body.length };
}

</storycode:copy>
<a id="f-39a9040eedce-countBreaks"></a>
### `countBreaks`

<storycode:copy id="f-39a9040eedce-countBreaks">
function countBreaks(text: string): number {
  let n = 0;
  for (const ch of text) if (ch === "\n") n += 1;
  return n;
}
</storycode:copy>

### Assemble `apps/web/src/lib/insertRendering.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/insertRendering.ts"><storycode:paste select="#f-39a9040eedce-FieldValues"/><storycode:paste select="#f-39a9040eedce-initialValues"/><storycode:paste select="#f-39a9040eedce-normalize"/><storycode:paste select="#f-39a9040eedce-validate"/><storycode:paste select="#f-39a9040eedce-attribute"/><storycode:paste select="#f-39a9040eedce-attributes"/><storycode:paste select="#f-39a9040eedce-render"/><storycode:paste select="#f-39a9040eedce-renderElement"/><storycode:paste select="#f-39a9040eedce-renderExistingElement"/><storycode:paste select="#f-39a9040eedce-defaultBody"/><storycode:paste select="#f-39a9040eedce-EditContext"/><storycode:paste select="#f-39a9040eedce-Insertion"/><storycode:paste select="#f-39a9040eedce-trailingBreaks"/><storycode:paste select="#f-39a9040eedce-leadingBreaks"/><storycode:paste select="#f-39a9040eedce-currentIndent"/><storycode:paste select="#f-39a9040eedce-buildInsertion"/><storycode:paste select="#f-39a9040eedce-countBreaks"/></storycode:file>

</storycode:doc>
