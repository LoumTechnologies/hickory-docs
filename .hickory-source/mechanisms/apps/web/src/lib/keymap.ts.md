<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/keymap.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4d0aa846a85d"></a>
## `apps/web/src/lib/keymap.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](../../../../../guarantees/authoring/the-editor-keeps-its-space/contract.md), [Every Shortcut Is A Setting](../../../../../guarantees/editor-intelligence/every-shortcut-is-a-setting/contract.md).

<a id="f-4d0aa846a85d-Profile"></a>
### `Profile`

Every shortcut is a setting.
One catalogue of the actions a key can drive — the menu bar's, the
editor's, the tree's — and, for each, what it is bound to under each
profile: hickory's own keys (which are VS Code's, since that is what most
hands arrive with), JetBrains, and Visual Studio. A person picks a profile
and overrides any single action; both persist in ui.json beside the window
title, so the desktop app and `hick up` agree.
A binding is written the way people write them: `Ctrl+Shift+L`,
`Shift+Alt+F`, `F8`, and a chord as two strokes with a space,
`Ctrl+K Ctrl+D`. `Mod` is the platform's primary modifier — Ctrl here,
Cmd on a Mac — and is what the tables use, so one table serves every
platform. Three spellings come out of one binding: the display string,
CodeMirror's (`Mod-Shift-l`), and the shell's (`CmdOrCtrl+Shift+L`).
The menu bar is the shell's, built at launch from ui.json's
`native_accelerators`; the page cannot rebind it live, so a change to a
menu action lands at the next launch, and Settings says so. Chords cannot
drive a native menu item at all, so a menu action bound to a chord has no
accelerator and is reached from the menu itself.
See docs/guarantees/editor-intelligence/every-shortcut-is-a-setting.md.

<storycode:copy id="f-4d0aa846a85d-Profile">
// Every shortcut is a setting.
//
// One catalogue of the actions a key can drive — the menu bar's, the
// editor's, the tree's — and, for each, what it is bound to under each
// profile: hickory's own keys (which are VS Code's, since that is what most
// hands arrive with), JetBrains, and Visual Studio. A person picks a profile
// and overrides any single action; both persist in ui.json beside the window
// title, so the desktop app and `hick up` agree.
//
// A binding is written the way people write them: `Ctrl+Shift+L`,
// `Shift+Alt+F`, `F8`, and a chord as two strokes with a space,
// `Ctrl+K Ctrl+D`. `Mod` is the platform's primary modifier — Ctrl here,
// Cmd on a Mac — and is what the tables use, so one table serves every
// platform. Three spellings come out of one binding: the display string,
// CodeMirror's (`Mod-Shift-l`), and the shell's (`CmdOrCtrl+Shift+L`).
//
// The menu bar is the shell's, built at launch from ui.json's
// `native_accelerators`; the page cannot rebind it live, so a change to a
// menu action lands at the next launch, and Settings says so. Chords cannot
// drive a native menu item at all, so a menu action bound to a chord has no
// accelerator and is reached from the menu itself.
//
// See docs/guarantees/editor-intelligence/every-shortcut-is-a-setting.md.

import { api } from "../api/client";

export type Profile = "hickory" | "vscode" | "jetbrains" | "visualstudio";

</storycode:copy>
<a id="f-4d0aa846a85d-PROFILES"></a>
### `PROFILES`

<storycode:copy id="f-4d0aa846a85d-PROFILES">
export const PROFILES: { id: Profile; label: string }[] = [
  { id: "hickory", label: "Hickory (VS Code keys)" },
  { id: "vscode", label: "VS Code" },
  { id: "jetbrains", label: "JetBrains" },
  { id: "visualstudio", label: "Visual Studio" },
];

</storycode:copy>
<a id="f-4d0aa846a85d-Scope"></a>
### `Scope`

Where an action is answered, which is also how the Settings page groups
them. `menu` actions are the native menu bar's and take effect at the
next launch.

<storycode:copy id="f-4d0aa846a85d-Scope">
/** Where an action is answered, which is also how the Settings page groups
 * them. `menu` actions are the native menu bar's and take effect at the
 * next launch. */
export type Scope = "menu" | "editor" | "workspace" | "tree";

</storycode:copy>
<a id="f-4d0aa846a85d-ActionSpec"></a>
### `ActionSpec`

