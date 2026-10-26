<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Every Shortcut Is A Setting

For: Authors using completion, diagnostics and refactoring inside literate source.

Given the app's keyboard shortcuts — the menu bar's (new, open, save,
terminal, files, settings, zoom, insert), the editor's (find, format,
rename, code action, add next occurrence, select all occurrences), the
workspace's (find in folder, problems, debug continue) and the tree's
dired and column-selection keys — when a person opens Settings → Keyboard, then every one of
them is listed with the key it is on, a **profile** binds them all the way
one IDE does (Hickory's own keys, which are VS Code's; VS Code; JetBrains;
Visual Studio), and any single action can be changed by pressing its key
and typing a new one (a chord of two strokes included), unbound, or reset
to the profile's. The choice and the overrides persist beside the window
title, so the desktop app and `hick up` agree; the editor's and the
tree's bindings take effect at once for editors opened after the change,
and the menu bar's at the next launch, which the page says.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/every-shortcut-is-a-setting.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-4f41c9184a60"></a>
## `apps/web/src/lib/keymap.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Every Shortcut Is A Setting](contract.md).

<a id="f-4f41c9184a60-ev"></a>
### `ev`

<storycode:copy id="f-4f41c9184a60-ev">
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

</storycode:copy>
<a id="f-4f41c9184a60-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4f41c9184a60-expression_statement">
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

</storycode:copy>
<a id="f-4f41c9184a60-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4f41c9184a60-expression_statement-2">
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
    expect(code.get("debug.continue")).toBe("F5");
    expect(code.get("debug.stop")).toBe("Shift+F5");
    expect(code.get("editor.definition")).toBe("F12");
    expect(jb.get("debug.start")).toBe("Shift+F9");
    expect(jb.get("debug.continue")).toBe("F9");
    expect(jb.get("debug.over")).toBe("F8");
    expect(jb.get("debug.in")).toBe("F7");
    expect(jb.get("debug.stop")).toBe("Ctrl+F2");
    expect(jb.get("editor.definition")).toBe("Mod+B");
    expect(jb.get("editor.references")).toBe("Alt+F7");
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
</storycode:copy>

### Assemble `apps/web/src/lib/keymap.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/keymap.test.ts"><storycode:paste select="#f-4f41c9184a60-ev"/><storycode:paste select="#f-4f41c9184a60-expression_statement"/><storycode:paste select="#f-4f41c9184a60-expression_statement-2"/></storycode:file>

<a id="output-17918b3a4e86"></a>
## `apps/web/src/views/KeyboardSection.controls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Every Shortcut Is A Setting](contract.md).

<a id="f-17918b3a4e86-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-17918b3a4e86-expression_statement">
// Guarantee: docs/guarantees/editor-intelligence/every-shortcut-is-a-setting.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { KeyboardSection } from "./KeyboardSection";
import { ACTIONS, DEFAULT_SETTINGS, keymapSettings, setKeymapSettings } from "../lib/keymap";
import { api } from "../api/client";
afterEach(() => { cleanup(); vi.restoreAllMocks(); setKeymapSettings(DEFAULT_SETTINGS); });
</storycode:copy>
<a id="f-17918b3a4e86-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-17918b3a4e86-expression_statement-2">
it("shares persistent profiles and individual bindings with native controls, rejects malformed chords and retires handles", async () => {
  const save = vi.spyOn(api, "saveSettingsUi").mockResolvedValue({ window_title: null, format_on_save: false });
  setKeymapSettings(DEFAULT_SETTINGS);
  const registry = new UiActions({ workspace: "settings", window: "keyboard" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><KeyboardSection /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => {
    const total = registry.observe(group).coverage.total;
    return Array.from({ length: Math.ceil(total / 64) }, (_, index) => registry.observe(group, 64, index * 64).actions).flat();
  });
  let serial = 0;
  const invoke = async (label: string, args: Record<string, string>) => {
    const action = actions().find(action => action.label === label)!;
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, {kind:"agent", id:"keyboard-agent"}, { handle: action.handle, arguments: args, request_id: `key-${serial++}` }); });
    return result;
  };
  const stale = actions().find(action => action.label === "Set keyboard profile")!;
  await invoke("Set keyboard profile", { profile: "vscode" });
  expect(screen.getByLabelText("Profile")).toHaveProperty("value", "vscode");
  expect(save).toHaveBeenCalled();
  const spec = ACTIONS.find(spec => spec.scope === "editor")!;
  await invoke(`Change key for ${spec.label}`, { binding: "Mod+Shift+K" });
  expect(keymapSettings().overrides[spec.id]).toBe("Mod+Shift+K");
  const menu = ACTIONS.find(spec => spec.scope === "menu")!;
  const before = save.mock.calls.length;
  await invoke(`Change key for ${menu.label}`, { binding: "Mod+K Mod+C" });
  expect(save).toHaveBeenCalledTimes(before);
  await invoke(`Reset key for ${spec.label}`, {});
  expect(keymapSettings().overrides[spec.id]).toBeUndefined();
  fireEvent.change(screen.getByLabelText("Profile"), { target: { value: "jetbrains" } });
  await waitFor(() => expect(keymapSettings().profile).toBe("jetbrains"));
  expect(await registry.act(registry.scope, { kind:"agent", id:"keyboard-agent" }, { handle: stale.handle, arguments: { profile:"hickory" }, request_id:"stale" })).toMatchObject({status:"unavailable"});
});
</storycode:copy>

