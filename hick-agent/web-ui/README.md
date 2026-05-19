# Hick agent web UI

Vite 6 + React 19 + TypeScript + Tailwind CSS. The production build in `dist/` is **committed**
so `hick-agent-sdk` can embed it with `include_bytes!` — Rust builds do not require Node.js.

Microsoft Adaptive Cards for a future renderer (task-37.6) are pulled in via the npm package
[`adaptivecards`](https://www.npmjs.com/package/adaptivecards) (there is no `@microsoft/adaptivecards`
scope on npm; that name in planning docs refers to this library).

## Commands

From the repository root:

- `just build-web-ui` — install deps and `npm run build` (refresh `dist/` **and commit** when the UI changes).
- `just dev-web-ui` — Vite dev server; `/ws` is proxied to `http://127.0.0.1:7890` so you can hack on the UI while a local agent serves the WebSocket.

Or from this directory: `npm install`, `npm run dev`, `npm run build`, `npm run lint`.

## WebSocket URL

Default: same-origin `/ws` (matches the axum server on port 7890). When the UI is served under a
relay path such as `/session/<uuid>/` (task-37.8), the hook connects to that path on the same host.
Optional hash overrides: `ws`, `wsUrl`, or `relay_ws` (percent-encoded full `ws:` / `wss:` URL).
