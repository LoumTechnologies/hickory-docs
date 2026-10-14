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
