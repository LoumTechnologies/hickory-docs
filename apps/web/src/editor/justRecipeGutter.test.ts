// Contract: .hickory-source/guarantees/authoring/the-editor-keeps-its-space/contract.md
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { waitFor } from "@testing-library/react";
import { api } from "../api/client";
import { gutterSlots } from "./gutterIconLayout";
import { justRecipeGutter, launchJustRecipe, type JustRecipe } from "./justRecipeGutter";
vi.mock("../api/client", () => ({ api: { justRecipes: vi.fn(), openTerminal: vi.fn() } }));
const views: EditorView[] = [];
afterEach(() => { for (const view of views.splice(0)) { const host = view.dom.parentElement; view.destroy(); host?.remove(); } vi.restoreAllMocks(); vi.mocked(api.justRecipes).mockReset(); vi.mocked(api.openTerminal).mockReset(); });
const recipe: JustRecipe = { name: "build", doc: "Build", parameters: [] };
function setup() {
  const run = vi.fn(), error = vi.fn();
  const view = new EditorView({ parent: document.body.appendChild(document.createElement("div")), state: EditorState.create({ doc: "src/\nnext/", extensions: justRecipeGutter([{ line: 1, path: "src/justfile" }], () => true, run, error) }) });
  views.push(view); return { view, run, error };
}
it("reserves the overflow slot before the next row rather than pushing it down", () => {
  expect(gutterSlots(4, 60)).toEqual({ visible: 2, overflow: 2 });
  expect(gutterSlots(4, 20)).toEqual({ visible: 0, overflow: 4 });
  expect(gutterSlots(3, 60)).toEqual({ visible: 3, overflow: 0 });
});
it("ignores stale discovery and invalidates previously drawn recipe buttons on an edit", async () => {
  vi.mocked(api.justRecipes).mockResolvedValue({ recipes: [recipe] });
  const { view, run } = setup();
  await waitFor(() => expect(view.dom.querySelector(".cm-just-recipe")).not.toBeNull());
  const old = view.dom.querySelector<HTMLButtonElement>(".cm-just-recipe")!;
  old.click(); expect(run).toHaveBeenCalledWith("src/justfile", recipe);
  run.mockClear(); view.dispatch({ changes: { from: 0, to: 3, insert: "elsewhere" } });
  old.click(); expect(run).not.toHaveBeenCalled();
  expect(view.dom.querySelector(".cm-just-recipe")).toBeNull();
  let resolve!: (result: { recipes: JustRecipe[] }) => void;
  vi.mocked(api.justRecipes).mockImplementation(() => new Promise(done => { resolve = done; }));
  const second = setup(); second.view.dispatch({ changes: { from: 0, insert: "draft" } });
  resolve({ recipes: [recipe] }); await Promise.resolve(); await Promise.resolve();
  expect(second.view.dom.querySelector(".cm-just-recipe")).toBeNull();
});
it("passes parameter values as exact argv and cancels without starting a terminal", async () => {
  vi.mocked(api.openTerminal).mockResolvedValue({ id: "terminal" } as never);
  const prompt = vi.spyOn(window, "prompt").mockReturnValueOnce("a b").mockReturnValueOnce("$(touch unwanted)").mockReturnValueOnce("");
  const parameterized = { ...recipe, parameters: [{ name: "input", required: true, variadic: true }] };
  await launchJustRecipe("folder with spaces/justfile", parameterized);
  expect(api.openTerminal).toHaveBeenCalledWith({ title: "just build", cwd: "folder with spaces", argv: ["just", "--justfile", "justfile", "--", "build", "a b", "$(touch unwanted)"] });
  vi.mocked(api.openTerminal).mockClear(); prompt.mockReturnValue(null);
  expect(await launchJustRecipe("justfile", parameterized)).toBeNull(); expect(api.openTerminal).not.toHaveBeenCalled();
  prompt.mockReturnValue("");
  await launchJustRecipe("justfile", { ...recipe, parameters: [{ name: "input", required: false, variadic: false }] });
  expect(api.openTerminal).toHaveBeenCalledWith({ title: "just build", cwd: ".", argv: ["just", "--justfile", "justfile", "--", "build"] });
});
