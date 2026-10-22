import { afterEach, expect, it, vi } from "vitest";
import { request } from "../api/client";
import { retainCapturedContext } from "./capturedContexts";
import { openReading, publishReading } from "./readingViews";
import { openCapturedCode } from "./capturedCode";
vi.mock("../api/client", () => ({ request: vi.fn() }));
vi.mock("./capturedContexts", () => ({ retainCapturedContext: vi.fn() }));
vi.mock("./readingViews", () => ({ publishReading: vi.fn(), openReading: vi.fn() }));
afterEach(() => vi.clearAllMocks());
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
it("releases late captured code instead of publishing/opening for a retired debugger stop", async () => {
  const release = vi.fn(); vi.mocked(retainCapturedContext).mockReturnValue(release);
  let resolve!: (value: unknown) => void;
  vi.mocked(request).mockImplementationOnce(() => new Promise(done => { resolve = done; }));
  let current = true;
  const target = { uri: "hick-project:///fixture/a.py", range: { start: { line: 1, character: 0 }, end: { line: 1, character: 0 } } };
  const loading = openCapturedCode(target, "owner", () => current);
  current = false; resolve({ path: "a.py", bytes: [120], files: [] });
  await expect(loading).rejects.toThrow("owner changed while loading");
  expect(release).toHaveBeenCalledTimes(1); expect(publishReading).not.toHaveBeenCalled(); expect(openReading).not.toHaveBeenCalled();
  await expect(openCapturedCode(target, "owner", () => false)).rejects.toThrow("before loading");
  expect(request).toHaveBeenCalledTimes(1); expect(retainCapturedContext).toHaveBeenCalledTimes(1);
  current = true; vi.mocked(request).mockResolvedValue({ path: "a.py", bytes: [120], files: [] });
  await openCapturedCode(target, "owner", () => current);
  expect(publishReading).toHaveBeenCalledWith(`captured:${target.uri}`, expect.objectContaining({ before: "x", after: "x", debugOwner: "owner", location: target.range.start, dispose: release }));
  expect(openReading).toHaveBeenCalledWith(`captured:${target.uri}`, "a.py");
});
