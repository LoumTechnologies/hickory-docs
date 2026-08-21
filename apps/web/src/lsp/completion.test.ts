import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { CompletionContext } from "@codemirror/autocomplete";

import {
  completionSource,
  lspCompletion,
  originLabel,
  originMark,
  projectCompletion,
  wordBefore,
} from "./completion";

const contextAt = (doc: string, pos: number, explicit = false) =>
  new CompletionContext(EditorState.create({ doc }), pos, explicit);

describe("telling the two sources apart", () => {
  it("gives each origin its own mark", () => {
    // A list that mixed them silently would make the guesses look like facts.
    expect(originMark("lsp")).not.toBe(originMark("project"));
  });

  it("says what each one actually knows", () => {
    expect(originLabel("lsp")).toMatch(/in scope/i);
    expect(originLabel("project", false)).toMatch(/how often/i);
    expect(originLabel("project", true)).toMatch(/what you are writing/i);
  });

  it("does not claim semantics that did not happen", () => {
    // With no model installed the ranking is frequency, and the label says
    // frequency.
    expect(originLabel("project", false)).not.toMatch(/what you are writing/i);
  });

  it("sorts the authoritative answers first", () => {
    // When something IS in scope, burying it under a frequency count would
    // be a poor trade.
    const lsp = lspCompletion({ label: "x" });
    const project = projectCompletion({ text: "y", detail: "a.py:1", score: 9, semantic: false });
    expect((lsp.boost ?? 0) > (project.boost ?? 0)).toBe(true);
  });

  it("puts them in named sections", () => {
    expect(lspCompletion({ label: "x" }).section).toBe("In scope");
    expect(
      projectCompletion({ text: "y", detail: "", score: 1, semantic: false }).section,
    ).toBe("This project");
  });
});

describe("when the popup opens at all", () => {
  it("waits for a word rather than firing on every keystroke", () => {
    expect(wordBefore(contextAt("x = ", 4))).toBeNull();
  });

  it("completes from nothing when it was asked for explicitly", () => {
    // Ctrl-Space means "show me now", which is a different question.
    expect(wordBefore(contextAt("x = ", 4, true))).toBeNull();
    expect(wordBefore(contextAt("x = tot", 7, true))?.text).toBe("tot");
  });

  it("finds the word the caret is inside", () => {
    const word = wordBefore(contextAt("total_uni", 9));
    expect(word).toEqual({ from: 0, text: "total_uni" });
  });
});

describe("the combined list", () => {
  const suggest = (text: string) => ({ text, detail: "a.py:1", score: 1, semantic: false });

  it("shows both sources", async () => {
    const source = completionSource({
      project: async () => [suggest("total_units"), suggest("total_revenue")],
    });
    const result = await source(contextAt("tot", 3));
    expect(result?.options.map((o) => o.label)).toEqual(["total_units", "total_revenue"]);
  });

  it("does not show one name twice when both sources have it", async () => {
    // Showing it twice would say the two disagree, when they agree.
    const source = completionSource({
      lsp: {
        client: { completion: async () => [{ label: "total_units" }] } as never,
        uri: "file:///x",
        positionAt: () => ({ line: 0, character: 0 }),
      },
      project: async () => [suggest("total_units"), suggest("total_revenue")],
    });
    const result = await source(contextAt("tot", 3));
    expect(result?.options.map((o) => o.label)).toEqual(["total_units", "total_revenue"]);
  });

  it("keeps the project's answers when the language server fails", async () => {
    // A server mid-index must not stop the project's own answers appearing.
    const source = completionSource({
      lsp: {
        client: { completion: async () => { throw new Error("not ready"); } } as never,
        uri: "file:///x",
        positionAt: () => ({ line: 0, character: 0 }),
      },
      project: async () => [suggest("total_units")],
    });
    const result = await source(contextAt("tot", 3));
    expect(result?.options.map((o) => o.label)).toEqual(["total_units"]);
  });

  it("keeps the language server's answers when the project index fails", async () => {
    const source = completionSource({
      lsp: {
        client: { completion: async () => [{ label: "typed_thing" }] } as never,
        uri: "file:///x",
        positionAt: () => ({ line: 0, character: 0 }),
      },
      project: async () => {
        throw new Error("indexing");
      },
    });
    const result = await source(contextAt("ty", 2));
    expect(result?.options.map((o) => o.label)).toEqual(["typed_thing"]);
  });

  it("answers nothing rather than an empty popup", async () => {
    const source = completionSource({ project: async () => [] });
    expect(await source(contextAt("tot", 3))).toBeNull();
  });

  it("stays open while the word is still being typed", async () => {
    const source = completionSource({ project: async () => [suggest("total_units")] });
    const result = await source(contextAt("tot", 3));
    expect("total".match(result!.validFor as RegExp)).toBeTruthy();
  });
});
