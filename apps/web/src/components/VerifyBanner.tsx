import { useState } from "react";
import { api } from "../api/client";
import type { User } from "../api/types";

type Sent = "idle" | "sending" | "sent" | "limited" | "failed";

/**
 * Prompts an unconfirmed account to verify, with a resend button.
 *
 * Renders nothing unless verification is both *pending* and *possible*. On a
 * deployment with no mailer every account is unverified and nothing they do
 * can change it, so a banner there would be a permanent scold about a
 * situation the user cannot affect — which is why the server reports
 * `verification_required` separately from `email_verified`.
 */
export function VerifyBanner({ user }: { user: User }) {
  const [state, setState] = useState<Sent>("idle");

  if (!user.verification_required || user.email_verified) return null;

  const resend = async () => {
    setState("sending");
    try {
      const r = await api.sendVerification();
      // The endpoint answers 200 even when the provider rejected the message,
      // because the token IS issued and the link works if it ever arrives.
      // But "check your inbox" is bad advice for mail that was rejected.
      setState(r.status === "send_failed" ? "failed" : "sent");
    } catch (e) {
      // 429 is its own message: "try again" is bad advice when the reason is
      // that they already tried too often.
      const status = (e as { status?: number }).status;
      setState(status === 429 ? "limited" : "failed");
    }
  };

  return (
    <div className="banner banner-warn" role="status">
      <span>
        Confirm <strong>{user.email}</strong> to run documents. Reading and
        editing work already.
      </span>
      {state === "sent" ? (
        <span className="muted">Sent — check your inbox.</span>
      ) : state === "limited" ? (
        <span className="muted">
          Too many requests. Try again in an hour.
        </span>
      ) : state === "failed" ? (
        <span className="muted">
          We could not send that email. Try again shortly.
        </span>
      ) : (
        <button
          className="btn btn-link"
          onClick={resend}
          disabled={state === "sending"}
        >
          {state === "sending" ? "Sending…" : "Resend link"}
        </button>
      )}
    </div>
  );
}
