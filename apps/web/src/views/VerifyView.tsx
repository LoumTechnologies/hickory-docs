import { useEffect, useRef, useState } from "react";
import { api } from "../api/client";
import { navigate } from "../router";

type State =
  | { status: "working" }
  | { status: "done" }
  | { status: "failed"; message: string };

/**
 * Landing page for the link in a verification email.
 *
 * Redeems on mount, because the user already expressed intent by clicking the
 * link — asking them to press a second button would be ceremony.
 */
export function VerifyView({
  token,
  onVerified,
}: {
  token: string;
  onVerified: () => void;
}) {
  const [state, setState] = useState<State>({ status: "working" });
  // React 18 mounts twice in StrictMode. Tokens are single use, so a second
  // redemption would fail and show an error for a verification that just
  // succeeded.
  const started = useRef(false);

  useEffect(() => {
    if (started.current) return;
    started.current = true;
    api.confirmVerification(token).then(
      () => {
        setState({ status: "done" });
        onVerified();
      },
      (e: Error) =>
        setState({
          status: "failed",
          message: e.message || "That link could not be used.",
        }),
    );
  }, [token, onVerified]);

  return (
    <div className="auth-page">
      <div className="auth-card">
        <h1>Confirming your email</h1>
        {state.status === "working" && <p className="muted">One moment…</p>}
        {state.status === "done" && (
          <>
            <p>Your address is confirmed. You can run documents now.</p>
            <button
              className="btn btn-primary"
              onClick={() => navigate("/projects")}
            >
              Go to your projects
            </button>
          </>
        )}
        {state.status === "failed" && (
          <>
            <p className="error">{state.message}</p>
            <p className="muted">
              Links are single use and expire after 24 hours. Sign in and
              request a new one.
            </p>
            <button className="btn" onClick={() => navigate("/projects")}>
              Continue
            </button>
          </>
        )}
      </div>
    </div>
  );
}
