// The one place `import.meta.env` is read.
//
// Vite inlines these at BUILD time, so a value here is baked into the bundle
// and is public by definition — never put a secret behind a `VITE_` name. The
// module validates once at import and exports a frozen object, so a component
// can never silently receive `undefined` from a typo'd variable name
// (.instructions/config-and-environments.md).
//
// One source tree, two builds:
//
// - **the static site** (`site.html`): the marketing page and its in-browser
//   demos. No API client, no accounts, nothing to buy. Deployed to
//   hickorydocs.com.
// - **the desktop app** (`index.html`): the editor, bundled by Tauri and
//   talking only to the local server in its own process.
//
// There is no third. See `docs/specs/freeform/local-only.md`.

/** PostHog project write keys are `phc_…`. */
const PROJECT_KEY_PREFIX = "phc_";

function readPosthogKey(): string | null {
  const raw = (import.meta.env.VITE_POSTHOG_KEY ?? "").trim();
  if (!raw) return null;
  if (!raw.startsWith(PROJECT_KEY_PREFIX)) {
    // The mistake this catches is easy to make and expensive: PostHog's
    // *personal* API key (`phx_…`) can create and destroy projects, and a
    // build that inlined one would publish it to every visitor.
    //
    // Refuse the key, but do NOT throw. This module is imported by the app's
    // entry point, so throwing here would white-screen the marketing site over
    // a misconfigured analytics variable — trading a data problem for an
    // outage. The build-time gate that *does* fail loudly is in `just site`;
    // this is the last line of defence, and its job is to ship a working page
    // with no analytics rather than a broken one.
    console.error(
      `VITE_POSTHOG_KEY must be a PostHog project write key (${PROJECT_KEY_PREFIX}…); ` +
        `got "${raw.slice(0, 4)}…". Analytics is disabled for this build. A personal ` +
        `API key (phx_…) must never reach the browser — see docs/operators/analytics.md.`,
    );
    return null;
  }
  return raw;
}

export interface WebConfig {
  /** Demo profile: no network at all (`npm run dev:mock`). */
  readonly mock: boolean;
  /** PostHog project write key; `null` disables browser-side capture. */
  readonly posthogKey: string | null;
  readonly posthogHost: string;
  /** The one-line installer shown as the primary call to action. */
  readonly installCommand: string;
}

export const config: WebConfig = Object.freeze({
  mock: import.meta.env.VITE_MOCK === "1",
  posthogKey: readPosthogKey(),
  posthogHost: (import.meta.env.VITE_POSTHOG_HOST ?? "https://us.i.posthog.com").replace(
    /\/+$/,
    "",
  ),
  installCommand:
    import.meta.env.VITE_INSTALL_COMMAND ??
    "curl -fsSL https://hickorydocs.com/install.sh | sh",
});