<storycode:copy id="f-4d0aa846a85d-ActionSpec">
export interface ActionSpec {
  id: string;
  label: string;
  scope: Scope;
  /** The native menu item id, for `menu` actions. */
  menuId?: string;
  /** Bindings per profile; a profile absent here falls back to `hickory`,
   * and `""` means deliberately unbound. */
  keys: Partial<Record<Profile, string>> & { hickory: string };
}

</storycode:copy>
<a id="f-4d0aa846a85d-ACTIONS"></a>
### `ACTIONS`

<storycode:copy id="f-4d0aa846a85d-ACTIONS">
export const ACTIONS: readonly ActionSpec[] = [
  // The menu bar.
  { id: "file.newWindow", label: "New window", scope: "menu", menuId: "new-window", keys: { hickory: "Mod+Shift+N" } },
  { id: "file.new", label: "New document", scope: "menu", menuId: "new", keys: { hickory: "Mod+N" } },
  { id: "file.newProject", label: "New project", scope: "menu", menuId: "new-project", keys: { hickory: "Mod+Alt+Shift+N" } },
  { id: "file.openFile", label: "Open file", scope: "menu", menuId: "open-file", keys: { hickory: "Mod+O" } },
  { id: "file.openFolder", label: "Open folder", scope: "menu", menuId: "open-folder", keys: { hickory: "Mod+Shift+O", jetbrains: "Mod+Alt+O" } },
  { id: "file.save", label: "Save", scope: "menu", menuId: "save", keys: { hickory: "Mod+S" } },
  { id: "file.saveAs", label: "Save as", scope: "menu", menuId: "save-as", keys: { hickory: "Mod+Shift+S", jetbrains: "", visualstudio: "" } },
  { id: "file.saveAll", label: "Save all", scope: "menu", menuId: "save-all", keys: { hickory: "Mod+Alt+S", jetbrains: "", visualstudio: "Mod+Shift+S" } },
  { id: "file.print", label: "Print", scope: "menu", menuId: "print", keys: { hickory: "Mod+P" } },
  { id: "view.terminal", label: "New terminal", scope: "menu", menuId: "terminal", keys: { hickory: "Mod+Shift+T", jetbrains: "Alt+F12", visualstudio: "Mod+`" } },
  { id: "view.attention", label: "Next terminal needing attention", scope: "menu", menuId: "attention", keys: { hickory: "Mod+J" } },
  { id: "view.files", label: "Show files", scope: "menu", menuId: "files", keys: { hickory: "Mod+Shift+E", jetbrains: "Alt+1", visualstudio: "Mod+Alt+L" } },
  { id: "view.agent", label: "Agent pane", scope: "menu", menuId: "show-agent", keys: { hickory: "Mod+Shift+I" } },
  { id: "view.settings", label: "Settings", scope: "menu", menuId: "settings", keys: { hickory: "Mod+,", jetbrains: "Mod+Alt+S" } },
  { id: "view.blame", label: "Show blame column", scope: "menu", menuId: "blame", keys: { hickory: "Mod+Alt+B" } },
  { id: "edit.insert", label: "Insert element", scope: "menu", menuId: "insert", keys: { hickory: "Mod+I" } },
  { id: "zoom.in", label: "Zoom in", scope: "menu", menuId: "zoom-in", keys: { hickory: "Mod+=" } },
  { id: "zoom.out", label: "Zoom out", scope: "menu", menuId: "zoom-out", keys: { hickory: "Mod+-" } },
  { id: "zoom.reset", label: "Actual size", scope: "menu", menuId: "zoom-reset", keys: { hickory: "Mod+0" } },
  { id: "zoom.tabIn", label: "Zoom in this tab", scope: "menu", menuId: "zoom-tab-in", keys: { hickory: "Mod+Alt+=" } },
  { id: "zoom.tabOut", label: "Zoom out this tab", scope: "menu", menuId: "zoom-tab-out", keys: { hickory: "Mod+Alt+-" } },
  { id: "zoom.tabReset", label: "Actual size in this tab", scope: "menu", menuId: "zoom-tab-reset", keys: { hickory: "Mod+Alt+0" } },
  // The workspace.
  { id: "search.quickOpen", label: "Search open files and folder", scope: "workspace", keys: { hickory: "Mod+P" } },
  { id: "search.commands", label: "Command palette", scope: "workspace", keys: { hickory: "Mod+Shift+P" } },
  { id: "search.inFolder", label: "Find in folder", scope: "workspace", keys: { hickory: "Mod+Shift+F" } },
  { id: "view.problems", label: "Problems list", scope: "workspace", keys: { hickory: "F8", jetbrains: "Alt+6", visualstudio: "Mod+\\ E" } },
  { id: "debug.continue", label: "Debug: start or continue", scope: "workspace", keys: { hickory: "F5", jetbrains: "F9" } },
  { id: "debug.start", label: "Debug: start", scope: "workspace", keys: { hickory: "", jetbrains: "Shift+F9" } },
  { id: "debug.stop", label: "Debug: stop", scope: "workspace", keys: { hickory: "Shift+F5", jetbrains: "Ctrl+F2" } },
  { id: "debug.over", label: "Debug: step over", scope: "workspace", keys: { hickory: "F10", jetbrains: "F8" } },
  { id: "debug.in", label: "Debug: step into", scope: "workspace", keys: { hickory: "F11", jetbrains: "F7" } },
  { id: "debug.out", label: "Debug: step out", scope: "workspace", keys: { hickory: "Shift+F11", jetbrains: "Shift+F8" } },
  // The editor.
  { id: "editor.definition", label: "Go to definition", scope: "editor", keys: { hickory: "F12", jetbrains: "Mod+B" } },
  { id: "editor.references", label: "Find references", scope: "editor", keys: { hickory: "Shift+F12", jetbrains: "Alt+F7" } },
  { id: "editor.find", label: "Find", scope: "editor", keys: { hickory: "Mod+F" } },
  { id: "editor.format", label: "Format document", scope: "editor", keys: { hickory: "Shift+Alt+F", jetbrains: "Mod+Alt+L", visualstudio: "Mod+K Mod+D" } },
  { id: "editor.rename", label: "Rename symbol", scope: "editor", keys: { hickory: "F2", jetbrains: "Shift+F6", visualstudio: "Mod+R Mod+R" } },
  { id: "editor.codeAction", label: "Code action", scope: "editor", keys: { hickory: "Mod+.", jetbrains: "Alt+Enter" } },
  { id: "editor.addNextOccurrence", label: "Add next occurrence to selection", scope: "editor", keys: { hickory: "Mod+D", jetbrains: "Alt+J", visualstudio: "Shift+Alt+." } },
  { id: "editor.selectAllOccurrences", label: "Select all occurrences", scope: "editor", keys: { hickory: "Mod+Shift+L", jetbrains: "Mod+Alt+Shift+J", visualstudio: "Shift+Alt+;" } },
  // The tree, as dired. The same under every profile: these are dired's
  // keys, not an IDE's, and they fire only with a tree row focused.
  { id: "tree.mark", label: "Tree: mark", scope: "tree", keys: { hickory: "m" } },
  { id: "tree.unmark", label: "Tree: unmark", scope: "tree", keys: { hickory: "u" } },
  { id: "tree.unmarkAll", label: "Tree: unmark all", scope: "tree", keys: { hickory: "Shift+U" } },
  { id: "tree.delete", label: "Tree: delete", scope: "tree", keys: { hickory: "Shift+D" } },
  { id: "tree.rename", label: "Tree: rename", scope: "tree", keys: { hickory: "F2" } },
  { id: "tree.copy", label: "Tree: copy to…", scope: "tree", keys: { hickory: "Shift+C" } },
  { id: "tree.move", label: "Tree: move to…", scope: "tree", keys: { hickory: "Shift+M" } },
  { id: "tree.newFolder", label: "Tree: new folder", scope: "tree", keys: { hickory: "+" } },
  { id: "tree.newFile", label: "Tree: new file", scope: "tree", keys: { hickory: "n" } },
  { id: "tree.addSelectionAbove", label: "Tree: add row above to column selection", scope: "tree", keys: { hickory: "Shift+Alt+ArrowUp" } },
  { id: "tree.addSelectionBelow", label: "Tree: add row below to column selection", scope: "tree", keys: { hickory: "Shift+Alt+ArrowDown" } },
];

