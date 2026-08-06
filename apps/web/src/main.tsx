import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { MOCK } from "./api/client";
import { setSharedRealtime } from "./api/realtime";
import "./styles.css";

async function boot() {
  if (MOCK) {
    const { installMockApi, mockRealtime } = await import("./mock/mockApi");
    installMockApi();
    setSharedRealtime(mockRealtime);
  }
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}

void boot();
