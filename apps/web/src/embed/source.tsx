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
