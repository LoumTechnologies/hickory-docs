import { createRoot } from "react-dom/client";
import { CiPublicationPage } from "./CiPublication";
import { StaticUiSurface } from "./StaticUiSurface";
import "../styles.css";
import "./ci.css";
createRoot(document.getElementById("root")!).render(<StaticUiSurface><CiPublicationPage /></StaticUiSurface>);
