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

</storycode:doc>
