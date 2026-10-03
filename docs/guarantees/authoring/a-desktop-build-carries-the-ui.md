# A Desktop Build Carries The UI

Given `just local-install` or a Tauri desktop build, when the frontend is built
and the desktop binary is compiled, then the binary embeds that frontend's
index, scripts, and styles. Rebuilding the frontend invalidates Cargo's cached
desktop build even when no Rust source changes. A release build without a built
frontend fails before it can replace the installed app.

Debug app bundles embed their assets too, so moving a bundle away from its
source checkout does not take its interface away.

---

Last LLM verification:

- Date: 2026-10-03
- Reviewer: Codex
- Result: partially verified
- Evidence: `apps/desktop/src-tauri/build.rs` tracks `../../web/dist` and
  refuses a release without `index.html`; `Cargo.toml` enables rust-embed's
  `debug-embed` feature. `tauri.conf.json` builds the frontend before Cargo;
  `justfile::local-install` installs only after Tauri succeeds.
- Test coverage: `apps/desktop/src-tauri/tests/serves_one_origin.rs::the_page_serves_its_built_scripts_and_styles`
  requests the shell and its script/style assets over real HTTP. Run in release
  mode to exercise the same embedding as the installed app.
- Caveat: build invalidation and the missing-frontend refusal require build
  checks; the HTTP test alone does not exercise Cargo's cache decisions.
