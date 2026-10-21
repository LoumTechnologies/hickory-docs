import { createRoot } from "react-dom/client";
import { SnapshotPage } from "./SnapshotPage";
import { StaticUiSurface } from "./StaticUiSurface";
import "../styles.css";
import "./snapshot.css";
createRoot(document.getElementById("root")!).render(<StaticUiSurface><SnapshotPage /></StaticUiSurface>);
