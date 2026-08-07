import { useState } from "react";
import { api } from "../api/client";
import { navigate } from "../router";

/** Mirrors the server's rule, so the failure is local instead of a round trip. */
const MIN_PASSWORD = 8;

/**
 * Landing page for the link in a password-reset email.
 *
 * Unlike verification this does NOT act on mount: redeeming needs the new
 * password, and the token is single use — spending it before the form is
 * filled in would strand the user with a dead link.
 */
export function ResetView({ token }: { token: string }) {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (password.length < MIN_PASSWORD) {
      setError(`Password must be at least ${MIN_PASSWORD} characters.`);
      return;
    }
    if (password !== confirm) {
      setError("Those passwords do not match.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await api.confirmReset(token, password);
      setDone(true);
    } catch (err) {
      setError((err as Error).message || "That link could not be used.");
    } finally {
      setBusy(false);
    }
  };

  if (done) {
    return (
      <div className="auth-page">
        <div className="auth-card">
          <h1>Password changed</h1>
          <p>You can sign in with your new password.</p>
          <button
            className="btn btn-primary"
            onClick={() => navigate("/login")}
          >
            Sign in
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="auth-page">
      <div className="auth-card">
        <h1>Choose a new password</h1>
        <form onSubmit={submit}>
          <label>
            New password
            <input
              type="password"
              autoComplete="new-password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </label>
          <label>
            Confirm password
            <input
              type="password"
              autoComplete="new-password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              required
            />
          </label>
          {error && <p className="error">{error}</p>}
          <button className="btn btn-primary" type="submit" disabled={busy}>
            {busy ? "Saving…" : "Change password"}
          </button>
        </form>
        <p className="muted">
          Reset links are single use and expire after an hour.
        </p>
      </div>
    </div>
  );
}