</storycode:copy>
<a id="f-4d0aa846a85d-Stroke"></a>
### `Stroke`

<storycode:copy id="f-4d0aa846a85d-Stroke">
// ---------------------------------------------------------------------------
// One binding, three spellings
// ---------------------------------------------------------------------------

export interface Stroke {
  mod: boolean;
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  /** The key, as `KeyboardEvent.key` spells an unshifted one: `s`, `F8`,
   * `,`, `Enter`, `+`. Letters are lower case. */
  key: string;
}

</storycode:copy>
<a id="f-4d0aa846a85d-MODIFIERS"></a>
### `MODIFIERS`

<storycode:copy id="f-4d0aa846a85d-MODIFIERS">
const MODIFIERS = new Set(["mod", "ctrl", "control", "alt", "option", "shift", "cmd", "meta"]);

</storycode:copy>
<a id="f-4d0aa846a85d-parseStroke"></a>
### `parseStroke`

Parse one stroke, `Ctrl+Shift+L`. Null when it names nothing usable.

<storycode:copy id="f-4d0aa846a85d-parseStroke">
/** Parse one stroke, `Ctrl+Shift+L`. Null when it names nothing usable. */
export function parseStroke(text: string): Stroke | null {
  const parts = text.trim().split("+");
  // `+` itself: `Ctrl++` splits into ["Ctrl", "", ""]; a bare `+` into ["", ""].
  const raw = text.trim();
  if (raw === "+") return { mod: false, ctrl: false, alt: false, shift: false, key: "+" };
  const stroke: Stroke = { mod: false, ctrl: false, alt: false, shift: false, key: "" };
  const names = raw.endsWith("++") ? [...parts.slice(0, -2), "+"] : parts;
  for (const part of names) {
    const low = part.toLowerCase();
    if (!low) return null;
    if (MODIFIERS.has(low)) {
      if (low === "mod" || low === "cmd" || low === "meta") stroke.mod = true;
      else if (low === "ctrl" || low === "control") stroke.ctrl = true;
      else if (low === "alt" || low === "option") stroke.alt = true;
      else stroke.shift = true;
      continue;
    }
    if (stroke.key) return null;
    stroke.key = part.length === 1 ? part.toLowerCase() : normalizeKeyName(part);
  }
  return stroke.key ? stroke : null;
}

