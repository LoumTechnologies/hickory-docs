import { useEffect, useState } from "react";

// Tiny hash router — deliberately no routing dependency.

export type Route =
  | { name: "login" }
  | { name: "projects" }
  | { name: "project"; id: string }
  | { name: "doc"; id: string }
  | { name: "pricing" }
  // Token-bearing routes reached from an email link. The token stays in the
  // hash, which never reaches the server as a query string.
  | { name: "verify"; token: string }
  | { name: "reset"; token: string }
  | { name: "forgot" };

export function parseRoute(hash: string): Route {
  const path = hash.replace(/^#/, "") || "/";
  let m: RegExpMatchArray | null;
  if (path === "/login") return { name: "login" };
  if (path === "/pricing") return { name: "pricing" };
  if (path === "/forgot") return { name: "forgot" };
  if ((m = path.match(/^\/verify\?token=(.+)$/)))
    return { name: "verify", token: decodeURIComponent(m[1]) };
  if ((m = path.match(/^\/reset\?token=(.+)$/)))
    return { name: "reset", token: decodeURIComponent(m[1]) };
  if ((m = path.match(/^\/projects\/([^/]+)$/))) return { name: "project", id: m[1] };
  if ((m = path.match(/^\/docs\/([^/]+)$/))) return { name: "doc", id: m[1] };
  return { name: "projects" };
}

export function navigate(path: string) {
  location.hash = path;
}

export function useRoute(): Route {
  const [route, setRoute] = useState<Route>(() => parseRoute(location.hash));
  useEffect(() => {
    const onChange = () => setRoute(parseRoute(location.hash));
    window.addEventListener("hashchange", onChange);
    return () => window.removeEventListener("hashchange", onChange);
  }, []);
  return route;
}
