import { EditorView } from "@codemirror/view";
import { responseStart } from "../editor/protectedPrefix";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  cleanup,
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";

vi.mock("../api/client", () => ({
  api: { agent: vi.fn(), agentTurns: vi.fn(), agentStop: vi.fn(), executor: vi.fn().mockResolvedValue({}), sessionView: vi.fn().mockRejectedValue(new Error("No session fixture")), complete: vi.fn().mockResolvedValue({ suggestions: [] }) },
}));

vi.mock("../api/acp", () => ({ acpApi: { catalogue: vi.fn().mockResolvedValue({ agents: [] }), connect: vi.fn(), state: vi.fn() } }));

import {
  ChatDock,
  appendStream,
  branchOf,
  cacheHitRate,
  childrenOf,
  deepestFrom,
  formatTokens,
  formatUsd,
  statsLine,
  REWIND_TIP,
  RERUN_TIP,
  SLASH_HELP,
} from "./ChatDock";
import { api } from "../api/client";
import { acpApi } from "../api/acp";
import type { AgentTotals, AgentTurn, AgentTurnsResponse } from "../api/types";
import type { Realtime } from "../api/realtime";

afterEach(() => { localStorage.clear(); });

// Protects docs/guarantees/agent/acp-agents-are-first-class.md.
describe("switching agents", () => {
  afterEach(() => { cleanup(); vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [] }); });
  it("detects adapters, remembers a choice, and sends an ACP turn from a new thread", async () => {
    const agents = [{ id: "codex", name: "Codex", command: "codex-acp", args: [], available: true },
      { id: "claude", name: "Claude Agent", command: "claude-agent-acp", args: [], available: false, installable: true }];
    vi.mocked(acpApi.catalogue).mockResolvedValue({ agents });
    vi.mocked(acpApi.connect).mockResolvedValue({ backend: "codex", ready: true });
    vi.mocked(acpApi.state).mockResolvedValue({ backend: "codex", ready: true });
    vi.mocked(api.agentTurns).mockResolvedValue({ turns: [turn("old", null)], backend: "builtin", provider: "anthropic", model: "claude-sonnet-5", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
    vi.mocked(api.agent).mockResolvedValue({ session_id: "acp-turn" });
    const realtime = { onRunEvent: () => () => undefined } as unknown as Realtime;
    const props = { docId: "d1", realtime, onAgentFinished: () => undefined };
    const view = render(<ChatDock {...props} />);
    expect(screen.queryByRole("combobox", { name: "Agent" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Agent settings" }));
    await screen.findByRole("option", { name: "Codex (ACP)" });
    expect(screen.getByRole("option", { name: "Hickory (built-in)" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Claude Agent — install adapter" })).toBeTruthy();
    await waitFor(() => expect(document.querySelector(".cm-content")?.textContent).toContain("p:old"));
    fireEvent.change(screen.getByRole("combobox", { name: "Agent" }), { target: { value: "codex" } });
    await waitFor(() => expect(acpApi.connect).toHaveBeenCalledWith("d1", "codex", undefined));
    expect(localStorage.getItem("hickory.agent")).toBe("codex");
    await typeResponse("Use Codex");
    await waitFor(() => expect((screen.getByRole("button", { name: "Send" }) as HTMLButtonElement).disabled).toBe(false));
    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    await waitFor(() => expect(api.agent).toHaveBeenCalledWith("d1", "Use Codex", null, "anthropic", "", "codex"));
    await screen.findByRole("button", { name: "Stop the agent" });
    expect((screen.getByRole("combobox", { name: "Agent" }) as HTMLSelectElement).value).toBe("codex");
    view.unmount();
    vi.mocked(api.agentTurns).mockResolvedValue({ turns: [], backend: "builtin", provider: "anthropic", model: "claude-sonnet-5", totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 } });
    render(<ChatDock {...props} />);
    fireEvent.click(screen.getByRole("button", { name: "Agent settings" }));
    await waitFor(() => expect((screen.getByRole("combobox", { name: "Agent" }) as HTMLSelectElement).value).toBe("codex"));
    vi.mocked(acpApi.catalogue).mockResolvedValue({ agents: [...agents, { id: "custom", name: "My agent", command: "my-acp", args: [], available: true }] });
    fireEvent.click(screen.getByRole("button", { name: "Refresh agents" }));
    await screen.findByRole("option", { name: "My agent (ACP)" });
  });
});

function turn(id: string, parent: string | null): AgentTurn {
  return {
    id,
    parent_id: parent,
    prompt: `p:${id}`,
    answer: `a:${id}`,
    status: "ok",
    error: null,
    created_at: "2026-01-01T00:00:00Z",
    provider: "anthropic",
    model: "claude-sonnet-5",
    usage: null,
  };
}

//        root
//       /    \
//      a      b        <- a rewind at `root` forked `b`
//      |      |
//      a2     b2
const TREE = [
  turn("root", null),
  turn("a", "root"),
  turn("a2", "a"),
  turn("b", "root"),
  turn("b2", "b"),
];

describe("live stream", () => {
  it("appends LLM tokens and legacy out/err transcript data", () => {
    let s = appendStream("", { kind: "token", data: "Hello " });
    s = appendStream(s, { kind: "token", data: "world" });
    expect(s).toBe("Hello world");
    expect(appendStream("a", { t: 1, kind: "out", data: "b" })).toBe("ab");
    expect(appendStream("a", { t: 1, kind: "err", data: "b" })).toBe("ab");
  });

  it("marks script and tool starts so a long pause reads as work", () => {
    expect(
      appendStream("x", {
        kind: "script_started",
        lang: "python",
        data: "print(1)",
      }),
    ).toContain("[running python script…]");
    expect(
      appendStream("x", {
        kind: "tool_started",
        name: "edit_doc",
        data: "<xml/>",
      }),
    ).toContain("[edit_doc…]");
  });

  it("ignores bookkeeping events instead of dumping them into the preview", () => {
    expect(appendStream("keep", { kind: "thinking" })).toBe("keep");
    expect(appendStream("keep", { kind: "turn_usage" })).toBe("keep");
    expect(
      appendStream("keep", {
        kind: "script_finished",
        result: { exit_code: 0, stdout: "s", stderr: "" },
      }),
    ).toBe("keep");
  });
});

describe("conversation tree", () => {
  it("shows the path from the root to the selected tip, oldest first", () => {
    expect(branchOf(TREE, "a2").map((t) => t.id)).toEqual(["root", "a", "a2"]);
    expect(branchOf(TREE, "b2").map((t) => t.id)).toEqual(["root", "b", "b2"]);
  });

  it("returns nothing for an empty or unknown tip rather than throwing", () => {
    expect(branchOf(TREE, null)).toEqual([]);
    expect(branchOf(TREE, "nope")).toEqual([]);
  });

  it("terminates on a cycle instead of hanging the dock", () => {
    const cyclic = [turn("x", "y"), turn("y", "x")];
    expect(branchOf(cyclic, "x").length).toBeLessThanOrEqual(2);
  });

  it("lists the alternatives at a fork, so the switcher can page through them", () => {
    expect(childrenOf(TREE, "root").map((t) => t.id)).toEqual(["a", "b"]);
    expect(childrenOf(TREE, null).map((t) => t.id)).toEqual(["root"]);
    expect(childrenOf(TREE, "a2")).toEqual([]);
  });

  it("switching branches lands on that branch's newest tip, not its fork point", () => {
    expect(deepestFrom(TREE, "b")).toBe("b2");
    expect(deepestFrom(TREE, "a")).toBe("a2");
    expect(deepestFrom(TREE, "a2")).toBe("a2");
  });
});

// The two suites below protect
// docs/guarantees/agent/the-dock-reports-spend-and-runs-the-chosen-model.md.
describe("session stats formatting", () => {
  it("compacts token counts the way the header has room for", () => {
    expect(formatTokens(0)).toBe("0");
    expect(formatTokens(950)).toBe("950");
    expect(formatTokens(12_400)).toBe("12.4k");
    expect(formatTokens(250_000)).toBe("250k");
    expect(formatTokens(3_100_000)).toBe("3.1M");
  });

  it("shows USD to four places, and a dash when the price is unknown", () => {
    expect(formatUsd(0.03419)).toBe("$0.0342");
    expect(formatUsd(0)).toBe("$0.0000");
    expect(formatUsd(null)).toBe("$—");
  });

  it("computes the cache hit rate as reads over reads-plus-uncached-input", () => {
    expect(cacheHitRate(100, 300)).toBeCloseTo(0.75);
    expect(cacheHitRate(0, 0)).toBeNull();
  });

  it("renders the one-line summary, dashing what is unknowable", () => {
    const totals: AgentTotals = {
      usd: 0.0342,
      input: 12_400,
      output: 3_100,
      cache_read: 44_000,
      cache_write: 500,
    };
    expect(statsLine(totals)).toBe("$0.0342 · in 12.4k · out 3.1k · cache 78%");
    expect(
      statsLine({
        usd: null,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_write: 0,
      }),
    ).toBe("$— · in 0 · out 0 · cache —");
  });
});

describe("the dock's model control and stats line", () => {
  afterEach(cleanup);

  const realtime = { onRunEvent: () => () => undefined } as unknown as Realtime;

  function listing(
    overrides: Partial<AgentTurnsResponse> = {},
  ): AgentTurnsResponse {
    return {
      turns: [],
      provider: "anthropic",
      model: "claude-sonnet-5",
      totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 },
      ...overrides,
    };
  }

  function dock() {
    return render(
      <ChatDock
        docId="d1"
        realtime={realtime}
        collapsed={false}
        onToggleCollapsed={() => undefined}
        onAgentFinished={() => undefined}
      />,
    );
  }

  it("hydrates the provider select from the server's resolved default and posts the chosen values", async () => {
    vi.mocked(api.agentTurns).mockResolvedValue(listing());
    vi.mocked(api.agent).mockResolvedValue({ session_id: "s1" });
    dock();

    fireEvent.click(screen.getByRole("button", { name: "Agent settings" }));
    const select = screen.getByLabelText("Provider") as HTMLSelectElement;
    await waitFor(() => expect(select.value).toBe("anthropic"));
    // The default model stays a placeholder, not an explicit choice.
    const modelInput = screen.getByLabelText("Model") as HTMLInputElement;
    expect(modelInput.value).toBe("");
    expect(modelInput.placeholder).toBe("claude-sonnet-5");

    fireEvent.change(select, { target: { value: "openai" } });
    expect(modelInput.placeholder).toBe("gpt-5");
    fireEvent.change(modelInput, { target: { value: "gpt-5-mini" } });
    await typeResponse("do the thing");
    fireEvent.click(screen.getByText("Send"));

    await waitFor(() =>
      expect(api.agent).toHaveBeenCalledWith(
        "d1",
        "do the thing",
        null,
        "openai",
        "gpt-5-mini",
        "builtin",
      ),
    );
  });

  it("shows the stats line with its full-breakdown tooltip once usage exists", async () => {
    vi.mocked(api.agentTurns).mockResolvedValue(
      listing({
        totals: {
          usd: 0.0342,
          input: 12_400,
          output: 3_100,
          cache_read: 44_000,
          cache_write: 500,
        },
      }),
    );
    dock();

    const stats = await screen.findByText(
      "$0.0342 · in 12.4k · out 3.1k · cache 78%",
    );
    expect(stats.dataset.tip).toContain("cache read 44,000 tokens");
    expect(stats.dataset.tip).toContain("cache write 500 tokens");
  });

  it("shows no stats line before any turn has reported usage", async () => {
    vi.mocked(api.agentTurns).mockResolvedValue(listing());
    const { container } = dock();
    await waitFor(() => expect(api.agentTurns).toHaveBeenCalled());
    expect(container.querySelector(".chat-stats")).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// Moving around the tree from the keyboard, and zooming out of it
// ---------------------------------------------------------------------------
import { parseSlash, rewindFrom } from "./ChatDock";
import { pathTo } from "./ChatTree";

// Guarantee: docs/guarantees/agent/a-session-is-the-conversation.md
describe("slash commands and the tree", () => {
  it("parses /rewind, /rewind N, /tree, /new, /help and nothing else", () => {
    expect(parseSlash("/rewind")).toEqual({ kind: "rewind", steps: 1 });
    expect(parseSlash("/rewind 3")).toEqual({ kind: "rewind", steps: 3 });
    expect(parseSlash("/TREE")).toEqual({ kind: "tree" });
    expect(parseSlash("/new")).toEqual({ kind: "new" });
    expect(parseSlash("/help")).toEqual({ kind: "help" });
    expect(parseSlash("/usr/bin/env is a path")).toBeNull();
    expect(parseSlash("rewind please")).toBeNull();
  });

  it("rewinds along parent pointers and stops at the root", () => {
    const turns = [
      turn("a", null),
      turn("b", "a"),
      turn("c", "b"),
      turn("d", "b"),
    ];
    expect(rewindFrom(turns, "c", 1)).toBe("b");
    expect(rewindFrom(turns, "c", 2)).toBe("a");
    expect(rewindFrom(turns, "c", 9)).toBeNull();
    expect(rewindFrom(turns, null, 1)).toBeNull();
  });

  it("the path to the tip is the branch the dock shows", () => {
    const turns = [
      turn("a", null),
      turn("b", "a"),
      turn("c", "b"),
      turn("d", "b"),
    ];
    expect([...pathTo(turns, "d")].sort()).toEqual(["a", "b", "d"]);
    expect(pathTo(turns, "c").has("d")).toBe(false);
  });
});

// docs/guarantees/agent/rewind-and-re-run-are-two-different-acts.md
describe("rewind and re-run are named apart", () => {
  it("parses both, and only as whole commands", () => {
    expect(parseSlash("/rerun")).toEqual({ kind: "rerun" });
    expect(parseSlash("/rewind 2")).toEqual({ kind: "rewind", steps: 2 });
    // A path or a date at the start of a sentence must not be eaten.
    expect(parseSlash("/rerun the thing")).toBeNull();
  });

  it("says what each act KEEPS, which is the only difference", () => {
    // Rewind keeps the change of mind, visible in the file forever; a re-run
    // does not, and the first attempt becomes a draft you may discard.
    expect(REWIND_TIP).toMatch(/THIS conversation/);
    expect(REWIND_TIP).toMatch(/kept/);
    expect(RERUN_TIP).toMatch(/SECOND conversation/);
    expect(RERUN_TIP).toMatch(/not kept/);
    expect(RERUN_TIP).toMatch(/gitignored/);
  });

  it("offers both in the help, with the difference stated", () => {
    expect(SLASH_HELP).toMatch(/keeping the branch/);
    expect(SLASH_HELP).toMatch(/discarding this one/);
  });
});

// A runaway turn — a model looping mid-stream — bills tokens until somebody
// pulls the cord, and "close the whole program" must never be the only cord.
describe("stopping a run", () => {
  afterEach(cleanup);

  const realtime = { onRunEvent: () => () => undefined } as unknown as Realtime;

  function listing(turns: AgentTurn[] = []): AgentTurnsResponse {
    return {
      turns,
      provider: "anthropic",
      model: "claude-sonnet-5",
      totals: { usd: 0, input: 0, output: 0, cache_read: 0, cache_write: 0 },
    };
  }

  function dock() {
    return render(
      <ChatDock
        docId="d1"
        realtime={realtime}
        collapsed={false}
        onToggleCollapsed={() => undefined}
        onAgentFinished={() => undefined}
      />,
    );
  }

  it("replaces Send with a Stop button while a turn runs, and it pulls the cord", async () => {
    vi.mocked(api.agentTurns).mockResolvedValue(listing());
    vi.mocked(api.agent).mockResolvedValue({ session_id: "s1" });
    vi.mocked(api.agentStop).mockResolvedValue({ stopping: "s1" });
    dock();

    await typeResponse("loop forever");
    fireEvent.click(screen.getByText("Send"));

    const stop = await screen.findByRole("button", { name: "Stop the agent" });
    // The Send button is gone — one verb at a time — and Stop is LIVE, never
    // disabled while the run is: a disabled stop is no stop at all.
    expect(screen.queryByText("Send")).toBeNull();
    expect((stop as HTMLButtonElement).disabled).toBe(false);

    fireEvent.click(stop);
    await waitFor(() => expect(api.agentStop).toHaveBeenCalledWith("d1"));
    // Pressed once: the button says so and refuses a second pull while the
    // first is in flight.
    expect(stop.textContent).toContain("Stopping…");
    expect((stop as HTMLButtonElement).disabled).toBe(true);
  });

  it("renders a stopped turn quietly — the user's own act, never a red error", async () => {
    const stopped: AgentTurn = {
      ...turn("s1", null),
      answer: null,
      status: "stopped",
      error: "stopped by you",
    };
    vi.mocked(api.agentTurns).mockResolvedValue(listing([stopped]));
    const { container } = dock();

    await screen.findByText(/Stopped by you/);
    expect(container.querySelector(".chat-error")).toBeNull();
  });
});

async function typeResponse(text: string) {
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const view = EditorView.findFromDOM(host)!;
  act(() => view.dispatch({ changes: { from: view.state.field(responseStart), to: view.state.doc.length, insert: text }, userEvent: "input.type" }));
}