</storycode:copy>
<a id="f-4d0aa846a85d-normalizeKeyName"></a>
### `normalizeKeyName`

<storycode:copy id="f-4d0aa846a85d-normalizeKeyName">
function normalizeKeyName(name: string): string {
  const low = name.toLowerCase();
  const known: Record<string, string> = {
    esc: "Escape",
    escape: "Escape",
    enter: "Enter",
    return: "Enter",
    tab: "Tab",
    space: " ",
    backspace: "Backspace",
    delete: "Delete",
    del: "Delete",
    up: "ArrowUp",
    down: "ArrowDown",
    left: "ArrowLeft",
    right: "ArrowRight",
    home: "Home",
    end: "End",
    pageup: "PageUp",
    pagedown: "PageDown",
  };
  if (known[low]) return known[low];
  if (/^f\d{1,2}$/.test(low)) return low.toUpperCase();
  return name;
}

</storycode:copy>
<a id="f-4d0aa846a85d-parseBinding"></a>
### `parseBinding`

Parse a binding: one stroke, or a chord of two separated by a space.

<storycode:copy id="f-4d0aa846a85d-parseBinding">
/** Parse a binding: one stroke, or a chord of two separated by a space. */
export function parseBinding(text: string): Stroke[] | null {
  const trimmed = text.trim();
  if (!trimmed) return null;
  // A binding whose key is a space is written `Space`, never a literal.
  const strokes = trimmed.split(/\s+/).map(parseStroke);
  if (strokes.length > 2 || strokes.some((s) => s === null)) return null;
  return strokes as Stroke[];
}

</storycode:copy>
<a id="f-4d0aa846a85d-isMac"></a>
### `isMac`

Whether this is a Mac, for how `Mod` is spelled and matched.

<storycode:copy id="f-4d0aa846a85d-isMac">
/** Whether this is a Mac, for how `Mod` is spelled and matched. */
export function isMac(): boolean {
  return typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform ?? "");
}

</storycode:copy>
<a id="f-4d0aa846a85d-formatBinding"></a>
### `formatBinding`

The display spelling: `Ctrl+Shift+L`, or `⌘⇧L` on a Mac.

<storycode:copy id="f-4d0aa846a85d-formatBinding">
/** The display spelling: `Ctrl+Shift+L`, or `⌘⇧L` on a Mac. */
export function formatBinding(text: string, mac = isMac()): string {
  const strokes = parseBinding(text);
  if (!strokes) return text;
  return strokes.map((s) => formatStroke(s, mac)).join(" ");
}

</storycode:copy>
<a id="f-4d0aa846a85d-displayKey"></a>
### `displayKey`

<storycode:copy id="f-4d0aa846a85d-displayKey">
function displayKey(key: string): string {
  if (key === " ") return "Space";
  if (key.length === 1) return key.toUpperCase();
  return key.replace(/^Arrow/, "");
}

