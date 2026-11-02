import { createRoot } from "react-dom/client";
import { SourcePage } from "./SourcePage";
import { StaticUiSurface } from "./StaticUiSurface";
import "../styles.css";
import "./source.css";
createRoot(document.getElementById("root")!).render(<StaticUiSurface><SourcePage /></StaticUiSurface>);
