// Contract: .hickory-source/frontends/contract.md
import { describe, expect, it, vi } from "vitest";
import fixture from "../../../simple-contract/layout-cases.json";
import { activate, groups, initialLayout, move, remove, split } from "./layout";
import { TerminalSession } from "./Terminal";
import { Buffer } from "./buffers";
import { api } from "../api/client";
vi.mock("../api/client", () => ({ api: { saveFile: vi.fn(), saveDoc: vi.fn(), openTerminal: vi.fn(), terminalInput: vi.fn(), agent: vi.fn() } }));
describe("shared frontend layout contract", () => {
  for (const scenario of fixture.cases) it(scenario.name, () => {
    let layout = initialLayout();
    for (const step of scenario.steps) {
      const group = "group" in step ? groups(layout)[step.group!].id : "";
      if (step.op === "open") layout = activate(layout, group, step.tab!);
      if (step.op === "move") layout = move(layout, step.tab!, group);
      if (step.op === "remove") layout = remove(layout, step.tab!);
      if (step.op === "split" && "axis" in step && "id" in step) layout = split(layout, group, step.axis as "vertical", step.id!);
    }
    expect(groups(layout).map(group => group.tabs)).toEqual(scenario.expected);
    for (const group of groups(layout)) expect(group.active === null || group.tabs.includes(group.active)).toBe(true);
  });
  it("refuses an unknown move target without removing the document", () => {
    const layout = activate(initialLayout(), "document-group", "notes.md");
    expect(move(layout, "notes.md", "missing")).toBe(layout);
  });
});
describe("simple frontend save safety", () => {
  const plain = { name: "notes.txt", path: "notes.txt", dir: false };
  it("retains edits after a conflict and sends the original hash", async () => {
    vi.mocked(api.saveFile).mockRejectedValueOnce(new Error("changed on disk"));
    const buffer = new Buffer(plain, "before", "hash-before", () => {});
    buffer.edit("mine"); await buffer.save();
    expect(api.saveFile).toHaveBeenLastCalledWith("notes.txt", "mine", "hash-before");
    expect(buffer.text).toBe("mine"); expect(buffer.base).toBe("before"); expect(buffer.dirty).toBe(true);
    expect(buffer.error).toContain("edits are retained");
  });
  it("does not mark later keystrokes as saved", async () => {
    let finish!: (value: { path: string; hash: string }) => void;
    vi.mocked(api.saveFile).mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }));
    const buffer = new Buffer(plain, "before", "hash-before", () => {});
    buffer.edit("submitted"); const saving = buffer.save(); buffer.edit("typed later");
    finish({ path: plain.path, hash: "hash-submitted" }); await saving;
    expect(buffer.base).toBe("submitted"); expect(buffer.text).toBe("typed later"); expect(buffer.dirty).toBe(true);
  });
  it("uses source and disk expectations for literate source", async () => {
    vi.mocked(api.saveDoc).mockResolvedValueOnce({ id: "doc", path: "source.md", source: "next", updated_at: "" });
    const buffer = new Buffer({ ...plain, doc_id: "doc" }, "before", "hash", () => {});
    buffer.edit("next"); await buffer.save();
    expect(api.saveDoc).toHaveBeenLastCalledWith("doc", "next", { source: "before", disk: "before" });
  });
  it("does not edit or save generated outputs", async () => {
    const buffer = new Buffer({ ...plain, generated_by: "owner" }, "generated", "hash", () => {});
    const calls = vi.mocked(api.saveFile).mock.calls.length;
    buffer.edit("overwrite"); await buffer.save();
    expect(buffer.text).toBe("generated"); expect(vi.mocked(api.saveFile).mock.calls.length).toBe(calls);
  });
});
describe("terminal and agent tab identity", () => {
  it("opens the terminal once and retains unsent input when shown again", async () => {
    vi.mocked(api.openTerminal).mockResolvedValueOnce({ id: "pty" } as Awaited<ReturnType<typeof api.openTerminal>>);
    const session = new TerminalSession(); session.input = "pending input";
    await session.open(); await session.open();
    expect(api.openTerminal).toHaveBeenCalledTimes(1); expect(session.id).toBe("pty"); expect(session.input).toBe("pending input");
  });
  it("retains input after a refused terminal write without retry", async () => {
    vi.mocked(api.terminalInput).mockRejectedValueOnce(new Error("disconnected"));
    const session = new TerminalSession(); session.id = "pty"; session.input = "echo hello";
    await session.send();
    expect(api.terminalInput).toHaveBeenCalledTimes(1);
    expect(api.terminalInput).toHaveBeenLastCalledWith("pty", "echo hello\r");
    expect(session.input).toBe("echo hello"); expect(session.error).toContain("not retried");
  });

});