### Assemble `apps/web/src/views/KeyboardSection.controls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/KeyboardSection.controls.test.tsx"><storycode:paste select="#f-17918b3a4e86-expression_statement"/><storycode:paste select="#f-17918b3a4e86-expression_statement-2"/></storycode:file>

<a id="output-63f1f3c4618f"></a>
## `apps/web/src/views/KeyboardSection.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Every Shortcut Is A Setting](contract.md).

<a id="f-63f1f3c4618f-SCOPES"></a>
### `SCOPES`

Settings → Keyboard: the profile, and every action's binding.
A profile is a starting point, not a mode: picking JetBrains binds every
action the JetBrains way, and any single one can then be changed or
unbound, which is recorded as an override on top of the profile. "Reset"
on a row drops its override; "Reset all" drops them all.
Menu-bar actions are the shell's, built at launch, so a change to one of
them is said to land at the next launch rather than pretended live. See

<storycode:copy id="f-63f1f3c4618f-SCOPES">
// Settings → Keyboard: the profile, and every action's binding.
//
// A profile is a starting point, not a mode: picking JetBrains binds every
// action the JetBrains way, and any single one can then be changed or
// unbound, which is recorded as an override on top of the profile. "Reset"
// on a row drops its override; "Reset all" drops them all.
//
// Menu-bar actions are the shell's, built at launch, so a change to one of
// them is said to land at the next launch rather than pretended live. See
// docs/guarantees/editor-intelligence/every-shortcut-is-a-setting.md.
import { useEffect, useId, useMemo, useState } from "react";
import type { KeyboardEvent } from "react";

import {
  ACTIONS,
  PROFILES,
  conflicts,
  formatBinding,
  isMac,
  keymapSettings,
  onKeymapChange,
  parseBinding,
  profileBinding,
  resolve,
  saveKeymap,
} from "../lib/keymap";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import type { ActionSpec, KeymapSettings, Profile, Scope } from "../lib/keymap";

const SCOPES: { id: Scope; title: string; note?: string }[] = [
  { id: "editor", title: "Editor" },
  { id: "workspace", title: "Workspace" },
  { id: "tree", title: "File tree (dired)", note: "With a tree row focused." },
  { id: "menu", title: "Menu bar", note: "Takes effect at the next launch of the desktop app." },
];

</storycode:copy>
<a id="f-63f1f3c4618f-bindingFromEvent"></a>
### `bindingFromEvent`

A keydown as a binding string, or null for a bare modifier.

