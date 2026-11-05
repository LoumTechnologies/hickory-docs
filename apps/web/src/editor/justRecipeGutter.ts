import { RangeSet, StateEffect, StateField, type Extension } from "@codemirror/state";
import { EditorView, GutterMarker, ViewPlugin, gutter } from "@codemirror/view";
import { api } from "../api/client";

import type { JustRecipe } from "../api/types";
export type { JustRecipe } from "../api/types";

export interface RecipeFolder { line: number; path: string }

/** Pass exact argv values; a recipe argument never becomes shell syntax here. */
export async function launchJustRecipe(path: string, recipe: JustRecipe) {
  const args: string[] = [];
  for (const parameter of recipe.parameters) {
    let first = true;
    do {
      const required = first && parameter.required;
      const answer = window.prompt(`Run just ${recipe.name} — ${parameter.name}${required ? "" : " (leave empty to use defaults or finish)"}`, "");
      if (answer === null) return null;
      if (!required && answer === "") break;
      args.push(answer); first = false;
    } while (parameter.variadic);
    // Arguments are positional. Omitting a default also omits later defaults.
    if (first && !parameter.required) break;
  }
  const slash = path.lastIndexOf("/");
  return api.openTerminal({ title: `just ${recipe.name}`, cwd: slash < 0 ? "." : path.slice(0, slash), argv: ["just", "--justfile", path.slice(slash + 1), "--", recipe.name, ...args] });
}

class RecipeMarker extends GutterMarker {
  constructor(readonly path: string, readonly recipe: JustRecipe, readonly run: (path: string, recipe: JustRecipe) => void) { super(); }
  eq(other: RecipeMarker) { return other.path === this.path && JSON.stringify(other.recipe) === JSON.stringify(this.recipe); }
  toDOM() {
    const button = document.createElement("button");
    button.type = "button"; button.textContent = "▶";
    button.className = "cm-just-recipe";
    button.title = `just ${this.recipe.name}${this.recipe.doc ? ` — ${this.recipe.doc}` : ""}`;
    button.setAttribute("aria-label", `Run just ${this.recipe.name} in ${this.path.includes("/") ? this.path.slice(0, this.path.lastIndexOf("/")) : "."}`);
    button.onclick = () => this.run(this.path, this.recipe);
    return button;
  }
}

/** Saved filesystem identity must never be inferred from an edited tree label. */
export function justRecipeGutter(folders: RecipeFolder[], current: () => boolean, run: (path: string, recipe: JustRecipe) => void, error: (reason: string) => void): Extension {
  if (!folders.length) return [];
  const replace = StateEffect.define<{ line: number; path: string; recipes: JustRecipe[] }[]>();
  const rows = StateField.define<{ line: number; path: string; recipes: JustRecipe[] }[]>({
    create: () => [], update(value, transaction) {
      if (transaction.docChanged) return [];
      for (const effect of transaction.effects) if (effect.is(replace)) value = effect.value;
      return value;
    },
  });
  return [rows, gutter({ side: "after", class: "cm-just-gutter", markers: view => { const source = view.state.doc; return RangeSet.of(view.state.field(rows).flatMap(row =>
    row.line <= view.state.doc.lines ? row.recipes.map(recipe => new RecipeMarker(row.path, recipe, (path, selected) => { if (current() && view.state.doc === source) run(path, selected); }).range(view.state.doc.line(row.line).from)) : []), true); } }),
    ViewPlugin.fromClass(class {
      alive = true;
      constructor(view: EditorView) {
        const source = view.state.doc;
        void Promise.all(folders.map(async folder => {
          try { return { ...folder, recipes: (await api.justRecipes(folder.path)).recipes }; }
          catch (reason) { if (this.alive && current()) error(reason instanceof Error ? reason.message : String(reason)); return { ...folder, recipes: [] }; }
        })).then(result => { if (this.alive && view.state.doc === source && current()) view.dispatch({ effects: replace.of(result) }); });
      }
      destroy() { this.alive = false; }
    }),
    EditorView.theme({ ".cm-just-gutter": { minWidth: "20px" }, ".cm-just-recipe": { color: "var(--ok)", background: "transparent", border: "0", cursor: "pointer", font: "inherit" } }),
  ];
}
