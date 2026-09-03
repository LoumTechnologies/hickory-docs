// Protects docs/guarantees/editor-intelligence/a-hover-is-rendered-not-dumped.md

import { describe, expect, it } from "vitest";
import { renderHoverMarkdown } from "./hoverMarkdown";

function render(text: string): HTMLElement {
  const host = document.createElement("div");
  host.appendChild(renderHoverMarkdown(text));
  return host;
}

/** What rust-analyzer actually answers for a `Vec` binding. */
const RUST_ANALYZER = [
  "```rust",
  "let names: Vec<String>",
  "```",
  "",
  "---",
  "",
  "A contiguous growable array type, written as `Vec<T>`, short for *vector*.",
  "",
  "See [Vec](https://doc.rust-lang.org/std/vec/struct.Vec.html) for more.",
].join("\n");

describe("a hover's markdown", () => {
  it("renders a fence as a code block, without its backticks", () => {
    const host = render(RUST_ANALYZER);
    const code = host.querySelector("pre.cm-lsp-hover-code code");
    expect(code?.textContent).toBe("let names: Vec<String>");
    // The three things a person used to see and should not.
    expect(host.textContent).not.toContain("```");
    expect(host.textContent).not.toContain("---");
    expect(host.querySelector("pre")?.getAttribute("data-language")).toBe("rust");
  });

  it("draws a rule as a rule", () => {
    expect(render(RUST_ANALYZER).querySelectorAll("hr")).toHaveLength(1);
  });

  it("shows a link as its text, since a tooltip is not clickable", () => {
    const host = render(RUST_ANALYZER);
    expect(host.textContent).toContain("See Vec for more.");
    expect(host.textContent).not.toContain("https://doc.rust-lang.org");
    expect(host.querySelector("a")).toBeNull();
  });

  it("renders inline code and emphasis", () => {
    const host = render("A `Vec<T>`, short for *vector*, and **not** a slice.");
    expect(host.querySelector("code")?.textContent).toBe("Vec<T>");
    expect(host.querySelector("em")?.textContent).toBe("vector");
    expect(host.querySelector("strong")?.textContent).toBe("not");
  });

  it("does not read bold as two emphases", () => {
    const host = render("**bold**");
    expect(host.querySelector("strong")?.textContent).toBe("bold");
    expect(host.querySelector("em")).toBeNull();
  });

  it("never interprets the server's text as HTML", () => {
    // The string comes from another program. `innerHTML` on it would be an
    // injection with extra steps.
    const host = render("<img src=x onerror=alert(1)> and `<b>literal</b>`");
    expect(host.querySelector("img")).toBeNull();
    expect(host.querySelector("b")).toBeNull();
    expect(host.textContent).toContain("<img src=x onerror=alert(1)>");
    expect(host.querySelector("code")?.textContent).toBe("<b>literal</b>");
  });

  it("keeps a paragraph's own line breaks", () => {
    const host = render("first line\nsecond line");
    expect(host.querySelectorAll("p")).toHaveLength(1);
    expect(host.textContent).toBe("first line\nsecond line");
  });

  it("splits paragraphs on a blank line", () => {
    expect(render("one\n\ntwo").querySelectorAll("p")).toHaveLength(2);
  });

  it("renders a fence nobody closed rather than swallowing it", () => {
    const host = render("```rust\nfn main() {}");
    expect(host.querySelector("code")?.textContent).toBe("fn main() {}");
  });

  it("produces nothing for nothing", () => {
    expect(render("").childNodes).toHaveLength(0);
    expect(render("\n\n  \n").childNodes).toHaveLength(0);
  });
});
