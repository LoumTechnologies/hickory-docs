import { describe, expect, it } from "vitest";

// Protects docs/guarantees/authoring/the-gutters-never-skip-a-number.md
// and docs/guarantees/authoring/a-fence-becomes-a-cell-that-runs.md
import { cardsOf } from "./cards";
import { containerNamesOf, parseHickDoc, proseFences } from "./hickDoc";

const cards = (text: string) => cardsOf(parseHickDoc(text), { text });

describe("what the rail lists", () => {
  it("gives every exec cell an icon, in document order", () => {
    const text =
      '<hick:exec container="a">\nls\n</hick:exec>\n\n<hick:exec container="b">\npwd\n</hick:exec>\n';
    const list = cards(text).filter((c) => c.kind === "exec");
    expect(list.map((c) => c.index)).toEqual([0, 1]);
    expect(list[0].label).toContain("a");
    expect(list[1].label).toContain("b");
    expect(list[0].at).toBeLessThan(list[1].at);
  });

  it("gives a cell an icon before the server has ever heard of it", () => {
    // A cell you just typed must be runnable without a round-trip; an icon
    // that appeared late would read as a bug.
    expect(cards("<hick:exec>\nls\n</hick:exec>\n")).toHaveLength(1);
  });

  it("lists diagrams, and can be told not to", () => {
    const text = '<hick:diagram renderer="mermaid">\ngraph TD\n</hick:diagram>\n';
    expect(cards(text).map((c) => c.kind)).toEqual(["diagram"]);
    expect(cardsOf(parseHickDoc(text), { text, diagrams: false })).toEqual([]);
  });

  it("gives a file block that writes a picture its own icon", () => {
    // The block's source is the plotting code; what it MEANS is a chart, so
    // the rail offers the way to see which chart it is.
    const text =
      '<hick:file path="chart.svg">\n<hick:exec container="r">\nplot\n</hick:exec>\n</hick:file>\n';
    const list = cards(text);
    expect(list.map((c) => c.kind).sort()).toEqual(["exec", "picture"]);
    expect(list.find((c) => c.kind === "picture")?.label).toContain("chart.svg");
    // And the cell inside it is marked, so the rail can leave its source
    // toggle off — the picture block owns that verb.
    expect(list.find((c) => c.kind === "exec")?.insidePicture).toBe(true);
  });

  it("leaves a cell outside a picture with its own source toggle", () => {
    const list = cards('<hick:exec container="a">\nls\n</hick:exec>\n');
    expect(list[0].insidePicture).toBeFalsy();
  });

  it("leaves a file block that writes text alone", () => {
    // A README is read as its source. Only a file a browser can DRAW has a
    // second thing to show.
    const text = '<hick:file path="README.md">\nhello\n</hick:file>\n';
    expect(cards(text).some((c) => c.kind === "picture")).toBe(false);
  });

  it("sorts every kind into one document-order rail", () => {
    const text =
      "# Doc\n\n```bash\nls\n```\n\n" +
      '<hick:exec container="a">\npwd\n</hick:exec>\n';
    expect(cards(text).map((c) => c.kind)).toEqual(["fence", "exec"]);
  });
});

describe("which fences are offered", () => {
  const fencesIn = (text: string) => proseFences(parseHickDoc(text), text);

  it("finds a closed fence in prose, with its language and body", () => {
    const text = "before\n\n```python\nprint(1)\n```\n\nafter\n";
    const [fence] = fencesIn(text);
    expect(fence.info).toBe("python");
    expect(fence.body).toBe("print(1)");
    expect(text.slice(fence.from, fence.to)).toBe("```python\nprint(1)\n```");
  });

  it("ignores a fence inside a generated file — that is the file's content", () => {
    const text = '<hick:file path="README.md">\n```bash\nls\n```\n</hick:file>\n';
    expect(fencesIn(text)).toEqual([]);
    expect(cards(text).some((c) => c.kind === "fence")).toBe(false);
  });

  it("ignores a fence inside an exec — that is part of a command", () => {
    const text = '<hick:exec container="a">\ncat <<EOF\n```\nEOF\n</hick:exec>\n';
    expect(fencesIn(text)).toEqual([]);
  });

  it("ignores an unterminated fence rather than guessing where it ends", () => {
    expect(fencesIn("text\n\n```python\nprint(1)\n")).toEqual([]);
  });

  it("keeps tilde fences and backtick fences apart", () => {
    const text = "~~~js\nlet a\n~~~\n";
    const [fence] = fencesIn(text);
    expect(fence.info).toBe("js");
    expect(fence.body).toBe("let a");
  });

  it("handles an empty fence without producing a negative range", () => {
    const text = "```\n```\n";
    const [fence] = fencesIn(text);
    expect(fence.body).toBe("");
    expect(fence.to).toBeGreaterThan(fence.from);
  });
});

describe("the containers a conversion can choose from", () => {
  const names = (text: string) => containerNamesOf(parseHickDoc(text));

  it("includes self-closing declarations", () => {
    expect(names('<hick:container name="py" image="python:3.12" />\n')).toEqual(["py"]);
  });

  it("includes a container that only ever exists implicitly", () => {
    // `image=` on the first exec creates the container, so a document with no
    // <hick:container> at all still has one — and saying "this document
    // declares no container" when its name is right there is just wrong.
    expect(names('<hick:exec container="py" image="python:3.12">\nls\n</hick:exec>\n')).toEqual([
      "py",
    ]);
  });

  it("includes a fork's target", () => {
    const text =
      '<hick:container name="base" image="ubuntu" />\n<hick:fork from="base" to="analyzer" />\n';
    expect(names(text)).toEqual(["base", "analyzer"]);
  });

  it("lists each name once, in document order", () => {
    const text =
      '<hick:exec container="b">\nx\n</hick:exec>\n<hick:exec container="a">\ny\n</hick:exec>\n' +
      '<hick:exec container="b">\nz\n</hick:exec>\n';
    expect(names(text)).toEqual(["b", "a"]);
  });

  it("is empty for a document with no containers at all", () => {
    expect(names("# Just prose\n")).toEqual([]);
  });
});
