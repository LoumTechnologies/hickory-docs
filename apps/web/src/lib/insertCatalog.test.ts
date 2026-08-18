import { describe, expect, it } from "vitest";

// Protects docs/guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed.md
import {
  INSERT_ELEMENTS,
  INSERT_GROUPS,
  buildInsertion,
  elementById,
  filterElements,
  initialValues,
  normalize,
  renderElement,
  validate,
  type EditContext,
  type InsertElement,
} from "./insertCatalog";

function el(id: string): InsertElement {
  const element = elementById(id);
  if (!element) throw new Error(`no element ${id}`);
  return element;
}

/** A caret in the middle of an ordinary paragraph. */
function at(before: string, after: string, selected = ""): EditContext {
  return { before, after, selected };
}

describe("the catalogue", () => {
  it("gives every element a group the panel lists", () => {
    for (const element of INSERT_ELEMENTS) {
      expect(INSERT_GROUPS).toContain(element.group);
    }
  });

  it("has unique menu ids even where several entries write one tag", () => {
    const ids = INSERT_ELEMENTS.map((e) => e.id);
    expect(new Set(ids).size).toBe(ids.length);
    // The capability rules are the case that forces id and tag apart.
    const allows = INSERT_ELEMENTS.filter((e) => e.tag === "allow");
    expect(allows.length).toBeGreaterThan(1);
  });

  it("gives an element with a body somewhere for the caret to go", () => {
    for (const element of INSERT_ELEMENTS) {
      if (element.body !== "none") expect(element.bodyLabel).toBeTruthy();
    }
  });

  it("carries no machine-written transcript tags", () => {
    const tags = new Set(INSERT_ELEMENTS.map((e) => e.tag));
    for (const machine of ["user", "assistant", "action", "observation", "tool-result", "usage"]) {
      expect(tags.has(machine)).toBe(false);
    }
  });
});

describe("search", () => {
  it("ranks a title match above a summary mention", () => {
    const hits = filterElements("network");
    expect(hits[0].id).toBe("allow-network");
    expect(hits.map((h) => h.id)).toContain("deny-network");
  });

  it("finds an element by its tag name", () => {
    expect(filterElements("attenuate").map((h) => h.id)).toContain("attenuate");
  });

  it("keeps catalogue order for an empty query", () => {
    expect(filterElements("  ").map((e) => e.id)).toEqual(INSERT_ELEMENTS.map((e) => e.id));
  });
});

describe("values", () => {
  it("starts a choice field on its first option and a text field empty", () => {
    const values = initialValues(el("exec"));
    expect(values.show).toBe("");
    expect(values.container).toBe("");
  });

  it("prefills the rule fields that only ever have one sensible value", () => {
    expect(initialValues(el("deny-network")).network).toBe("*");
  });

  it("adds the # a selector needs, and leaves a class selector alone", () => {
    expect(normalize(el("paste"), "select", "version")).toBe("#version");
    expect(normalize(el("paste"), "select", "#version")).toBe("#version");
    expect(normalize(el("diagram"), "asserts", ".claims")).toBe(".claims");
  });

  it("refuses a required field left empty, naming the element and the field", () => {
    const problems = validate(el("copy"), { id: "" });
    expect(problems.id).toContain("Copy");
    expect(problems.id).toContain("id");
  });

  it("accepts a value carrying one kind of quote", () => {
    expect(validate(el("confirm"), { message: 'Deploy "prod"?' })).toEqual({});
    expect(validate(el("confirm"), { message: "Deploy 'prod'?" })).toEqual({});
  });

  it("refuses a value carrying both, because hick never escapes one", () => {
    const problems = validate(el("confirm"), { message: `it's "fine"` });
    expect(problems.message).toContain("never escaped");
  });
});

