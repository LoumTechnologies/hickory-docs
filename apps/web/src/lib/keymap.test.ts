// docs/guarantees/editor-intelligence/every-shortcut-is-a-setting.md
import { describe, expect, it } from "vitest";

import {
  ACTIONS,
  PROFILES,
  conflicts,
  formatBinding,
  nativeAccelerators,
  parseBinding,
  parseStroke,
  profileBinding,
  resolve,
  strokeMatches,
  toCodeMirror,
  toNativeAccelerator,
} from "./keymap";

const ev = (key: string, mods: Partial<{ ctrl: boolean; meta: boolean; alt: boolean; shift: boolean }> = {}) => ({
  key,
  ctrlKey: !!mods.ctrl,
  metaKey: !!mods.meta,
  altKey: !!mods.alt,
  shiftKey: !!mods.shift,
});

describe("one binding, three spellings", () => {
  it("parses modifiers in any order and letters case-insensitively", () => {
    expect(parseStroke("Ctrl+Shift+L")).toEqual({ mod: false, ctrl: true, alt: false, shift: true, key: "l" });
    expect(parseStroke("shift+alt+f")).toEqual({ mod: false, ctrl: false, alt: true, shift: true, key: "f" });
    expect(parseStroke("Mod+,")).toEqual({ mod: true, ctrl: false, alt: false, shift: false, key: "," });
    expect(parseStroke("F8")).toEqual({ mod: false, ctrl: false, alt: false, shift: false, key: "F8" });
    expect(parseStroke("+")?.key).toBe("+");
    expect(parseStroke("Ctrl++")?.key).toBe("+");
    expect(parseStroke("Ctrl+")).toBeNull();
    expect(parseStroke("Ctrl+A+B")).toBeNull();
  });

  it("takes a chord of two strokes and refuses three", () => {
    expect(parseBinding("Mod+K Mod+D")?.length).toBe(2);
    expect(parseBinding("Mod+K Mod+D Mod+E")).toBeNull();
    expect(parseBinding("")).toBeNull();
  });

  it("spells for people, for CodeMirror, and for the menu bar", () => {
    expect(formatBinding("Mod+Shift+L", false)).toBe("Ctrl+Shift+L");
    expect(formatBinding("Mod+Shift+L", true)).toBe("⇧⌘L");
    expect(formatBinding("Mod+K Mod+D", false)).toBe("Ctrl+K Ctrl+D");
    expect(toCodeMirror("Mod+Shift+L")).toBe("Mod-Shift-l");
    expect(toCodeMirror("Shift+Alt+F")).toBe("Alt-Shift-f");
    expect(toCodeMirror("Mod+K Mod+D")).toBe("Mod-k Mod-d");
    expect(toNativeAccelerator("Mod+Shift+S")).toBe("CmdOrCtrl+Shift+S");
    expect(toNativeAccelerator("Mod+,")).toBe("CmdOrCtrl+,");
    // A chord cannot drive a native menu item; an unbound action has none.
    expect(toNativeAccelerator("Mod+K Mod+D")).toBeNull();
    expect(toNativeAccelerator("")).toBeNull();
  });

  it("matches events with Mod as Ctrl here and Cmd on a Mac", () => {
    const s = parseStroke("Mod+Shift+L")!;
    expect(strokeMatches(ev("L", { ctrl: true, shift: true }), s, false)).toBe(true);
    expect(strokeMatches(ev("L", { meta: true, shift: true }), s, true)).toBe(true);
    expect(strokeMatches(ev("L", { meta: true, shift: true }), s, false)).toBe(false);
    expect(strokeMatches(ev("l", { ctrl: true }), s, false)).toBe(false);
    const plain = parseStroke("Shift+D")!;
    expect(strokeMatches(ev("D", { shift: true }), plain, false)).toBe(true);
    expect(strokeMatches(ev("D", { shift: true, ctrl: true }), plain, false)).toBe(false);
    const f8 = parseStroke("F8")!;
    expect(strokeMatches(ev("F8"), f8, false)).toBe(true);
  });
});

describe("the profiles", () => {
  it("bind every action under every profile, unbound only on purpose", () => {
    for (const profile of PROFILES) {
      for (const spec of ACTIONS) {
        const binding = profileBinding(spec, profile.id);
        if (binding === "") continue;
        expect(parseBinding(binding), `${profile.id} ${spec.id} ${binding}`).not.toBeNull();
      }
    }
  });

  it("differ where the IDEs differ, and share what dired is", () => {
    const vs = resolve({ profile: "visualstudio", overrides: {} });
    const jb = resolve({ profile: "jetbrains", overrides: {} });
    const code = resolve({ profile: "vscode", overrides: {} });
    expect(vs.get("editor.format")).toBe("Mod+K Mod+D");
    expect(jb.get("editor.format")).toBe("Mod+Alt+L");
    expect(code.get("editor.format")).toBe("Shift+Alt+F");
    expect(jb.get("editor.rename")).toBe("Shift+F6");
    expect(vs.get("file.saveAll")).toBe("Mod+Shift+S");
    for (const map of [vs, jb, code]) expect(map.get("tree.delete")).toBe("Shift+D");
  });

  it("have no two actions of one scope on the same keys", () => {
    for (const profile of PROFILES) {
      expect(conflicts(resolve({ profile: profile.id, overrides: {} })), profile.id).toEqual([]);
    }
  });

  it("let one action be overridden or unbound without touching the rest", () => {
    const map = resolve({ profile: "jetbrains", overrides: { "editor.format": "Mod+Shift+I", "view.problems": "" } });
    expect(map.get("editor.format")).toBe("Mod+Shift+I");
    expect(map.get("view.problems")).toBe("");
    expect(map.get("editor.rename")).toBe("Shift+F6");
    expect(conflicts(resolve({ profile: "hickory", overrides: { "editor.format": "Mod+F" } }))).toEqual([
      ["editor.find", "editor.format"],
    ]);
  });

  it("derive the menu bar's accelerators, with unbound and chorded items as null", () => {
    const accel = nativeAccelerators(resolve({ profile: "jetbrains", overrides: {} }));
    expect(accel["save"]).toBe("CmdOrCtrl+S");
    expect(accel["settings"]).toBe("CmdOrCtrl+Alt+S");
    expect(accel["save-all"]).toBeNull();
    expect(Object.keys(accel)).toContain("zoom-tab-reset");
  });
});
