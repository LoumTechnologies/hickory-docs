# Develop Hickory locally

Run `just dev` from this checkout. It opens the Simple native editor on macOS
and the Simple browser elsewhere. Both use an isolated seed workspace under
`.dev/project`. Set `HICKORY_PROJECT_DIR` to use an existing folder and
`HICKORY_OPEN_FILE` to open a specific workspace-relative file.

Use `just dev browser`, `just dev apple`, `just dev windows` or
`just dev linux` to choose a desktop frontend explicitly. The supervisor
rebuilds and relaunches that frontend when HEAD changes. Use `just dev-stop`
to stop it, or `just dev-clean` to also remove this checkout's development
scratch directory. Save or move any scratch edits you intend to keep first.

`just simple-dev apple` shows Cargo's build or lock status, then reports native
startup stages through signing and opening the app. On repeat launches, SwiftPM
can reuse the preview resources when their generated contents are identical.
Cargo can also reuse the Rust engine across edits outside its dependencies while
the checkout commit and clean or dirty state stay the same.

### Refresh Simple Dev Apple when Codex finishes

On macOS, run `just codex-simple-dev-install` from your main checkout, then
review and trust the new Stop hook in Codex Settings → Hooks or with `/hooks`
in the CLI. The hook queues the checkout used by the completed turn, including
linked worktrees and uncommitted changes. The detached worker builds and opens
Simple Dev Apple in the background. A new request during a build waits for the
app to start, then uses the latest queued inputs. When those inputs match the
running app, the agent keeps it open. A later turn can retry a failed build.

Close a manually started dev session with `just dev-stop` before using the
automatic preview in that checkout. The automatic preview also responds to
`just dev-stop`. See build and launch output in
`~/Library/Application Support/Hickory Docs/Codex Simple Dev/preview.log`.

Run `just dev-site` to work on the public site and its embedded demonstrations.
The site has separate development and build entry points.

The [Simple frontend guide](simple-frontends.md) lists prerequisites for each
platform, including the iOS simulator and Android. On macOS, use
`just local-install` to install a signed local build in Applications.
