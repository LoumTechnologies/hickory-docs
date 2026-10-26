<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/useWorkspaceAppearance.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-14288d7adf50"></a>
## `apps/web/src/views/useWorkspaceAppearance.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Settings preserves the open workspace](../../../../../guarantees/authoring/settings-preserves-the-open-workspace/contract.md).

<a id="f-14288d7adf50-useWorkspaceAppearance"></a>
### `useWorkspaceAppearance`

<storycode:copy id="f-14288d7adf50-useWorkspaceAppearance">
import { useEffect, useState } from "react";
import { loadTabStyle, TAB_STYLE_CHANGED } from "../lib/tabStyle";
import { loadChannelWidth, CHANNEL_WIDTH_CHANGED } from "../lib/channelWidth";
import { api } from "../api/client";
import { SETTINGS_CLOSED } from "../lib/workspaceVisibility";

export function useWorkspaceAppearance() {
  const [tabStyle, setTabStyle] = useState(loadTabStyle);
  const [channelWidth, setChannelWidth] = useState(loadChannelWidth);
  useEffect(() => {
    const tabs = () => setTabStyle(loadTabStyle());
    const width = () => setChannelWidth(loadChannelWidth());
    window.addEventListener(TAB_STYLE_CHANGED, tabs);
    window.addEventListener(CHANNEL_WIDTH_CHANGED, width);
    return () => {
      window.removeEventListener(TAB_STYLE_CHANGED, tabs);
      window.removeEventListener(CHANNEL_WIDTH_CHANGED, width);
    };
  }, []);
  return { tabStyle, channelWidth };
}

</storycode:copy>
<a id="f-14288d7adf50-useWorkspaceSettings"></a>
### `useWorkspaceSettings`

<storycode:copy id="f-14288d7adf50-useWorkspaceSettings">
export function useWorkspaceSettings(setTitle: (title: string | null) => void, setRetain: (retain: boolean) => void) {
  useEffect(() => {
    let live = true;
    const refresh = () => {
      void api.settingsUi().then(ui => {
        if (live) { setTitle(ui.window_title); setRetain(ui.retain_unsaved_saved_files === true); }
      }, () => {});
    };
    refresh();
    window.addEventListener(SETTINGS_CLOSED, refresh);
    return () => { live = false; window.removeEventListener(SETTINGS_CLOSED, refresh); };
  }, [setTitle, setRetain]);
}
</storycode:copy>

### Assemble `apps/web/src/views/useWorkspaceAppearance.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useWorkspaceAppearance.ts"><storycode:paste select="#f-14288d7adf50-useWorkspaceAppearance"/><storycode:paste select="#f-14288d7adf50-useWorkspaceSettings"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-3606526b11a4"></a>
## `apps/web/src/views/useWorkspaceAppearance.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Settings preserves the open workspace](../../../../../guarantees/authoring/settings-preserves-the-open-workspace/contract.md).

<a id="f-3606526b11a4-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-3606526b11a4-expression_statement">
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { useWorkspaceAppearance, useWorkspaceSettings } from "./useWorkspaceAppearance";
import { saveTabStyle } from "../lib/tabStyle";
import { saveChannelWidth } from "../lib/channelWidth";
import { SETTINGS_CLOSED } from "../lib/workspaceVisibility";
import { api } from "../api/client";

afterEach(() => { cleanup(); localStorage.clear(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-3606526b11a4-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-3606526b11a4-expression_statement-2">
it("updates retained workspace presentation without remounting editors", () => {
  const { result } = renderHook(useWorkspaceAppearance);
  act(() => { saveTabStyle("side"); saveChannelWidth(72); });
  expect(result.current).toEqual({ tabStyle: "side", channelWidth: 72 });
});
</storycode:copy>
<a id="f-3606526b11a4-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-3606526b11a4-expression_statement-3">
it("refreshes server preferences when Settings closes", async () => {
  const title = vi.fn(), retain = vi.fn();
  const fetch = vi.spyOn(api, "settingsUi").mockResolvedValue({ window_title: "Before", retain_unsaved_saved_files: false } as never);
  renderHook(() => useWorkspaceSettings(title, retain));
  await waitFor(() => expect(title).toHaveBeenCalledWith("Before"));
  fetch.mockResolvedValue({ window_title: "After", retain_unsaved_saved_files: true } as never);
  act(() => window.dispatchEvent(new Event(SETTINGS_CLOSED)));
  await waitFor(() => expect(title).toHaveBeenCalledWith("After"));
  expect(retain).toHaveBeenLastCalledWith(true);
});
</storycode:copy>

### Assemble `apps/web/src/views/useWorkspaceAppearance.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useWorkspaceAppearance.test.tsx"><storycode:paste select="#f-3606526b11a4-expression_statement"/><storycode:paste select="#f-3606526b11a4-expression_statement-2"/><storycode:paste select="#f-3606526b11a4-expression_statement-3"/></storycode:file>


</storycode:doc>
