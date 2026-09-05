/// <reference types="vite/client" />

/**
 * The build-time variables this app understands. Declared so a typo is a type
 * error rather than `undefined` at runtime; read in exactly one place
 * (`src/config.ts`), never in a component.
 */
interface ImportMetaEnv {
  /** `1` → the mock profile: no network at all. */
  readonly VITE_MOCK?: string;
  /** PostHog project write key (`phc_…`). Absent → no browser capture. */
  readonly VITE_POSTHOG_KEY?: string;
  readonly VITE_POSTHOG_HOST?: string;
  /** Override the install command shown as the primary call to action. */
  readonly VITE_INSTALL_COMMAND?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

/**
 * The one Node module the test setup reads: the parser's WebAssembly bytes
 * come off the disk there, because a test has a filesystem and no fetch.
 * Declared by hand rather than through `@types/node`, which this app does
 * not take on — nothing that ships to a browser may reach for Node.
 */
declare module "node:fs" {
  export function readFileSync(path: string): Uint8Array<ArrayBuffer>;
}