</storycode:copy>
<a id="f-4d0aa846a85d-formatStroke"></a>
### `formatStroke`

<storycode:copy id="f-4d0aa846a85d-formatStroke">
function formatStroke(s: Stroke, mac: boolean): string {
  if (mac) {
    return `${s.ctrl ? "⌃" : ""}${s.alt ? "⌥" : ""}${s.shift ? "⇧" : ""}${s.mod ? "⌘" : ""}${displayKey(s.key)}`;
  }
  const parts: string[] = [];
  if (s.mod || s.ctrl) parts.push("Ctrl");
  if (s.alt) parts.push("Alt");
  if (s.shift) parts.push("Shift");
  parts.push(displayKey(s.key));
  return parts.join("+");
}

</storycode:copy>
<a id="f-4d0aa846a85d-toCodeMirror"></a>
### `toCodeMirror`

CodeMirror's spelling: `Mod-Shift-l`, chords space-separated.

<storycode:copy id="f-4d0aa846a85d-toCodeMirror">
/** CodeMirror's spelling: `Mod-Shift-l`, chords space-separated. */
export function toCodeMirror(text: string): string | null {
  const strokes = parseBinding(text);
  if (!strokes) return null;
  return strokes
    .map((s) => {
      const parts: string[] = [];
      if (s.ctrl) parts.push("Ctrl");
      if (s.mod) parts.push("Mod");
      if (s.alt) parts.push("Alt");
      if (s.shift) parts.push("Shift");
      parts.push(s.key === " " ? "Space" : s.key);
      return parts.join("-");
    })
    .join(" ");
}

</storycode:copy>
<a id="f-4d0aa846a85d-toNativeAccelerator"></a>
### `toNativeAccelerator`

The shell's spelling for a native accelerator: `CmdOrCtrl+Shift+S`. Null
for a chord, which no menu bar can carry, and for an unbound action.

<storycode:copy id="f-4d0aa846a85d-toNativeAccelerator">
/** The shell's spelling for a native accelerator: `CmdOrCtrl+Shift+S`. Null
 * for a chord, which no menu bar can carry, and for an unbound action. */
export function toNativeAccelerator(text: string): string | null {
  const strokes = parseBinding(text);
  if (!strokes || strokes.length !== 1) return null;
  const s = strokes[0];
  const parts: string[] = [];
  if (s.mod) parts.push("CmdOrCtrl");
  if (s.ctrl) parts.push("Ctrl");
  if (s.alt) parts.push("Alt");
  if (s.shift) parts.push("Shift");
  parts.push(s.key === " " ? "Space" : s.key.length === 1 ? s.key.toUpperCase() : s.key);
  return parts.join("+");
}

</storycode:copy>
<a id="f-4d0aa846a85d-strokeMatches"></a>
### `strokeMatches`

Whether a keyboard event is this stroke. `mod` matches Cmd on a Mac and
Ctrl elsewhere, and a plain Ctrl in a binding is Ctrl on both.

<storycode:copy id="f-4d0aa846a85d-strokeMatches">
/** Whether a keyboard event is this stroke. `mod` matches Cmd on a Mac and
 * Ctrl elsewhere, and a plain Ctrl in a binding is Ctrl on both. */
export function strokeMatches(
  event: { key: string; ctrlKey: boolean; metaKey: boolean; altKey: boolean; shiftKey: boolean },
  s: Stroke,
  mac = isMac(),
): boolean {
  const modDown = mac ? event.metaKey : event.ctrlKey;
  const ctrlDown = mac ? event.ctrlKey : event.ctrlKey && !s.mod;
  if (s.mod !== modDown) return false;
  if (s.ctrl !== (mac ? ctrlDown : event.ctrlKey && !s.mod)) return false;
  if (s.alt !== event.altKey) return false;
  if (s.shift !== event.shiftKey) return false;
  if (mac && !s.mod && event.metaKey) return false;
  const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
  return key === s.key;
}

</storycode:copy>
<a id="f-4d0aa846a85d-KeymapSettings"></a>
### `KeymapSettings`

The resolved keymap: profile + overrides → binding per action

<storycode:copy id="f-4d0aa846a85d-KeymapSettings">
// ---------------------------------------------------------------------------
// The resolved keymap: profile + overrides → binding per action
// ---------------------------------------------------------------------------

