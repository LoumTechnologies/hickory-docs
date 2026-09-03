// The little bit of Markdown a hover actually contains.
//
// A language server answers `textDocument/hover` with `MarkupContent`, and
// every real one sends Markdown: rust-analyzer wraps the signature in a
// ```rust fence, rules off the sections with `---`, and links types to
// docs.rs. The tooltip used to set that string as `textContent`, so what a
// person saw was the backticks, the word `rust`, and a row of hyphens —
// noted while dogfooding on 2026-09-02.
//
// This is deliberately not a Markdown library. A hover is a paragraph, a
// fenced block, a rule, and some inline emphasis, and the failure mode of a
// full parser here is a tooltip that renders a table. What is NOT optional is
// that everything is built as DOM and set as text: the string comes from
// another program, and `innerHTML` on it would be an injection with extra
// steps.

/** A link is shown as its text: a tooltip is not somewhere you can click. */
const LINK = /\[([^\]]+)\]\([^)]*\)/g;

/** Inline spans, in the order they are taken off. */
function inline(target: HTMLElement, text: string): void {
  const withoutLinks = text.replace(LINK, "$1");
  // One pass, longest markers first, so `**bold**` is not read as two `*em*`.
  const pattern = /(`[^`]+`)|(\*\*[^*]+\*\*)|(__[^_]+__)|(\*[^*]+\*)|(_[^_]+_)/g;
  let last = 0;
  for (const match of withoutLinks.matchAll(pattern)) {
    const at = match.index ?? 0;
    if (at > last) target.appendChild(document.createTextNode(withoutLinks.slice(last, at)));
    const token = match[0];
    if (token.startsWith("`")) {
      const code = document.createElement("code");
      code.textContent = token.slice(1, -1);
      target.appendChild(code);
    } else if (token.startsWith("**") || token.startsWith("__")) {
      const strong = document.createElement("strong");
      strong.textContent = token.slice(2, -2);
      target.appendChild(strong);
    } else {
      const em = document.createElement("em");
      em.textContent = token.slice(1, -1);
      target.appendChild(em);
    }
    last = at + token.length;
  }
  if (last < withoutLinks.length) {
    target.appendChild(document.createTextNode(withoutLinks.slice(last)));
  }
}

/** Whether a line is a horizontal rule. */
function isRule(line: string): boolean {
  const trimmed = line.trim();
  return /^(-{3,}|\*{3,}|_{3,})$/.test(trimmed);
}

/**
 * Render a hover's Markdown into a fragment.
 *
 * Everything is `textContent`; nothing is ever parsed as HTML.
 */
export function renderHoverMarkdown(text: string): DocumentFragment {
  const fragment = document.createDocumentFragment();
  const lines = text.replace(/\r\n/g, "\n").split("\n");

  let i = 0;
  let paragraph: string[] = [];
  const flush = () => {
    // Trailing blank lines are not a paragraph.
    while (paragraph.length > 0 && paragraph[paragraph.length - 1].trim() === "") paragraph.pop();
    if (paragraph.length === 0) return;
    const p = document.createElement("p");
    p.className = "cm-lsp-hover-text";
    inline(p, paragraph.join("\n"));
    fragment.appendChild(p);
    paragraph = [];
  };

  while (i < lines.length) {
    const line = lines[i];
    const fence = /^\s*```+\s*([\w+#-]*)\s*$/.exec(line);
    if (fence) {
      flush();
      const language = fence[1] ?? "";
      const body: string[] = [];
      i += 1;
      while (i < lines.length && !/^\s*```+\s*$/.test(lines[i])) {
        body.push(lines[i]);
        i += 1;
      }
      // A fence that is never closed still renders: the server sent it, and
      // swallowing the signature because a backtick is missing is worse than
      // showing it.
      i += 1;
      const pre = document.createElement("pre");
      pre.className = "cm-lsp-hover-code";
      if (language) pre.dataset.language = language;
      const code = document.createElement("code");
      code.textContent = body.join("\n");
      pre.appendChild(code);
      fragment.appendChild(pre);
      continue;
    }
    if (isRule(line)) {
      flush();
      fragment.appendChild(document.createElement("hr"));
      i += 1;
      continue;
    }
    if (line.trim() === "") {
      flush();
      i += 1;
      continue;
    }
    paragraph.push(line);
    i += 1;
  }
  flush();
  return fragment;
}
