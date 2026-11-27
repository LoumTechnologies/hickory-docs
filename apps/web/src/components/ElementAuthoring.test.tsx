// Contract: .hickory-source/frontends/contract.md
import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ElementAuthoring, elementsByLine, mountNativeElements } from "./ElementAuthoring";
import { elementOptions, newElement, planElement, readElements } from "./ElementAuthoringModel";

const option = (name: string) => elementOptions.find(item => item.id === name)!;
function target(source: string, id: string) {
  const reading = readElements(source);
  const item = reading.targets.find(item => item.option.id === id)!;
  expect(item, id).toBeDefined();
  return { reading, item };
}

describe("element authoring plans", () => {
  it("writes into the current namespace and selects the starter body through CRLF", () => {
    const source = '<x:doc xmlns:x="http://www.hickorydocs.com/1.0">\r\n\r\n😀 selected\r\n\r\n</x:doc>';
    const from = source.indexOf("😀"), to = from + "😀 selected".length;
    const reading = readElements(source), copy = option("copy");
    const start = newElement(copy, source.slice(from, to));
    const plan = planElement(reading, { from, to }, copy, { id: "keep" }, start.body);
    expect(plan.insert).toBe('<x:copy id="keep">\r\n😀 selected\r\n</x:copy>');
    expect(plan.insert.slice(plan.selectFrom - from, plan.selectTo - from)).toBe("😀 selected");
  });

  it("preserves unknown Hickory attributes, quote style and mixed body bytes on attribute edits", () => {
    const source = "😀\r\n<hick:copy  id = 'before' future='a id=\"b\"' intact>\r\n α\n β\r\n</hick:copy>";
    const { reading, item } = target(source, "copy");
    expect(item.from).toBe(source.indexOf("<"));
    const plan = planElement(reading, item, item.option, { id: "after" }, item.body, item);
    expect(plan.insert).toBe(source.slice(item.from).replace("id = 'before'", 'id="after"'));
  });

  it("keeps Markdown syntax inside Hickory file payloads out of the gutter", () => {
    const source = '<hick:file path="README.md">\n# Generated\n**example**\n</hick:file>\n\n# Actual **prose**\n\n```md\n# Sample\n```';
    const reading = readElements(source);
    expect([...elementsByLine(reading)].map(([line, items]) => [line, items.map(item => item.option.id)])).toEqual([
      [0, ["file"]], [5, ["md:heading", "md:bold"]], [7, ["md:fence"]],
    ]);
    expect(() => planElement(reading, { from: 12, to: 12 }, option("md:bold"), {}, "Example")).toThrow("outside the tag");
  });

  it("keeps executable fence spelling when editing its body", () => {
    const source = '```bash exec="true"\nprintf hi\n```';
    const { reading, item } = target(source, "md:fence");
    expect(reading.structure.blocks.some(block => block.name === "exec")).toBe(true);
    expect(item.render(item.values, "printf hello")).toBe(source.replace("printf hi", "printf hello"));
  });

  it("edits the level and content of a heading without replacing its closing markers", () => {
    const source = "# Before ##\r\nNext";
    const { reading, item } = target(source, "md:heading");
    const plan = planElement(reading, item, item.option, { level: "3" }, "After", item);
    expect(plan.insert).toBe("### After ##");
    expect(source.slice(plan.to)).toBe("\r\nNext");
  });

  it("keeps link titles and widens a fence when the edited body contains its delimiter", () => {
    const link = '[Title](<a path> "tip")';
    const { item } = target(link, "md:link");
    expect(item.render({ destination: "https://example.com" }, "Next")).toBe('[Next](https://example.com "tip")');
    const fence = target("```ts\n```", "md:fence").item;
    expect(fence.render({ info: "ts" }, "x\n```")).toBe("````ts\nx\n```\n````");
  });

  it("distinguishes a project entry route and makes new construction forms available", () => {
    const { item } = target('<hick:project entry="../project.md" />', "project-route");
    expect(item.render({ entry: "project.md" }, "")).toBe('<hick:project entry="project.md" />');
    for (const id of ["project", "project-route", "entry", "patch", "scope"]) expect(option(id)).toBeDefined();
  });

  it("uses Markdown alternatives with inline selections and validates required fields", () => {
    const source = "Before text after";
    const reading = readElements(source);
    const range = { from: 7, to: 11 };
    const bold = planElement(reading, range, option("md:bold"), {}, "text");
    expect(bold).toEqual({ ...range, insert: "**text**", selectFrom: 9, selectTo: 13 });
    expect(() => planElement(reading, range, option("md:link"), { destination: "" }, "text")).toThrow("destination");
    expect(planElement(reading, range, option("allow-network"), { network: "*" }, "").insert).toContain('<hick:allow network="*" />');
  });
});

it("shows the exact preview and applies only after submitting the form", () => {
  const apply = vi.fn();
  render(<ElementAuthoring reading={readElements("Title")} range={{ from: 0, to: 5 }} close={() => {}} apply={apply} />);
  fireEvent.change(screen.getByRole("textbox", { name: "Find an element" }), { target: { value: "heading" } });
  fireEvent.click(screen.getByRole("button", { name: "Heading Markdown" }));
  fireEvent.change(screen.getByRole("combobox", { name: "Level" }), { target: { value: "2" } });
  expect(screen.getByLabelText("Source preview").textContent).toBe("## Title");
  expect(apply).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Insert" }));
  expect(apply).toHaveBeenCalledWith({ from: 0, to: 5, insert: "## Title", selectFrom: 3, selectTo: 8 });
});

it("shows multiline project attributes without folding their source into a single input line", () => {
  const source = '<hick:project exclude="target/\r\nnode_modules/" />';
  const { reading, item } = target(source, "project");
  const result = render(<ElementAuthoring reading={reading} range={item} targets={[item]} close={() => {}} apply={() => undefined} />);
  const form = within(result.container);
  expect((form.getByRole("textbox", { name: /^Exclude/ }) as HTMLTextAreaElement).value).toBe("target/\nnode_modules/");
  expect(form.getByLabelText("Source preview").textContent).toBe(source);
  result.unmount();
});

it("binds native gutter readings and submissions to the document revision and request", async () => {
  const node = document.createElement("div");
  document.body.append(node);
  const send = vi.fn();
  const form = mountNativeElements(node, send);
  try {
    await form.read("one", "## Original");
    expect(send).toHaveBeenCalledWith({ kind: "lines", revision: "one", lines: [0] });
    await form.read("two", "## Current");
    form.open({ id: "old", revision: "one", from: 0, to: 0, line: 0 });
    expect(send).toHaveBeenCalledWith({ kind: "error", message: expect.stringContaining("changed") });
    act(() => form.open({ id: "opened", revision: "two", from: 0, to: 0, line: 0 }));
    const formView = within(node);
    const field = formView.getByRole("textbox", { name: "Text" });
    fireEvent.change(field, { target: { value: "Revised" } });
    expect(formView.getByLabelText("Source preview").textContent).toBe("## Revised");
    fireEvent.click(formView.getByRole("button", { name: "Apply edit" }));
    expect(send).toHaveBeenCalledWith({ kind: "apply", id: "opened", plan: { from: 0, to: 10, insert: "## Revised", selectFrom: 10, selectTo: 10 } });
  } finally {
    act(() => form.close());
    node.remove();
  }
});
