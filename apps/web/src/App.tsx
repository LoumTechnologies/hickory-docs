import { useEffect, useState } from "react";
import { api, getToken, setToken } from "./api/client";
import type { User } from "./api/types";
import { navigate, useRoute } from "./router";
import { LoginView } from "./views/LoginView";
import { ProjectsView } from "./views/ProjectsView";
import { DocumentView } from "./views/DocumentView";
import { PricingView } from "./views/PricingView";

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
  const needsAuth = route.name !== "login" && route.name !== "pricing";

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
        {route.name === "pricing" ? (
          <PricingView />
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