export interface KeymapSettings {
  profile: Profile;
  /** Per action: a binding, or `""` to unbind. Absent means the profile's. */
  overrides: Record<string, string>;
}

</storycode:copy>
<a id="f-4d0aa846a85d-DEFAULT_SETTINGS"></a>
### `DEFAULT_SETTINGS`

<storycode:copy id="f-4d0aa846a85d-DEFAULT_SETTINGS">
export const DEFAULT_SETTINGS: KeymapSettings = { profile: "hickory", overrides: {} };

</storycode:copy>
<a id="f-4d0aa846a85d-isProfile"></a>
### `isProfile`

<storycode:copy id="f-4d0aa846a85d-isProfile">
export function isProfile(value: unknown): value is Profile {
  return PROFILES.some((p) => p.id === value);
}

</storycode:copy>
<a id="f-4d0aa846a85d-profileBinding"></a>
### `profileBinding`

The profile's own binding for an action (`""` when unbound).

<storycode:copy id="f-4d0aa846a85d-profileBinding">
/** The profile's own binding for an action (`""` when unbound). */
export function profileBinding(spec: ActionSpec, profile: Profile): string {
  return spec.keys[profile] ?? spec.keys.hickory;
}

</storycode:copy>
<a id="f-4d0aa846a85d-resolve"></a>
### `resolve`

<storycode:copy id="f-4d0aa846a85d-resolve">
/** The binding in force for every action. */
export function resolve(settings: KeymapSettings): Map<string, string> {
  const out = new Map<string, string>();
  for (const spec of ACTIONS) {
    const own = settings.overrides[spec.id];
    out.set(spec.id, own !== undefined ? own : profileBinding(spec, settings.profile));
  }
  return out;
}

</storycode:copy>
<a id="f-4d0aa846a85d-conflicts"></a>
### `conflicts`

Two actions in the same scope bound to the same keys.

<storycode:copy id="f-4d0aa846a85d-conflicts">
/** Two actions in the same scope bound to the same keys. */
export function conflicts(bindings: Map<string, string>): [string, string][] {
  const seen = new Map<string, string>();
  const out: [string, string][] = [];
  for (const spec of ACTIONS) {
    const binding = bindings.get(spec.id);
    if (!binding) continue;
    const parsed = parseBinding(binding);
    if (!parsed) continue;
    const key = `${spec.scope}:${parsed.map((s) => toCodeMirror(formatStroke(s, false)) ?? "").join(" ")}`;
    const other = seen.get(key);
    if (other) out.push([other, spec.id]);
    else seen.set(key, spec.id);
  }
  return out;
}

</storycode:copy>
<a id="f-4d0aa846a85d-nativeAccelerators"></a>
### `nativeAccelerators`

What the shell's menu bar should carry, by menu id: a resolved
accelerator, or `null` for an action that is unbound or bound to a chord.
Every menu action is listed so the shell can tell "unbound" from
"not configured".

<storycode:copy id="f-4d0aa846a85d-nativeAccelerators">
/** What the shell's menu bar should carry, by menu id: a resolved
 * accelerator, or `null` for an action that is unbound or bound to a chord.
 * Every menu action is listed so the shell can tell "unbound" from
 * "not configured". */
export function nativeAccelerators(bindings: Map<string, string>): Record<string, string | null> {
  const out: Record<string, string | null> = {};
  for (const spec of ACTIONS) {
    if (!spec.menuId) continue;
    out[spec.menuId] = toNativeAccelerator(bindings.get(spec.id) ?? "");
  }
  return out;
}

</storycode:copy>
<a id="f-4d0aa846a85d-settings"></a>
### `settings`

<storycode:copy id="f-4d0aa846a85d-settings">
// ---------------------------------------------------------------------------
// The live keymap, read by every consumer
// ---------------------------------------------------------------------------

let settings: KeymapSettings = DEFAULT_SETTINGS;
</storycode:copy>
<a id="f-4d0aa846a85d-bindings"></a>
### `bindings`

<storycode:copy id="f-4d0aa846a85d-bindings">
let bindings = resolve(settings);
</storycode:copy>
<a id="f-4d0aa846a85d-listeners"></a>
### `listeners`

<storycode:copy id="f-4d0aa846a85d-listeners">
const listeners = new Set<() => void>();

</storycode:copy>
<a id="f-4d0aa846a85d-keymapSettings"></a>
### `keymapSettings`

<storycode:copy id="f-4d0aa846a85d-keymapSettings">
export function keymapSettings(): KeymapSettings {
  return settings;
}

