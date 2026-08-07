import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { VerifyBanner } from "./VerifyBanner";
import type { User } from "../api/types";

afterEach(cleanup);

function user(over: Partial<User> = {}): User {
  return { id: "u1", email: "a@b.test", ...over };
}

describe("VerifyBanner", () => {
  it("prompts an unconfirmed account when the deployment can send mail", () => {
    render(
      <VerifyBanner
        user={user({ email_verified: false, verification_required: true })}
      />,
    );
    expect(screen.getByText(/confirm/i)).toBeTruthy();
    expect(screen.getByRole("button", { name: /resend/i })).toBeTruthy();
  });

  it("says nothing once the address is confirmed", () => {
    const { container } = render(
      <VerifyBanner
        user={user({ email_verified: true, verification_required: true })}
      />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("says nothing when the deployment cannot send mail", () => {
    // Every account is unverified on an instance with no mailer, and nothing
    // the user does can change that. A banner there is a permanent scold
    // about a situation they cannot affect — which is exactly why the server
    // reports `verification_required` separately from `email_verified`.
    const { container } = render(
      <VerifyBanner
        user={user({ email_verified: false, verification_required: false })}
      />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("says nothing when the server did not report either field", () => {
    // An older server, or a cached response. Absent information must not be
    // read as "unverified".
    const { container } = render(<VerifyBanner user={user()} />);
    expect(container.innerHTML).toBe("");
  });
});
