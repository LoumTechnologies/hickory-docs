import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
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
import { SettingsView } from "./SettingsView";

// The backend contract (GET/PUT /api/settings/keys) is being added
// concurrently; these tests mock it at the client seam, which is also what
// keeps them honest about the one hard rule: GET never carries a full key.
let providers: ProviderKey[];
let puts: SettingsKeysPatch[];
let uiSettings: UiSettings;
let uiPuts: UiSettings[];

beforeEach(() => {
  providers = [
    { id: "anthropic", label: "Anthropic", configured: true, masked: "sk-a…f3" },
    { id: "openai", label: "OpenAI", configured: false, masked: null },
    { id: "openrouter", label: "OpenRouter", configured: false, masked: null },
    { id: "deepseek", label: "DeepSeek", configured: false, masked: null },
    { id: "xai", label: "xAI", configured: false, masked: null },
  ];
  puts = [];
  uiSettings = { window_title: null };
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
        uiSettings = { window_title: next.window_title };
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
    uiSettings = { window_title: "Old title" };
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
    uiSettings = { window_title: "Old title" };
    render(<SettingsView />);
    const input = await screen.findByLabelText("Window title");
    await waitFor(() => expect((input as HTMLInputElement).value).toBe("Old title"));
    fireEvent.change(input, { target: { value: "   " } });
    fireEvent.submit(input.closest("form")!);
    await waitFor(() => expect(uiPuts).toHaveLength(1));
    expect(uiPuts[0]).toEqual({ window_title: null });
  });
});