<storycode:copy id="f-63f1f3c4618f-bindingFromEvent">
/** A keydown as a binding string, or null for a bare modifier. */
export function bindingFromEvent(event: KeyboardEvent, mac = isMac()): string | null {
  const key = event.key;
  if (["Control", "Shift", "Alt", "Meta", "OS"].includes(key)) return null;
  const parts: string[] = [];
  if (mac ? event.metaKey : event.ctrlKey) parts.push("Mod");
  if (mac && event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  parts.push(key === " " ? "Space" : key.length === 1 ? key.toUpperCase() : key);
  return parts.join("+");
}

</storycode:copy>
<a id="f-63f1f3c4618f-KeyboardSection"></a>
### `KeyboardSection`

<storycode:copy id="f-63f1f3c4618f-KeyboardSection">
export function KeyboardSection() {
  const parent = `keyboard:${useId()}`;
  const [saving, setSaving] = useState(false);
  const [settings, setSettings] = useState<KeymapSettings>(() => keymapSettings());
  const [error, setError] = useState<string | null>(null);
  /** The action whose next keystroke is being recorded, and the first
   * stroke of a chord if one has been typed. */
  const [recording, setRecording] = useState<{ id: string; first: string | null } | null>(null);
  useEffect(() => onKeymapChange(() => setSettings(keymapSettings())), []);

  const bindings = useMemo(() => resolve(settings), [settings]);
  const clashes = useMemo(() => conflicts(bindings), [bindings]);
  const clashing = new Set(clashes.flat());

  const save = async (next: KeymapSettings): Promise<boolean> => {
    setError(null); setRecording(null); setSaving(true); setSettings(next);
    try { await saveKeymap(next); return true; }
    catch (e) { setError(e instanceof Error ? e.message : String(e)); return false; }
    finally { setSaving(false); }
  };
  const setProfile = (profile: Profile) => save({ profile, overrides: settings.overrides });
  const override = (id: string, binding: string) => save({ ...settings, overrides: { ...settings.overrides, [id]: binding } });
  const reset = (id: string) => {
    const overrides = { ...settings.overrides }; delete overrides[id];
    return save({ ...settings, overrides });
  };
  const revision = useUiRevision(() => [settings, recording, saving, error]);
  useUiReading("keyboard settings", { profile: settings.profile, saving, error, recording, conflicts: clashes,
    bindings: ACTIONS.map(spec => ({ id: spec.id, label: spec.label, scope: spec.scope, binding: bindings.get(spec.id) ?? "", overridden: spec.id in settings.overrides })), menu_changes: "next desktop launch" });
  const result = async (pending: Promise<boolean>): Promise<UiOutcome> => await pending
    ? { status: "completed", value: { persisted: true, menu_changes: "next desktop launch" } }
    : { status: "unavailable", reason: "Keyboard preference persistence failed; inspect current settings error" };
  const command = (key: string, label: string, enabled: boolean, parameters: UiCommand["parameters"], execute: UiCommand["execute"]) => ({ key: `${parent}:${key}`, command: {
    label, parent, subject: `${parent}:${key}`, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: enabled && !saving, reason: saving ? "Keyboard preferences are saving" : undefined }), execute,
  } });
  useUiCommands([
    command("profile", "Set keyboard profile", true, { profile: { type: "string", maxLength: 32, choices: PROFILES.map(p => p.id) } }, args => result(setProfile(args.profile as Profile))),
    command("reset-all", "Reset all keyboard overrides", Object.keys(settings.overrides).length > 0, {}, () => result(save({ profile: settings.profile, overrides: {} }))),
    ...ACTIONS.flatMap(spec => [
      command(`binding:${spec.id}`, `Change key for ${spec.label}`, true, { binding: { type: "string", maxLength: 128 } }, async args => {
        const binding = args.binding as string; const parsed = parseBinding(binding);
        if (binding && (!parsed || spec.scope === "menu" && parsed.length > 1)) return { status: "unavailable", reason: "Binding is unreadable or uses an unsupported menu chord" };
        return result(override(spec.id, binding));
      }),
      command(`reset:${spec.id}`, `Reset key for ${spec.label}`, spec.id in settings.overrides, {}, () => result(reset(spec.id))),
    ]),
    ...(recording ? [command("cancel", "Cancel keyboard recording", true, {}, async () => { setRecording(null); return { status: "completed", value: { recording: false } }; })] : []),
  ]);

  const onRecordKey = (spec: ActionSpec, event: KeyboardEvent) => {
    event.preventDefault();
    event.stopPropagation();
    if (event.key === "Escape") {
      setRecording(null);
      return;
    }
    const stroke = bindingFromEvent(event);
    if (!stroke) return;
    if (recording?.first) {
      override(spec.id, `${recording.first} ${stroke}`);
      setRecording(null);
      return;
    }
    // A menu-bar action cannot carry a chord; anything else may, and a
    // second stroke within the moment makes one.
    if (spec.scope === "menu") {
      override(spec.id, stroke);
      setRecording(null);
      return;
    }
    setRecording({ id: spec.id, first: stroke });
    window.setTimeout(() => {
      setRecording((current) => {
        if (current?.id === spec.id && current.first === stroke) {
          override(spec.id, stroke);
          return null;
        }
        return current;
      });
    }, 900);
  };

  return (
    <section className="settings__appearance" aria-label="Keyboard">
      <h2 className="settings__section-title">Keyboard</h2>
      <div className="settings__rows">
        <div className="settings-row settings-row--appearance">
          <div className="settings-row__who">
            <label className="settings-row__label" htmlFor="keymap-profile">
              Profile
            </label>
            <p className="settings-row__hint">
              A starting point. Change any single key below; a changed key stays yours when the
              profile changes.
            </p>
          </div>
          <div className="settings-row__actions">
            <select
              id="keymap-profile"
              disabled={saving}
              value={settings.profile}
              onChange={(event) => setProfile(event.target.value as Profile)}
            >
              {PROFILES.map((profile) => (
                <option key={profile.id} value={profile.id}>
                  {profile.label}
                </option>
              ))}
            </select>
            <button
              type="button"
              disabled={saving || Object.keys(settings.overrides).length === 0}
              onClick={() => save({ profile: settings.profile, overrides: {} })}
            >
              Reset all
            </button>
          </div>
        </div>
        {error && <p className="error">{error}</p>}
        {clashes.length > 0 && (
          <p className="settings__warning" role="status">
            Two actions share a key:{" "}
            {clashes.map(([a, b]) => `${label(a)} and ${label(b)}`).join("; ")}. The first wins.
          </p>
        )}
        {SCOPES.map((scope) => (
          <div key={scope.id} className="keymap-scope">
            <h3 className="keymap-scope__title">
              {scope.title}
              {scope.note && <span className="keymap-scope__note"> — {scope.note}</span>}
            </h3>
            <table className="keymap-table">
              <tbody>
                {ACTIONS.filter((spec) => spec.scope === scope.id).map((spec) => {
                  const binding = bindings.get(spec.id) ?? "";
                  const overridden = spec.id in settings.overrides;
                  const isRecording = recording?.id === spec.id;
                  return (
                    <tr key={spec.id} className={clashing.has(spec.id) ? "keymap-row--clash" : undefined}>
                      <td className="keymap-table__label">{spec.label}</td>
                      <td className="keymap-table__key">
                        <button
                          type="button"
                          disabled={saving}
                          className={`keymap-key${isRecording ? " keymap-key--recording" : ""}`}
                          aria-label={`Change key for ${spec.label}`}
                          onClick={() => setRecording({ id: spec.id, first: null })}
                          onKeyDown={(event) => isRecording && onRecordKey(spec, event)}
                          onBlur={() => isRecording && setRecording(null)}
                        >
                          {isRecording
                            ? recording?.first
                              ? `${formatBinding(recording.first)} …`
                              : "Press keys"
                            : binding
                              ? formatBinding(binding)
                              : "Unbound"}
                        </button>
                      </td>
                      <td className="keymap-table__actions">
                        {binding && !isRecording && (
                          <button type="button" disabled={saving} onClick={() => override(spec.id, "")}>
                            Unbind
                          </button>
                        )}
                        {overridden && (
                          <button
                            type="button"
                            disabled={saving}
                            onClick={() => reset(spec.id)}
                            data-tip={`Back to ${profileBinding(spec, settings.profile) || "unbound"}`}
                          >
                            Reset
                          </button>
                        )}
                        {binding && !parseBinding(binding) && <span className="error">unreadable</span>}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        ))}
      </div>
    </section>
  );
}

</storycode:copy>
<a id="f-63f1f3c4618f-label"></a>
### `label`

<storycode:copy id="f-63f1f3c4618f-label">
function label(id: string): string {
  return ACTIONS.find((spec) => spec.id === id)?.label ?? id;
}
</storycode:copy>

### Assemble `apps/web/src/views/KeyboardSection.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/KeyboardSection.tsx"><storycode:paste select="#f-63f1f3c4618f-SCOPES"/><storycode:paste select="#f-63f1f3c4618f-bindingFromEvent"/><storycode:paste select="#f-63f1f3c4618f-KeyboardSection"/><storycode:paste select="#f-63f1f3c4618f-label"/></storycode:file>

## Test the behavior: `the_keymap_and_native_accelerators_round_trip`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_ui_settings.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-c816a698604d-the_keymap_and_native_accelerators_round_trip"></a>
### `the_keymap_and_native_accelerators_round_trip`

keymap and the resolved menu accelerators ride the same file as the title.

<storycode:copy id="f-c816a698604d-the_keymap_and_native_accelerators_round_trip">
// docs/guarantees/editor-intelligence/every-shortcut-is-a-setting.md: the
// keymap and the resolved menu accelerators ride the same file as the title.
#[tokio::test]
async fn the_keymap_and_native_accelerators_round_trip() {
    let session = start().await;
    let (status, body) = put(
        &session,
        "/api/settings/ui",
        json!({
            "keymap": { "profile": "jetbrains", "overrides": { "editor.format": "Ctrl+Alt+L" } },
            "native_accelerators": { "save": "CmdOrCtrl+S", "save-all": null, "settings": "CmdOrCtrl+Alt+S" }
        }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["keymap"]["profile"], "jetbrains");
    assert_eq!(body["keymap"]["overrides"]["editor.format"], "Ctrl+Alt+L");
    assert_eq!(body["native_accelerators"]["save"], "CmdOrCtrl+S");
    assert!(
        body["native_accelerators"].get("save-all").is_none(),
        "null drops the entry: {body}"
    );

    let (_, again) = get(&session, "/api/settings/ui").await;
    assert_eq!(again["keymap"]["profile"], "jetbrains");
    assert_eq!(again["native_accelerators"]["settings"], "CmdOrCtrl+Alt+S");
    let stored = UiStore::load(&session.ui_path).unwrap();
    assert_eq!(
        stored.native_accelerators.get("save").map(String::as_str),
        Some("CmdOrCtrl+S")
    );

    let (status, body) = put(&session, "/api/settings/ui", json!({ "keymap": "vscode" })).await;
    assert_eq!(status, 400, "{body}");
    let (status, _) = put(&session, "/api/settings/ui", json!({ "keymap": null })).await;
    assert_eq!(status, 200);
    let (_, cleared) = get(&session, "/api/settings/ui").await;
    assert!(cleared["keymap"].is_null());
}
</storycode:copy>

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../mechanisms/apps/desktop/src-tauri/src/lib.rs.md#output-42caba993099)
- [apps/web/src/debug/DebugStrip.tsx](../../../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-09306f4ec1a5)
- [apps/web/src/editor/multiCursor.ts](../../../mechanisms/apps/web/src/editor/multiCursor.ts.md#output-08df9c7d9da5)
- [apps/web/src/lib/keymap.ts](../../../mechanisms/apps/web/src/lib/keymap.ts.md#output-4d0aa846a85d)
- [apps/web/src/lsp/cmLspFeatures.ts](../save-can-format-first/contract.md#output-a12307572b4c)
- [apps/web/src/main.tsx](../../verification/predictions-precede-managed-observation/implementation/apps/web/src/main.tsx.md#output-44decff65943)
- [apps/web/src/shell/ShellView.tsx](../../../mechanisms/apps/web/src/shell/ShellView.tsx.md#output-e5f8a7657f14)
- [apps/web/src/shell/dired.ts](../../authoring/the-tree-is-a-dired/contract.md#output-fabcac2ae554)
- [apps/web/src/shell/useTreeNavigation.ts](../../../mechanisms/apps/web/src/shell/useTreeNavigation.ts.md#output-d184bfff53d1)
- [apps/web/src/views/KeyboardSection.tsx](contract.md#output-63f1f3c4618f)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)

### Tests and fixtures

- [apps/web/src/debug/DebugStrip.test.tsx](../../../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-1063c352c210)
- [apps/web/src/lib/keymap.test.ts](contract.md#output-4f41c9184a60)
- [apps/web/src/shell/dired.test.ts](../../authoring/the-tree-is-a-dired/contract.md#output-0cb807eba045)
- [apps/web/src/views/KeyboardSection.controls.test.tsx](contract.md#output-17918b3a4e86)
- [crates/hickory-cli/tests/serve_ui_settings.rs](../../../mechanisms/crates/hickory-cli/tests/serve_ui_settings.rs.md#output-c816a698604d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
