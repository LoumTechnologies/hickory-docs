import { FILES_WINDOWS_KEY, loadFilesWindows } from "../lib/filesWindows";
import { FILES_PARALLAX_KEY, loadFilesParallax } from "../lib/filesParallax";
import { EDITOR_APPEARANCE_KEY, loadEditorAppearance } from "../lib/editorAppearance";
import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { installMockHandler } from "../api/client";
import type { ProviderKey, SettingsKeysPatch, UiSettings } from "../api/types";
import { RIBBON_STYLE_KEY, loadRibbonStyle } from "../lib/ribbonStyle";
import {
  RIBBON_VISIBILITY_KEY,
  loadRibbonVisibility,
} from "../lib/ribbonVisibility";
import { TAB_STYLE_KEY, loadTabStyle } from "../lib/tabStyle";
import { CHANNEL_WIDTH_KEY, loadChannelWidth } from "../lib/channelWidth";
import { THEME_KEY, loadTheme } from "../lib/theme";
import { WORD_MOTION_KEY, loadWordMotion } from "../lib/wordMotion";
import { SPATIAL_SIDES_KEY, loadSpatialSides } from "../lib/spatialSides";
import { SettingsView } from "./SettingsView";
import { UNWRAP_PARAGRAPHS_KEY, loadUnwrapParagraphs } from "../lib/unwrapParagraphs";

// The backend contract (GET/PUT /api/settings/keys) is being added
// concurrently; these tests mock it at the client seam, which is also what
// keeps them honest about the one hard rule: GET never carries a full key.
let providers: ProviderKey[];
let puts: SettingsKeysPatch[];
let uiSettings: UiSettings;
let uiPuts: UiSettings[];

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares setting choices while keeping credential drafts out of agent observations", async () => {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><SettingsView /></UiActionContext.Provider>);
  await screen.findByLabelText("OpenAI API key");
  fireEvent.change(screen.getByLabelText("OpenAI API key"), { target: { value: "draft-secret-never-context" } });
  expect(JSON.stringify(registry.observe("settings"))).not.toContain("draft-secret-never-context");
  const choice = registry.observe("settings-choices").actions.find(action => action.label === "Set Theme")!;
  await act(async () => { expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: choice.handle, arguments: { choice: "light" }, request_id: "theme" })).toMatchObject({ status: "result" }); });
  expect(loadTheme()).toBe("light");
  const credential = registry.observe("settings").actions.find(action => action.label === "Request credential change: OpenAI")!;
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: credential.handle, arguments: {}, request_id: "credential" })).toMatchObject({ status: "result", outcome: { status: "requires_input", request: "settings:credential:openai" } });
  expect(puts).toEqual([]);
  const toggle = registry.observe("settings-toggles").actions.find(action => action.label === "Set Right-hand line numbers")!;
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: toggle.handle, arguments: { checked: true }, request_id: "right-numbers" }); });
  expect(loadEditorAppearance().rightLineNumbers).toBe(true);
  const draft = registry.observe("settings-appearance").actions.find(action => action.label === "Set window title draft")!;
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: draft.handle, arguments: { title: "Investigation" }, request_id: "title-draft" }); });
  const save = registry.observe("settings-appearance").actions.find(action => action.label === "Save custom window title")!;
  await act(async () => { expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: save.handle, arguments: {}, request_id: "title-save" })).toMatchObject({ status: "result", outcome: { value: { saved: "Investigation" } } }); });
  expect(uiPuts).toContainEqual({ window_title: "Investigation" });
});

