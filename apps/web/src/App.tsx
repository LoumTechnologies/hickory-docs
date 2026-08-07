import { useEffect, useState } from "react";
import { api, getToken, setToken } from "./api/client";
import type { User } from "./api/types";
import { navigate, useRoute } from "./router";
import { LoginView } from "./views/LoginView";
import { ProjectsView } from "./views/ProjectsView";
import { DocumentView } from "./views/DocumentView";
import { PricingView } from "./views/PricingView";
import { VerifyView } from "./views/VerifyView";
import { ResetView } from "./views/ResetView";
import { ForgotView } from "./views/ForgotView";
import { VerifyBanner } from "./components/VerifyBanner";

export function App() {
  const route = useRoute();
  const [user, setUser] = useState<User | null>(null);
  const [checked, setChecked] = useState(false);

  useEffect(() => {
    if (!getToken()) {
      setChecked(true);
      return;
    }
    api.me().then(
      (u) => {
        setUser(u);
        setChecked(true);
      },
      () => {
        setToken(null);
        setChecked(true);
      },
    );
  }, []);

  const logout = () => {
    setToken(null);
    setUser(null);
    navigate("/login");
  };

  if (!checked) return null;

  const authed = user !== null;
  // Routes reached from an email link must work while signed out: the link is
  // often opened on a different device or browser from the one that signed up.
  const publicRoute =
    route.name === "login" ||
    route.name === "pricing" ||
    route.name === "verify" ||
    route.name === "reset" ||
    route.name === "forgot";
  const needsAuth = !publicRoute;

  return (
    <div className="app">
      <nav className="topnav">
        <button className="wordmark" onClick={() => navigate(authed ? "/projects" : "/login")}>
          Hickory Docs
        </button>
        <div className="nav-actions">
          <button className="btn btn-link" onClick={() => navigate("/pricing")}>
            Pricing
          </button>
          {authed ? (
            <>
              <span className="muted">{user.email}</span>
              <button className="btn btn-link" onClick={logout}>
                Log out
              </button>
            </>
          ) : (
            <button className="btn btn-link" onClick={() => navigate("/login")}>
              Log in
            </button>
          )}
        </div>
      </nav>
      <main className="content">
        {authed && <VerifyBanner user={user} />}
        {route.name === "pricing" ? (
          <PricingView />
        ) : route.name === "verify" ? (
          <VerifyView
            token={route.token}
            onVerified={() =>
              setUser((u) => (u ? { ...u, email_verified: true } : u))
            }
          />
        ) : route.name === "reset" ? (
          <ResetView token={route.token} />
        ) : route.name === "forgot" ? (
          <ForgotView />
        ) : !authed && needsAuth ? (
          <LoginView onAuth={setUser} />
        ) : route.name === "login" ? (
          authed ? (
            <ProjectsView />
          ) : (
            <LoginView onAuth={setUser} />
          )
        ) : route.name === "doc" ? (
          <DocumentView docId={route.id} />
        ) : route.name === "project" ? (
          <ProjectsView projectId={route.id} />
        ) : (
          <ProjectsView />
        )}
      </main>
    </div>
  );
}
