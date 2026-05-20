---
id: TASK-37.12
title: >-
  Agent panes — CWD static serving, folder-per-pane convention, and sandboxed
  HTML panes
status: Done
assignee: []
created_date: '2026-05-19 18:23'
updated_date: '2026-05-20 10:52'
labels:
  - web
  - sdk
  - security
  - mobile-ready
dependencies:
  - TASK-37.11
parent_task_id: TASK-37
priority: medium
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extends the pane system from TASK-37.11 with three related changes: (1) a richer folder-per-pane directory convention with a YAML config file, (2) a new `html` pane type that renders as a sandboxed iframe with a CSP derived from the agent's network allowlist, and (3) static serving of the entire project CWD so any file the agent can read can also be fetched by the browser without a separate copy step.

## Relationship to TASK-37.11

TASK-37.11 specifies flat `.agents/panes/*.md` files. This task **replaces** that convention with the folder-per-pane layout below. TASK-37.11's `PaneWatcher` implementation should be updated to discover panes via `pane.yaml` rather than raw `.md` files. Everything else in TASK-37.11 (UiEvent shape, TUI rendering, session bundle) is unchanged.

## Folder-per-pane convention

```
.agents/panes/
  source/
    pane.yaml
    content.md
    diagram.png
  dashboard/
    pane.yaml
    index.html
    app.js
    style.css
```

Each pane is a subdirectory of `.agents/panes/` containing a `pane.yaml` config file. The directory name is the stable pane `name` (used as the key in `PaneContent` events).

### `pane.yaml` schema

```yaml
# Required
type: markdown          # "markdown" or "html"
title: "Source Document"
entry: content.md       # filename inside this folder to render

# Optional — html panes only
# If omitted, CSP connect-src is derived solely from the agent's runtime allowlist.
# Values here are additive (union with the runtime allowlist, never subtract).
# Use sparingly — prefer letting the runtime allowlist drive permissions.
extra_connect_src: []   # e.g. ["https://fonts.googleapis.com"]
```

The `PaneWatcher` watches for `pane.yaml` create/modify/delete events (not raw `.md` files). On a change it reads `pane.yaml`, reads the entry file, and emits a `PaneContent` UiEvent. On delete of the directory or `pane.yaml`, it emits a `PaneContent` event with empty `markdown` to clear the pane.

## CWD static file serving

Add a new route to `WebServer`:

```
GET /cwd/{path}
```

Serves the file at `<project_cwd>/{path}`. Rules:

- **Path containment**: resolve the joined path and confirm it is still under `project_cwd`. Reject (404) any path that escapes via `..` or symlinks that point outside.
- Correct `Content-Type` for common extensions (html, js, css, png, svg, jpg, gif, webp, md, json, txt, wasm).
- `Cache-Control: no-store` on all responses (agent output changes frequently).
- No directory listing — if `path` resolves to a directory, return 404.
- Auth is out of scope; noted in the security section below.

Pane files are served via this route. The markdown pane base URL is `/cwd/.agents/panes/{name}/`, so relative image paths resolve automatically.

## HTML pane type — sandboxed iframe

When `pane.yaml` specifies `type: html`:

- The web UI renders the pane as `<iframe sandbox="allow-scripts allow-forms" src="/cwd/.agents/panes/{name}/{entry}">` inside the pane panel.
- The web server reads the agent's current network allowlist from the runtime when serving any file under `.agents/panes/{name}/` where that pane's type is `html`.
- It attaches a `Content-Security-Policy` response header to every file served for that pane:

```
Content-Security-Policy:
  default-src 'self';
  script-src 'self';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data:;
  connect-src <agent-allowlist-origins> <extra_connect_src>;
  frame-ancestors 'self';
  form-action 'none'
```

- If the agent's network allowlist is empty, `connect-src` is `'none'`.
- Origins are derived from the allowlist hostnames with their scheme. Wildcards in the agent allowlist map to the corresponding CSP wildcard form (e.g. `*.example.com` → `https://*.example.com`).
- The `sandbox` attribute on the `<iframe>` element (no `allow-same-origin`) ensures the iframe cannot access the parent DOM, cookies, or storage even if the CSP is permissive.

## CSP generation

Add a `generate_pane_csp(allowlist: &[AllowedHost], extra: &[String]) -> String` function in `hick-agent-sdk`. Unit-test the output for: empty allowlist, single host, multiple hosts, wildcard host, and extra_connect_src entries.

## Session bundle

Snapshot the entire `.agents/panes/` directory (including `pane.yaml` files and all assets) into the bundle at session end, consistent with TASK-37.11. The replay server must apply the same CSP logic when serving html-pane files from the bundle.

## Security notes

- **No auth in this task.** The `/cwd/` route is localhost-only for now. Remote-control auth is tracked separately and will gate this route when remote access is enabled.
- The security argument for no-auth on localhost: anyone with access to the web UI can already ask the agent to read any file, or use the `!` command prefix to run `cat` directly. Static serving does not expand the effective access boundary.
- Never serve files outside `project_cwd`, and never follow symlinks that escape it.
- HTML panes explicitly do not allow `allow-same-origin` in the iframe sandbox — this is load-bearing for DOM isolation and must not be relaxed.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 #1 PaneWatcher updated to discover panes via pane.yaml (not raw .md files); emits PaneContent on entry-file change or pane.yaml change
- [ ] #2 #2 pane.yaml schema validated on load: required fields type/title/entry present, type is 'markdown' or 'html', entry file exists in the same directory
- [ ] #3 #3 GET /cwd/{path} route added; serves files from project CWD with correct Content-Type and Cache-Control: no-store
- [ ] #4 #4 /cwd/ route enforces path containment: no .. traversal, no symlinks escaping CWD, directories return 404
- [ ] #5 #5 Markdown panes served via /cwd/ with base URL set so relative image paths resolve correctly
- [ ] #6 #6 HTML panes rendered as <iframe sandbox="allow-scripts allow-forms"> in the web UI
- [ ] #7 #7 generate_pane_csp() produces correct CSP string for: empty allowlist, single host, multiple hosts, wildcard host, extra_connect_src entries
- [ ] #8 #8 Web server attaches generated CSP header to all files served under an html-pane path
- [ ] #9 #9 connect-src is 'none' when agent network allowlist is empty and extra_connect_src is absent
- [ ] #10 #10 Session bundle snapshots pane.yaml files alongside entry files and assets; replay server applies same CSP logic
- [ ] #11 #11 TASK-37.11's flat .agents/panes/*.md file convention is replaced — no code paths watch for raw .md files at the panes root
- [ ] #12 #12 cargo test --workspace passes
<!-- AC:END -->