beforeEach(() => {
  localStorage.removeItem(FILES_WINDOWS_KEY);
  localStorage.removeItem(FILES_PARALLAX_KEY);
  localStorage.removeItem(EDITOR_APPEARANCE_KEY);
  providers = [
    { id: "anthropic", label: "Anthropic", configured: true, masked: "sk-a…f3" },
    { id: "openai", label: "OpenAI", configured: false, masked: null },
    { id: "openrouter", label: "OpenRouter", configured: false, masked: null },
    { id: "deepseek", label: "DeepSeek", configured: false, masked: null },
    { id: "xai", label: "xAI", configured: false, masked: null },
  ];
  puts = [];
  localStorage.removeItem(SPATIAL_SIDES_KEY);
  localStorage.removeItem(UNWRAP_PARAGRAPHS_KEY);
  uiSettings = { window_title: null, format_on_save: false };
  uiPuts = [];
  localStorage.removeItem(RIBBON_STYLE_KEY);
  localStorage.removeItem(TAB_STYLE_KEY);
  localStorage.removeItem(CHANNEL_WIDTH_KEY);
  localStorage.removeItem(THEME_KEY);
  delete document.documentElement.dataset.theme;
  installMockHandler(async (method, path, body) => {
    if (path === "/api/settings/ui") {
      if (method === "PUT") {
        const next = body as UiSettings;
        uiPuts.push(next);
        uiSettings = { ...uiSettings, ...next };
      }
      return uiSettings;
    }
    if (path !== "/api/settings/keys") throw new Error(`unexpected ${method} ${path}`);
    if (method === "PUT") {
      const patch = body as SettingsKeysPatch;
      puts.push(patch);
      providers = providers.map((p) =>
        p.id in patch
          ? patch[p.id] === null
            ? { ...p, configured: false, masked: null }
            : { ...p, configured: true, masked: `${String(patch[p.id]).slice(0, 4)}…` }
          : p,
      );
    }
    return { providers };
  });
});

// vitest runs without `globals`, so Testing Library's automatic cleanup never
// registers itself.
afterEach(() => {
  cleanup();
  installMockHandler(null as never);
});

describe("the settings view", () => {
  // Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
  it("remembers spacing, touch targets and right numbers independently", () => {
    const ui = render(<SettingsView />);
    expect(loadEditorAppearance()).toEqual({ density: "compact", touchTargets: false, rightLineNumbers: true });
    fireEvent.click(screen.getByRole("button", { name: "Comfortable" }));
    fireEvent.click(screen.getByRole("checkbox", { name: "Large Files touch targets" }));
    fireEvent.click(screen.getByRole("checkbox", { name: "Right-hand line numbers" }));
    expect(loadEditorAppearance()).toEqual({ density: "comfortable", touchTargets: true, rightLineNumbers: false });
    expect(document.documentElement.dataset.editorDensity).toBe("comfortable");
    expect(document.documentElement.dataset.rightLineNumbers).toBe("false");
    ui.unmount(); render(<SettingsView />);
    expect(screen.getByRole("button", { name: "Comfortable" }).getAttribute("aria-pressed")).toBe("true");
    expect((screen.getByRole("checkbox", { name: "Right-hand line numbers" }) as HTMLInputElement).checked).toBe(false);
  });
  it("defaults left context on and right context off, and remembers independent changes", () => {
    const ui = render(<SettingsView />);
    const left = () => screen.getByRole("checkbox", { name: "Left drawer" }) as HTMLInputElement;
    const right = () => screen.getByRole("checkbox", { name: "Right drawer" }) as HTMLInputElement;
    expect(left().checked).toBe(true); expect(right().checked).toBe(false);
    fireEvent.click(right()); expect(loadSpatialSides()).toEqual({ left: true, right: true });
    fireEvent.click(left()); expect(loadSpatialSides()).toEqual({ left: false, right: true });
    ui.unmount(); render(<SettingsView />);
    expect(left().checked).toBe(false); expect(right().checked).toBe(true);
  });
  // Guarantee: docs/guarantees/authoring/paragraphs-unwrap-by-default.md
  it("defaults paragraph unwrapping on and remembers switching it off", () => {
    const { unmount } = render(<SettingsView />);
    const checkbox = screen.getByRole("checkbox", { name: "Unwrap Markdown paragraphs" }) as HTMLInputElement;
    expect(checkbox.checked).toBe(true);
    fireEvent.click(checkbox);
    expect(loadUnwrapParagraphs()).toBe(false);
    unmount();
    render(<SettingsView />);
    expect((screen.getByRole("checkbox", { name: "Unwrap Markdown paragraphs" }) as HTMLInputElement).checked).toBe(false);
  });
  it("lists all five providers with their configured state", async () => {
    render(<SettingsView />);
    for (const label of ["Anthropic", "OpenAI", "OpenRouter", "DeepSeek", "xAI"]) {
      expect(await screen.findByText(label)).toBeTruthy();
    }
    // The configured row shows its masked hint; the rest say so plainly.
    expect(screen.getByText("configured · sk-a…f3")).toBeTruthy();
    expect(screen.getAllByText("not set")).toHaveLength(4);
  });

  it("takes keys through password inputs only — a key is never shown back", async () => {
    render(<SettingsView />);
    const input = await screen.findByLabelText("Anthropic API key");
    expect(input.getAttribute("type")).toBe("password");
    // The masked hint is on screen; nothing resembling a full key can be,
    // because GET does not return one and the view renders only GET's data.
    expect(document.body.textContent).toContain("sk-a…f3");
  });

  it("PUTs only the provider the user changed", async () => {
    render(<SettingsView />);
    const input = await screen.findByLabelText("OpenAI API key");
    fireEvent.change(input, { target: { value: "sk-openai-new" } });
    const row = input.closest("form")!;
    fireEvent.submit(row);

    await waitFor(() => expect(puts).toHaveLength(1));
    // Exactly one field — untouched providers are absent, not re-sent.
    expect(puts[0]).toEqual({ openai: "sk-openai-new" });
    // The row now reads as configured, and the draft is gone from the input.
    expect(await screen.findByText(/configured · sk-o/)).toBeTruthy();
    expect((input as HTMLInputElement).value).toBe("");
  });

  it("clears a key by PUTting null for that provider only", async () => {
    render(<SettingsView />);
    fireEvent.click(await screen.findByRole("button", { name: "Clear" }));
    await waitFor(() => expect(puts).toHaveLength(1));
    expect(puts[0]).toEqual({ anthropic: null });
    expect(await screen.findAllByText("not set")).toHaveLength(5);
  });

  it("says which provider the agent will pick when exactly one key is set", async () => {
    render(<SettingsView />);
    expect(
      await screen.findByText(/the agent uses Anthropic automatically/),
    ).toBeTruthy();
  });
});

