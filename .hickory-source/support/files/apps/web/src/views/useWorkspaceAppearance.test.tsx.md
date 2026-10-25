<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/useWorkspaceAppearance.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Settings preserves the open workspace](../../../../../../guarantees/authoring/settings-preserves-the-open-workspace/contract.md)

Captured SHA-256: `eff4a07dac5f3848dff7661783da9eed5a2662e2fba84a5a444ad5a3db7199cb`.

<book:entry path="apps/web/src/views/useWorkspaceAppearance.test.tsx">
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { useWorkspaceAppearance, useWorkspaceSettings } from "./useWorkspaceAppearance";
import { saveTabStyle } from "../lib/tabStyle";
import { saveChannelWidth } from "../lib/channelWidth";
import { SETTINGS_CLOSED } from "../lib/workspaceVisibility";
import { api } from "../api/client";

afterEach(() => { cleanup(); localStorage.clear(); vi.restoreAllMocks(); });
it("updates retained workspace presentation without remounting editors", () => {
  const { result } = renderHook(useWorkspaceAppearance);
  act(() => { saveTabStyle("side"); saveChannelWidth(72); });
  expect(result.current).toEqual({ tabStyle: "side", channelWidth: 72 });
});
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
</book:entry>

</book:doc>
