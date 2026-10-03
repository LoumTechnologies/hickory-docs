import type { InsertElement } from "./insertCatalog";

// ---------------------------------------------------------------------------
// Values
// ---------------------------------------------------------------------------

export type FieldValues = Readonly<Record<string, string>>;

/** The form's starting values: each field's `value`, else empty. */
export function initialValues(element: InsertElement): Record<string, string> {
  const values: Record<string, string> = {};
  for (const field of element.fields) {
    values[field.name] = field.value ?? (field.choices ? field.choices[0] : "");
  }
  return values;
}

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

// ---------------------------------------------------------------------------
// The text
// ---------------------------------------------------------------------------

/** The `name="value"` for one attribute, quoted so the value survives whole. */
function attribute(name: string, value: string): string {
  const quote = value.includes('"') ? "'" : '"';
  return ` ${name}=${quote}${value}${quote}`;
}

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

/** The element on its own. Exported for the panel's preview, which shows
 * exactly the bytes an insert would write. */
export function renderElement(
  element: InsertElement,
  values: FieldValues,
  body = "",
): string {
  return render(element, values, body).text;
}

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

export interface Insertion {
  /** The text to put in place of the selection. */
  text: string;
  /** Where the selection lands afterwards, as offsets into `text`. When the
   * body was written from a placeholder these span it, so typing replaces
   * it; otherwise both are the caret. */
  selectFrom: number;
  selectTo: number;
}

/** How many blank-line-ish newlines already sit at the end of `before`. */
function trailingBreaks(text: string): number {
  if (text.length === 0) return 2; // The start of a document needs no lead-in.
  const match = /\n*$/.exec(text);
  return match ? match[0].length : 0;
}

function leadingBreaks(text: string): number {
  if (text.length === 0) return 2;
  const match = /^\n*/.exec(text);
  return match ? match[0].length : 0;
}

/** The indentation of the line the caret is on, reused for a child element
 * so a rule lands under its container rather than at column zero. */
function currentIndent(before: string): string {
  const line = before.slice(before.lastIndexOf("\n") + 1);
  const match = /^[ \t]*/.exec(line);
  return match ? match[0] : "";
}

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

function countBreaks(text: string): number {
  let n = 0;
  for (const ch of text) if (ch === "\n") n += 1;
  return n;
}
