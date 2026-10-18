import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
const request = vi.hoisted(() => vi.fn());
vi.mock("../api/client", () => ({ request }));
import { ReviewPanel } from "./ReviewPanel";
import { openReview } from "../lib/reviews";
afterEach(() => { cleanup(); request.mockReset(); });
// Guarantee: docs/guarantees/authoring/review-threads-preserve-the-questioned-target.md
it("posts the captured target and workspace scope and retains its quotation", async () => {
  request.mockImplementation(async (method: string) => method === "GET" ? { scope:"workspace",events:[],targets:[] } : {thread:"review-1"});
  render(<ReviewPanel />);
  const target={kind:"claim",path:"spec.md",quote:"The whole operation is bounded",source_hash:"a".repeat(64),lines:[1,1] as [number,number]};
  openReview(target);
  await screen.findByText(target.quote);
  await waitFor(() => expect(request).toHaveBeenCalledWith("GET","/api/reviews"));
  fireEvent.change(screen.getByLabelText("Comment or reason"),{target:{value:"Only each attempt is bounded"}});
  fireEvent.click(screen.getByRole("button",{name:"Save comment"}));
  await waitFor(() => expect(request).toHaveBeenCalledWith("POST","/api/reviews",{scope:"workspace",thread:null,target,action:"comment",body:"Only each attempt is bounded"}));
});
