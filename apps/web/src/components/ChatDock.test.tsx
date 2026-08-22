import { afterEach, describe, expect, it, vi } from "vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";

vi.mock("../api/client", () => ({
  api: { agent: vi.fn(), agentTurns: vi.fn() },
}));

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
} from "./ChatDock";
import { api } from "../api/client";
import type { AgentTotals, AgentTurn, AgentTurnsResponse } from "../api/types";
import type { Realtime } from "../api/realtime";

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

    const select = screen.getByLabelText("Provider") as HTMLSelectElement;
    await waitFor(() => expect(select.value).toBe("anthropic"));
    // The default model stays a placeholder, not an explicit choice.
    const modelInput = screen.getByLabelText("Model") as HTMLInputElement;
    expect(modelInput.value).toBe("");
    expect(modelInput.placeholder).toBe("claude-sonnet-5");

    fireEvent.change(select, { target: { value: "openai" } });
    expect(modelInput.placeholder).toBe("gpt-5");
    fireEvent.change(modelInput, { target: { value: "gpt-5-mini" } });
    fireEvent.change(screen.getByPlaceholderText("Ask the agent…"), {
      target: { value: "do the thing" },
    });
    fireEvent.click(screen.getByText("Send"));

    await waitFor(() =>
      expect(api.agent).toHaveBeenCalledWith(
        "d1",
        "do the thing",
        null,
        "openai",
        "gpt-5-mini",
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
