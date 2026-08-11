import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { installMockHandler } from "../api/client";
import type { LlmKeysResponse, User } from "../api/types";
import { SettingsView } from "./SettingsView";

const USER: User = { id: "u1", email: "someone@example.com", plan: "open" };

let response: LlmKeysResponse;

beforeEach(() => {
  installMockHandler(async (_method, path) => {
    if (path === "/api/me/llm-keys") return response;
    throw new Error(`unexpected ${path}`);
  });
});

// vitest runs without `globals` here, so Testing Library's automatic cleanup
// is never registered and renders would stack across tests.
afterEach(cleanup);

describe("the API keys screen", () => {
  it("tells a byo_key account with no key why the agent will not run", async () => {
    response = { keys: [], storage_available: true, plan_agent: "byo_key" };
    render(<SettingsView user={USER} />);
    await waitFor(() =>
      expect(screen.getByText(/the agent cannot run/i)).toBeTruthy(),
    );
  });

  it("shows only the last four characters of a stored key", async () => {
    response = {
      keys: [
        {
          provider: "anthropic",
          last4: "8f2a",
          model: null,
          active: true,
          created_at: "2026-01-01T00:00:00Z",
          last_used_at: null,
        },
      ],
      storage_available: true,
      plan_agent: "byo_key",
    };
    const { container } = render(<SettingsView user={USER} />);
    await waitFor(() => expect(screen.getByText("••••8f2a")).toBeTruthy());
    // A single key is in use without anyone having chosen it, and there is no
    // "use this" button to press because there is no alternative.
    expect(screen.getByText("in use")).toBeTruthy();
    expect(container.textContent).not.toContain("Use this");
  });

  it("asks which key to use only once there is a choice to make", async () => {
    response = {
      keys: [
        {
          provider: "anthropic",
          last4: "aaaa",
          model: null,
          active: false,
          created_at: "2026-01-01T00:00:00Z",
          last_used_at: null,
        },
        {
          provider: "xai",
          last4: "bbbb",
          model: null,
          active: false,
          created_at: "2026-01-01T00:00:00Z",
          last_used_at: null,
        },
      ],
      storage_available: true,
      plan_agent: "byo_key",
    };
    render(<SettingsView user={USER} />);
    await waitFor(() =>
      expect(screen.getByText(/none is selected/i)).toBeTruthy(),
    );
    expect(screen.getAllByText("Use this").length).toBe(2);
  });

  it("explains an unconfigured deployment instead of offering a form that cannot work", async () => {
    response = { keys: [], storage_available: false, plan_agent: "byo_key" };
    const { container } = render(<SettingsView user={USER} />);
    await waitFor(() =>
      expect(screen.getByText(/KEY_ENCRYPTION_KEY/)).toBeTruthy(),
    );
    expect(container.querySelector(".settings-key-form")).toBeNull();
  });

  it("tells a metered account that its own key spares the allowance", async () => {
    response = {
      keys: [],
      storage_available: true,
      plan_agent: "metered_allowance",
    };
    render(<SettingsView user={{ ...USER, plan: "team" }} />);
    await waitFor(() =>
      expect(screen.getByText(/allowance unspent/i)).toBeTruthy(),
    );
  });
});
