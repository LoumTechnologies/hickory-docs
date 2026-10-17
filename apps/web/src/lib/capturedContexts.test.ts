import { expect, it, vi } from "vitest";
import { request } from "../api/client";
import { acquireCapturedContext, retainCapturedContext } from "./capturedContexts";
vi.mock("../api/client", () => ({request:vi.fn()}));
it("shares a complete endpoint across patch panes and retains it throughout debugging", async () => {
  vi.mocked(request).mockResolvedValue({id:"same",uri:"hick-project:///same/"});
  const entries={"main.py":{bytes:[1]},"library.py":{bytes:[2]}};
  const [caller,callee]=await Promise.all([acquireCapturedContext(entries),acquireCapturedContext({...entries})]);
  expect(request).toHaveBeenCalledTimes(1);
  expect(caller.uri).toBe(callee.uri);
  const debug=retainCapturedContext(caller.uri);
  caller.release();callee.release();
  expect(request).not.toHaveBeenCalledWith("DELETE",expect.anything());
  const reopened=await acquireCapturedContext(entries);
  expect(reopened.uri).toBe(caller.uri);
  expect(request).toHaveBeenCalledTimes(1);
  reopened.release();debug();
  expect(request).toHaveBeenLastCalledWith("DELETE","/api/project-contexts/same");
});