describe("the text an insert writes", () => {
  it("omits every attribute nobody filled in", () => {
    const text = renderElement(el("exec"), { container: "py", image: "", show: "", timeout: "" });
    expect(text).toBe('<hick:exec container="py">\n\n</hick:exec>');
  });

  it("writes attributes in catalogue order, not the order they were typed", () => {
    const text = renderElement(el("container"), { image: "alpine", name: "base" });
    expect(text).toBe('<hick:container name="base" image="alpine">\n\n</hick:container>');
  });

  it("switches to single quotes rather than escaping a double quote", () => {
    const text = renderElement(el("confirm"), { message: 'Deploy "prod"?' });
    expect(text).toBe(`<hick:confirm message='Deploy "prod"?' />`);
  });

  it("self-closes an element with no body", () => {
    expect(renderElement(el("paste"), { select: "#version" })).toBe(
      '<hick:paste select="#version" />',
    );
  });

  it("keeps a one-line element on one line", () => {
    expect(renderElement(el("var"), { name: "version" }, "2.0.0")).toBe(
      '<hick:var name="version">2.0.0</hick:var>',
    );
  });
});

describe("where the insert lands", () => {
  it("opens its own paragraph when the caret sits in prose", () => {
    const { text } = buildInsertion(el("copy"), { id: "version" }, at("Some prose.", " More."));
    expect(text).toBe('\n\n<hick:copy id="version">\nThe text to reuse.\n</hick:copy>\n\n');
  });

  it("adds no padding it does not need", () => {
    const { text } = buildInsertion(el("confirm"), { message: "Go?" }, at("done\n\n", "\n\nnext"));
    expect(text).toBe('<hick:confirm message="Go?" />');
  });

  it("needs no lead-in at the very start of a document", () => {
    const { text } = buildInsertion(el("confirm"), { message: "Go?" }, at("", "\n\nrest"));
    expect(text).toBe('<hick:confirm message="Go?" />');
  });

  it("wraps the selection instead of a placeholder, and reselects it", () => {
    const context = at("before\n\n", "\n\nafter", "the paragraph");
    const { text, selectFrom, selectTo } = buildInsertion(el("cut"), { id: "notes" }, context);
    expect(text).toBe('<hick:cut id="notes">\nthe paragraph\n</hick:cut>');
    expect(text.slice(selectFrom, selectTo)).toBe("the paragraph");
  });

  it("selects the placeholder body so the first keystroke replaces it", () => {
    const { text, selectFrom, selectTo } = buildInsertion(el("exec"), { container: "py" }, at("", ""));
    expect(text.slice(selectFrom, selectTo)).toBe("echo hello");
  });

  it("leaves the caret after an element with no body", () => {
    const { text, selectFrom, selectTo } = buildInsertion(el("val"), { name: "version" }, at("v", ""));
    expect(selectFrom).toBe(selectTo);
    expect(selectFrom).toBe(text.length);
  });

  it("puts an inline element exactly at the caret, padding nothing", () => {
    const { text } = buildInsertion(el("paste"), { select: "version" }, at("Version: ", "\n"));
    expect(text).toBe('<hick:paste select="#version" />');
  });

  it("indents a rule under its container and gives it its own line", () => {
    const context = at('<hick:container name="base" image="alpine">', "\n</hick:container>\n");
    const { text } = buildInsertion(el("allow-network"), { network: "github.com:443" }, context);
    expect(text).toBe('\n  <hick:allow network="github.com:443" />');
  });

  it("indents every line of a multi-line child, not just the first", () => {
    const context = at("cargo test\n", "</hick:exec>\n");
    const { text, selectFrom, selectTo } = buildInsertion(
      el("expect"),
      { match: "exact" },
      context,
    );
    expect(text).toBe('  <hick:expect match="exact">\n  hello\n  </hick:expect>\n');
    expect(text.slice(selectFrom, selectTo)).toBe("hello");
  });

  it("keeps the indentation the line it starts on already carries", () => {
    const context = at('  <hick:allow network="*:443" />', "\n</hick:container>");
    const { text } = buildInsertion(el("deny-network"), { network: "*" }, context);
    expect(text).toBe('\n  <hick:deny network="*" />');
  });
});
