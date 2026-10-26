<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/useMediaActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-fb9d2ad5052a"></a>
## `apps/web/src/components/useMediaActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-fb9d2ad5052a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-fb9d2ad5052a-expression_statement">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { useRef } from "react";
import { act, cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { useMediaActions } from "./useMediaActions";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-fb9d2ad5052a-Player"></a>
### `Player`

<storycode:copy id="f-fb9d2ad5052a-Player">
function Player({ source }: { source: string }) {
  const ref = useRef<HTMLVideoElement>(null); useMediaActions(ref, source);
  return <video ref={ref} src={source} controls />;
}
</storycode:copy>
<a id="f-fb9d2ad5052a-setup"></a>
### `setup`

<storycode:copy id="f-fb9d2ad5052a-setup">
function setup() {
  const registry = new UiActions({ workspace: "static", window: "media" }, () => true, async () => {});
  const wrap = (source: string) => <UiActionContext.Provider value={registry}><Player source={source} /></UiActionContext.Provider>;
  const app = render(wrap("one.webm"));
  const actions = () => registry.groups().groups.flatMap(parent => registry.observe(parent, 64).actions);
  const run = async (label: string, args: Record<string, UiValue> = {}) => {
    const action = actions().find(action => action.label === label)!;
    let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "parent", kind: "agent" }, { handle: action.handle, arguments: args, request_id: crypto.randomUUID() }); }); return result;
  };
  return { registry, app, wrap, actions, run, media: app.container.querySelector("video")! };
}
</storycode:copy>
<a id="f-fb9d2ad5052a-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-fb9d2ad5052a-expression_statement-2">
it("controls and reads the real media element and invalidates handles when its source changes", async () => {
  const { media, run, actions, app, wrap, registry } = setup();
  const play = vi.spyOn(media, "play").mockResolvedValue(), pause = vi.spyOn(media, "pause").mockImplementation(() => {});
  Object.defineProperty(media, "duration", { configurable: true, value: 20 });
  expect(await run("Play reviewed recording")).toMatchObject({ outcome: { status: "completed", value: { watched_or_heard: "not established" } } });
  expect(play).toHaveBeenCalledTimes(1);
  await run("Pause reviewed recording"); expect(pause).toHaveBeenCalledTimes(1);
  await run("Seek reviewed recording", { seconds: 8 }); expect(media.currentTime).toBe(8);
  await run("Set reviewed recording sound", { volume: 0.25, muted: true }); expect(media.volume).toBe(0.25); expect(media.muted).toBe(true);
  fireEvent.volumeChange(media);
  const read = await run("Read recorded media status bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(JSON.parse(new TextDecoder().decode(new Uint8Array(read.outcome.value.bytes)))).toMatchObject({ time: 8, volume: 0.25, muted: true });
  const old = actions().find(action => action.label === "Play reviewed recording")!;
  app.rerender(wrap("two.webm"));
  await act(async () => { expect(await registry.act(registry.scope, { id: "parent", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old-media" })).toMatchObject({ status: "stale" }); });
  expect(play).toHaveBeenCalledTimes(1);
});
</storycode:copy>
<a id="f-fb9d2ad5052a-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-fb9d2ad5052a-expression_statement-3">
it("reports real browser activation refusal as required input without claiming playback", async () => {
  const { media, run } = setup();
  vi.spyOn(media, "play").mockRejectedValue(new DOMException("User activation required", "NotAllowedError"));
  expect(await run("Play reviewed recording")).toMatchObject({ outcome: { status: "requires_input", reason: expect.stringContaining("User activation required") } });
  expect(await run("Seek reviewed recording", { seconds: 30 })).toMatchObject({ outcome: { status: "requires_input", reason: expect.stringContaining("loaded metadata") } });
});
</storycode:copy>

### Assemble `apps/web/src/components/useMediaActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/useMediaActions.test.tsx"><storycode:paste select="#f-fb9d2ad5052a-expression_statement"/><storycode:paste select="#f-fb9d2ad5052a-Player"/><storycode:paste select="#f-fb9d2ad5052a-setup"/><storycode:paste select="#f-fb9d2ad5052a-expression_statement-2"/><storycode:paste select="#f-fb9d2ad5052a-expression_statement-3"/></storycode:file>

</storycode:doc>