</storycode:copy>
<a id="f-4d0aa846a85d-setKeymapSettings"></a>
### `setKeymapSettings`

<storycode:copy id="f-4d0aa846a85d-setKeymapSettings">
export function setKeymapSettings(next: KeymapSettings): void {
  settings = next;
  bindings = resolve(next);
  for (const listener of listeners) listener();
}

</storycode:copy>
<a id="f-4d0aa846a85d-onKeymapChange"></a>
### `onKeymapChange`

<storycode:copy id="f-4d0aa846a85d-onKeymapChange">
export function onKeymapChange(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

</storycode:copy>
<a id="f-4d0aa846a85d-bindingOf"></a>
### `bindingOf`

The binding in force for an action, `""` when unbound.

<storycode:copy id="f-4d0aa846a85d-bindingOf">
/** The binding in force for an action, `""` when unbound. */
export function bindingOf(actionId: string): string {
  return bindings.get(actionId) ?? "";
}

</storycode:copy>
<a id="f-4d0aa846a85d-cmKeyOf"></a>
### `cmKeyOf`

CodeMirror's spelling of an action's binding, or null when unbound.

<storycode:copy id="f-4d0aa846a85d-cmKeyOf">
/** CodeMirror's spelling of an action's binding, or null when unbound. */
export function cmKeyOf(actionId: string): string | null {
  const binding = bindingOf(actionId);
  return binding ? toCodeMirror(binding) : null;
}

</storycode:copy>
<a id="f-4d0aa846a85d-KeyLike"></a>
### `KeyLike`

<storycode:copy id="f-4d0aa846a85d-KeyLike">
type KeyLike = { key: string; ctrlKey: boolean; metaKey: boolean; altKey: boolean; shiftKey: boolean };

</storycode:copy>
<a id="f-4d0aa846a85d-pending"></a>
### `pending`

The first stroke of a chord, seen within the last moment; a chord's
second stroke completes it. One pending stroke for the whole page: two
chords cannot be half-typed at once.

<storycode:copy id="f-4d0aa846a85d-pending">
/** The first stroke of a chord, seen within the last moment; a chord's
 * second stroke completes it. One pending stroke for the whole page: two
 * chords cannot be half-typed at once. */
let pending: { stroke: Stroke; at: number } | null = null;
</storycode:copy>
<a id="f-4d0aa846a85d-CHORD_MS"></a>
### `CHORD_MS`

<storycode:copy id="f-4d0aa846a85d-CHORD_MS">
const CHORD_MS = 1500;

</storycode:copy>
<a id="f-4d0aa846a85d-beginsChord"></a>
### `beginsChord`

Whether this event begins a chord bound to any action. A handler calls
it first and swallows the event, so the first stroke of `Ctrl+K Ctrl+D`
does not also reach the editor as Ctrl+K.

<storycode:copy id="f-4d0aa846a85d-beginsChord">
/**
 * Whether this event begins a chord bound to any action. A handler calls
 * it first and swallows the event, so the first stroke of `Ctrl+K Ctrl+D`
 * does not also reach the editor as Ctrl+K.
 */
export function beginsChord(event: KeyLike, now = Date.now()): boolean {
  for (const spec of ACTIONS) {
    const parsed = parseBinding(bindings.get(spec.id) ?? "");
    if (parsed && parsed.length === 2 && strokeMatches(event, parsed[0])) {
      pending = { stroke: parsed[0], at: now };
      return true;
    }
  }
  return false;
}

</storycode:copy>
<a id="f-4d0aa846a85d-isAction"></a>
### `isAction`

Whether a keyboard event is an action: its one stroke, or the second
stroke of its chord after the first was seen.

<storycode:copy id="f-4d0aa846a85d-isAction">
/** Whether a keyboard event is an action: its one stroke, or the second
 * stroke of its chord after the first was seen. */
export function isAction(event: KeyLike, actionId: string, now = Date.now()): boolean {
  const parsed = parseBinding(bindingOf(actionId));
  if (!parsed) return false;
  if (parsed.length === 1) return strokeMatches(event, parsed[0]);
  const first = pending;
  if (!first || now - first.at > CHORD_MS) return false;
  const same = (a: Stroke, b: Stroke) =>
    a.key === b.key && a.mod === b.mod && a.ctrl === b.ctrl && a.alt === b.alt && a.shift === b.shift;
  if (!same(first.stroke, parsed[0]) || !strokeMatches(event, parsed[1])) return false;
  pending = null;
  return true;
}

</storycode:copy>
<a id="f-4d0aa846a85d-loadKeymap"></a>
### `loadKeymap`

Read the settings the server keeps. A failure leaves the defaults.

<storycode:copy id="f-4d0aa846a85d-loadKeymap">
/** Read the settings the server keeps. A failure leaves the defaults. */
export async function loadKeymap(): Promise<void> {
  try {
    const ui = await api.settingsUi();
    const raw = ui.keymap;
    if (raw && typeof raw === "object") {
      const profile = isProfile(raw.profile) ? raw.profile : "hickory";
      const overrides: Record<string, string> = {};
      for (const [id, value] of Object.entries(raw.overrides ?? {})) {
        if (typeof value === "string") overrides[id] = value;
      }
      setKeymapSettings({ profile, overrides });
    }
  } catch {
    // The defaults: the honest answer when nothing can be read.
  }
}

</storycode:copy>
<a id="f-4d0aa846a85d-saveKeymap"></a>
### `saveKeymap`

Persist the settings, and the menu bar's accelerators derived from them.

<storycode:copy id="f-4d0aa846a85d-saveKeymap">
/** Persist the settings, and the menu bar's accelerators derived from them. */
export async function saveKeymap(next: KeymapSettings): Promise<void> {
  setKeymapSettings(next);
  await api.saveSettingsUi({
    keymap: next,
    native_accelerators: nativeAccelerators(resolve(next)),
  });
}
</storycode:copy>

### Assemble `apps/web/src/lib/keymap.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/keymap.ts"><storycode:paste select="#f-4d0aa846a85d-Profile"/><storycode:paste select="#f-4d0aa846a85d-PROFILES"/><storycode:paste select="#f-4d0aa846a85d-Scope"/><storycode:paste select="#f-4d0aa846a85d-ActionSpec"/><storycode:paste select="#f-4d0aa846a85d-ACTIONS"/><storycode:paste select="#f-4d0aa846a85d-Stroke"/><storycode:paste select="#f-4d0aa846a85d-MODIFIERS"/><storycode:paste select="#f-4d0aa846a85d-parseStroke"/><storycode:paste select="#f-4d0aa846a85d-normalizeKeyName"/><storycode:paste select="#f-4d0aa846a85d-parseBinding"/><storycode:paste select="#f-4d0aa846a85d-isMac"/><storycode:paste select="#f-4d0aa846a85d-formatBinding"/><storycode:paste select="#f-4d0aa846a85d-displayKey"/><storycode:paste select="#f-4d0aa846a85d-formatStroke"/><storycode:paste select="#f-4d0aa846a85d-toCodeMirror"/><storycode:paste select="#f-4d0aa846a85d-toNativeAccelerator"/><storycode:paste select="#f-4d0aa846a85d-strokeMatches"/><storycode:paste select="#f-4d0aa846a85d-KeymapSettings"/><storycode:paste select="#f-4d0aa846a85d-DEFAULT_SETTINGS"/><storycode:paste select="#f-4d0aa846a85d-isProfile"/><storycode:paste select="#f-4d0aa846a85d-profileBinding"/><storycode:paste select="#f-4d0aa846a85d-resolve"/><storycode:paste select="#f-4d0aa846a85d-conflicts"/><storycode:paste select="#f-4d0aa846a85d-nativeAccelerators"/><storycode:paste select="#f-4d0aa846a85d-settings"/><storycode:paste select="#f-4d0aa846a85d-bindings"/><storycode:paste select="#f-4d0aa846a85d-listeners"/><storycode:paste select="#f-4d0aa846a85d-keymapSettings"/><storycode:paste select="#f-4d0aa846a85d-setKeymapSettings"/><storycode:paste select="#f-4d0aa846a85d-onKeymapChange"/><storycode:paste select="#f-4d0aa846a85d-bindingOf"/><storycode:paste select="#f-4d0aa846a85d-cmKeyOf"/><storycode:paste select="#f-4d0aa846a85d-KeyLike"/><storycode:paste select="#f-4d0aa846a85d-pending"/><storycode:paste select="#f-4d0aa846a85d-CHORD_MS"/><storycode:paste select="#f-4d0aa846a85d-beginsChord"/><storycode:paste select="#f-4d0aa846a85d-isAction"/><storycode:paste select="#f-4d0aa846a85d-loadKeymap"/><storycode:paste select="#f-4d0aa846a85d-saveKeymap"/></storycode:file>

</storycode:doc>
