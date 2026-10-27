import { createRoot } from "react-dom/client";
import { loadHickLang } from "../editor/hickLang";
import { MobileApp } from "./MobileApp";
import "./mobile.css";

void loadHickLang().then(() => {
  createRoot(document.getElementById("root")!).render(<MobileApp />);
}).catch((error: unknown) => {
  const root = document.getElementById("root")!;
  root.setAttribute("role", "alert");
  root.textContent = `Hickory Docs could not open: ${String(error)}`;
});