describe("the appearance section", () => {
  it("offers the three themes with dark pressed by default", async () => {
    render(<SettingsView />);
    expect(await screen.findByRole("group", { name: "Theme" })).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Dark" }).getAttribute("aria-pressed"),
    ).toBe("true");
    for (const label of ["Warm dark", "Light"]) {
      expect(
        screen.getByRole("button", { name: label }).getAttribute("aria-pressed"),
      ).toBe("false");
    }
  });

  it("applies a picked theme immediately and persists it", async () => {
    render(<SettingsView />);
    fireEvent.click(await screen.findByRole("button", { name: "Light" }));
    // Applied on the spot — the page restyles without a reload…
    expect(document.documentElement.dataset.theme).toBe("light");
    // …and persisted under the same key the pre-bundle script reads.
    expect(localStorage.getItem(THEME_KEY)).toBe("light");
    expect(loadTheme()).toBe("light");
    fireEvent.click(screen.getByRole("button", { name: "Warm dark" }));
    expect(document.documentElement.dataset.theme).toBe("warm-dark");
    expect(loadTheme()).toBe("warm-dark");
  });

  it("shows the three display toggles that used to live in the toolbar", async () => {
    render(<SettingsView />);
    expect(await screen.findByRole("group", { name: "Lineage style" })).toBeTruthy();
    expect(screen.getByRole("group", { name: "Tab placement" })).toBeTruthy();
    expect(screen.getByRole("group", { name: "Channel width" })).toBeTruthy();
    for (const label of ["Ribbons", "Braces", "Top", "Side", "Narrow", "Normal", "Wide"]) {
      expect(screen.getByRole("button", { name: label })).toBeTruthy();
    }
  });

  it("reflects the persisted defaults as the pressed choices", async () => {
    render(<SettingsView />);
    const braces = await screen.findByRole("button", { name: "Braces" });
    expect(braces.getAttribute("aria-pressed")).toBe("true");
    expect(
      screen.getByRole("button", { name: "Top" }).getAttribute("aria-pressed"),
    ).toBe("true");
    expect(
      screen.getByRole("button", { name: "Normal" }).getAttribute("aria-pressed"),
    ).toBe("true");
  });

  it("persists each toggle through its lib so the workspace re-reads it on remount", async () => {
    render(<SettingsView />);
    fireEvent.click(await screen.findByRole("button", { name: "Ribbons" }));
    expect(loadRibbonStyle()).toBe("bands");
    fireEvent.click(screen.getByRole("button", { name: "Side" }));
    expect(loadTabStyle()).toBe("side");
    fireEvent.click(screen.getByRole("button", { name: "Wide" }));
    expect(loadChannelWidth()).toBe(72);
    // The keys are unchanged from the toolbar era — stored choices survive.
    expect(localStorage.getItem(RIBBON_STYLE_KEY)).toBe("bands");
    expect(localStorage.getItem(TAB_STYLE_KEY)).toBe("side");
    expect(localStorage.getItem(CHANNEL_WIDTH_KEY)).toBe("72");
  });

  it("offers a lineage-visibility choice, defaulting to the caret", async () => {
    render(<SettingsView />);
    expect(
      await screen.findByRole("group", { name: "Lineage visibility" }),
    ).toBeTruthy();
    expect(
      screen
        .getByRole("button", { name: "With the caret" })
        .getAttribute("aria-pressed"),
    ).toBe("true");
    // The old always-on reading is a setting, not a thing that was removed.
    fireEvent.click(screen.getByRole("button", { name: "Always" }));
    expect(loadRibbonVisibility()).toBe("always");
    expect(localStorage.getItem(RIBBON_VISIBILITY_KEY)).toBe("always");
  });

  it("offers a word-navigation choice, defaulting to whole words", async () => {
    render(<SettingsView />);
    expect(await screen.findByRole("group", { name: "Word navigation" })).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Whole words" }).getAttribute("aria-pressed"),
    ).toBe("true");
    fireEvent.click(screen.getByRole("button", { name: "Subwords" }));
    expect(loadWordMotion()).toBe("subword");
    expect(localStorage.getItem(WORD_MOTION_KEY)).toBe("subword");
  });

  it("saves a custom window title through PUT /api/settings/ui", async () => {
    render(<SettingsView />);
    const input = await screen.findByLabelText("Window title");
    fireEvent.change(input, { target: { value: "My Lab Notebook" } });
    fireEvent.submit(input.closest("form")!);
    await waitFor(() => expect(uiPuts).toHaveLength(1));
    expect(uiPuts[0]).toEqual({ window_title: "My Lab Notebook" });
    expect(await screen.findByText("custom")).toBeTruthy();
  });

  it("clears the custom title by PUTting null", async () => {
    uiSettings = { window_title: "Old title", format_on_save: false };
    render(<SettingsView />);
    // The saved title arrives in the input, and a Clear button with it. The
    // configured key row has its own Clear, so scope to the title's form.
    const input = await screen.findByLabelText("Window title");
    await waitFor(() => expect((input as HTMLInputElement).value).toBe("Old title"));
    fireEvent.click(within(input.closest("form")!).getByRole("button", { name: "Clear" }));
    await waitFor(() => expect(uiPuts).toHaveLength(1));
    expect(uiPuts[0]).toEqual({ window_title: null });
    expect(await screen.findByText("default (folder name)")).toBeTruthy();
  });

  it("treats a blanked-out field as clearing, not a title of nothing", async () => {
    uiSettings = { window_title: "Old title", format_on_save: false };
    render(<SettingsView />);
    const input = await screen.findByLabelText("Window title");
    await waitFor(() => expect((input as HTMLInputElement).value).toBe("Old title"));
    fireEvent.change(input, { target: { value: "   " } });
    fireEvent.submit(input.closest("form")!);
    await waitFor(() => expect(uiPuts).toHaveLength(1));
    expect(uiPuts[0]).toEqual({ window_title: null });
  });

  // Protects docs/guarantees/editor-intelligence/save-can-format-first.md
  it("turns format-on-save on through PUT /api/settings/ui", async () => {
    render(<SettingsView />);
    const box = (await screen.findByLabelText("Format on save")) as HTMLInputElement;
    expect(box.checked).toBe(false);
    fireEvent.click(box);
    await waitFor(() => expect(uiPuts.at(-1)).toEqual({ format_on_save: true }));
    await waitFor(() => expect(box.checked).toBe(true));
  });

  // Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("keeps saved-file recovery off by default and persists an opt-in", async () => {
    render(<SettingsView />);
    const box = (await screen.findByLabelText("Retain unsaved changes")) as HTMLInputElement;
    expect(box.checked).toBe(false);
    fireEvent.click(box);
    await waitFor(() =>
      expect(uiPuts.at(-1)).toEqual({ retain_unsaved_saved_files: true }),
    );
    expect(box.checked).toBe(true);
  });
});

it("groups Files experiments together, defaults them off, and saves opt-ins", () => {
  render(<SettingsView />);
  const experiments = within(screen.getByRole("region", { name: "Experiments" }));
  const windows = experiments.getByRole("checkbox", { name: "Show windows in Files pane" }) as HTMLInputElement;
  const parallax = experiments.getByRole("checkbox", { name: "Files pane parallax" }) as HTMLInputElement;
  expect(windows.checked).toBe(false);
  expect(parallax.checked).toBe(false);
  fireEvent.click(windows);
  fireEvent.click(parallax);
  expect(loadFilesWindows()).toBe(true);
  expect(loadFilesParallax()).toBe(true);
});
