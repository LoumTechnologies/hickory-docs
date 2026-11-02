<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Mount the static source browser

For engineers maintaining the public source browser.

Use the same scoped static UI host as the existing publication viewers, so document actions remain local to this read-only publication. This entry mounts the public source page without starting the desktop application. Apply the shared palette from the device preference, without reading the app’s stored theme choice: the public reader has no theme picker. Listen for preference changes so an open document switches with the device. The HTML entry sets the same root attribute before the bundle loads to avoid a dark first paint on a light device.

<storycode:copy id="public-source-apps-web-src-embed-source-tsx">
import { applyTheme, loadTheme } from "../lib/theme";
import { createRoot } from "react-dom/client";
import { SourcePage } from "./SourcePage";
import { StaticUiSurface } from "./StaticUiSurface";
import "../styles.css";
import "./source.css";
// This reader has no theme picker: follow the device, independent of app preferences.
const systemAppearance = matchMedia("(prefers-color-scheme: light)");
const syncAppearance = () => applyTheme(loadTheme(null));
syncAppearance();
systemAppearance.addEventListener("change", syncAppearance);
createRoot(document.getElementById("root")!).render(<StaticUiSurface><SourcePage /></StaticUiSurface>);
</storycode:copy>

<storycode:file path="apps/web/src/embed/source.tsx"><storycode:paste select="#public-source-apps-web-src-embed-source-tsx"/></storycode:file>
</storycode:doc>
