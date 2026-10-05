import { describe, expect, it } from "vitest";
import { BrowserRun } from "./runtime";
import type { BrowserEvent } from "./runtime";
import { materializeLiteralFiles } from "../../editor/hickLang";
import type { DebugBreakpoint } from "../client";

// Guarantees: docs/guarantees/debugging/browser-debugging-runs-edited-source.md
// docs/guarantees/embedding/literal-files-use-native-bytes.md
function start(code: string, breakpoints: DebugBreakpoint[] = [], language = "js") {
  const source = `# 🦀 Live\n<hick:file path="main.${language}">\n${code}\n</hick:file>\n`;
  const events: BrowserEvent[] = [];
  const run = new BrowserRun({ source, revision: "1", file: materializeLiteralFiles(source)[0] }, "session", "hick:///note.md", breakpoints, (e) => events.push(e));
  const drain = () => { let n = 0; while (run.pump() && n++ < 2000) { /* bounded runtime */ } };
  return { run, events, drain };
}
function pause(events: BrowserEvent[]) { const event = [...events].reverse().find((e) => e.event === "stopped"); if (!event || event.event !== "stopped") throw Error("Expected real pause"); return event; }

describe("browser interpreter", () => {
  it("pauses before the statement, steps into/out of a function and continues to output", () => {
    const { run, events, drain } = start("function price(n) {\n  var total = n * 3;\n  return total;\n}\nvar input = 4;\nvar result = price(input);\nconsole.log(result);", [{ line: 7 }, { line: 8 }]);
    drain();
    expect(pause(events).line).toBe(7);
    expect(pause(events).variables.find((v) => v.name === "input")?.value).toBe("4");
    expect(pause(events).variables.find((v) => v.name === "result")?.value).toBe("undefined");
    run.step("in"); drain();
    expect(pause(events).line).toBe(3);
    expect(pause(events).frames.map((f) => f.name)).toEqual(["price", "<program>"]);
    expect(pause(events).variables.find((v) => v.name === "n")?.value).toBe("4");
    run.step("out"); drain();
    expect(pause(events).line).toBe(8);
    expect(pause(events).variables.find((v) => v.name === "result")?.value).toBe("12");
    run.step("continue"); drain();
    expect(events).toContainEqual({ event: "output", session: "session", text: "12\n", category: "stdout" });
  });
  it.each(["js", "ts"])("matches native execution for edited %s, Unicode, multiline and closures", (language) => {
    const annotation = language === "ts" ? ": number" : "";
    const code = `function outer(n${annotation}) {\n  return function inner(x${annotation}) {\n    var label = 'é🦀';\n    return n +\n      x;\n  };\n}\nvar add = outer(7);\nconsole.log(add(5));`;
    const { events, drain } = start(code, [], language); drain();
    expect(events).toContainEqual({ event: "output", session: "session", text: "12\n", category: "stdout" });
    const native: number[] = [];
    Function("console", code.replaceAll(": number", ""))({ log: (v: number) => native.push(v) });
    expect(native).toEqual([12]);
    const edited = start(code.replace("outer(7)", "outer(9)"), [], language); edited.drain();
    expect(edited.events).toContainEqual({ event: "output", session: "session", text: "14\n", category: "stdout" });
  });
  it("inspects closures and objects without invoking getters or watch calls", () => {
    const { run, events, drain } = start("var touched = 0;\nvar object = { get value() { touched++; return 8; }, nested: { x: 3 } };\nconsole.log(touched);", [{ line: 4 }]); drain();
    run.evaluate("object.nested.x + 2");
    expect(events.at(-1)).toMatchObject({ event: "value", value: "5" });
    expect(() => run.evaluate("object.value")).toThrow("getter");
    expect(() => run.evaluate("console.log(5)")).toThrow("Unsupported");
    const variable = pause(events).variables.find((v) => v.name === "object")!;
    run.children(variable.variables_reference);
    expect(events.at(-1)).toMatchObject({ event: "children", variables: expect.arrayContaining([expect.objectContaining({ name: "value", value: "[Getter — not invoked]" })]) });
    run.evaluate("touched"); expect(events.at(-1)).toMatchObject({ value: "0" });
  });
  it("captures exceptions and denies page, network and indirect eval access", () => {
    for (const code of ["fetch('https://example.com');", "console.log(document.cookie);", "(function(){}).constructor('return 1')();", "throw new Error('broken');"]) {
      const test = start(code); test.drain();
      expect(test.events.at(-1)).toMatchObject({ event: "finished", exit_code: 1 });
    }
  });
  it("refuses unsupported syntax, type errors and dependent weave", () => {
    expect(() => start("var f = () => 1;")).toThrow("Unsupported");
    expect(() => start("async function f() {}", [], "ts")).toThrow("Unsupported");
    expect(() => start("var x: number = 'bad';", [], "ts")).toThrow("Compiler diagnostics");
    expect(() => materializeLiteralFiles('<hick:file path="a.js"><hick:paste select="#x" /></hick:file>')).toThrow("full weave");
  });
  it("bounds an infinite loop and permits pause between batches", () => {
    const { run, events, drain } = start("var n = 0;\nwhile (true) { n++; }");
    expect(run.pump(100)).toBe(true);
    run.pause(); run.pump(); expect(pause(events).reason).toBeDefined();
    run.step("continue"); drain();
    expect(events.at(-1)).toMatchObject({ event: "finished", exit_code: 1 });
  });
});
