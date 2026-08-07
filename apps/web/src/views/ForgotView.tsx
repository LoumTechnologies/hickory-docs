import { useState } from "react";
import { api } from "../api/client";
import { navigate } from "../router";

/**
 * Request a password-reset link.
 *
 * The confirmation is deliberately the same whether or not the address has an
 * account, mirroring the server, which always answers 200. Saying "no such
 * account" here would leak the user list just as surely as saying it in the
 * response body.
 */
export function ForgotView() {
  const [email, setEmail] = useState("");
  const [sent, setSent] = useState(false);
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setBusy(true);
    try {
      await api.requestReset(email);
    } catch {
      // Swallowed on purpose. A failure here is either a transport problem or
      // an unconfigured mailer; either way, reporting it differently from
      // success would tell an attacker something about the address.
    } finally {
      setBusy(false);
      setSent(true);
    }
  };

  if (sent) {
    return (
      <div className="auth-page">
        <div className="auth-card">
          <h1>Check your email</h1>
          <p>
            If an account exists for <strong>{email}</strong>, a reset link is
            on its way. It expires in an hour.
          </p>
          <button className="btn" onClick={() => navigate("/login")}>
            Back to sign in
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="auth-page">
      <div className="auth-card">
        <h1>Reset your password</h1>
        <form onSubmit={submit}>
          <label>
            Email
            <input
              type="email"
              autoComplete="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
            />
          </label>
          <button className="btn btn-primary" type="submit" disabled={busy}>
            {busy ? "Sending…" : "Send reset link"}
          </button>
        </form>
        <button className="btn btn-link" onClick={() => navigate("/login")}>
          Back to sign in
        </button>
      </div>
    </div>
  );
}
