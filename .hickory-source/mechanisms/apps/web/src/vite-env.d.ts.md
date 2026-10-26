<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/vite-env.d.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4ccec8944b4c"></a>
## `apps/web/src/vite-env.d.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-4ccec8944b4c-ImportMetaEnv"></a>
### `ImportMetaEnv`

<reference types="vite/client" />
The build-time variables this app understands. Declared so a typo is a type
error rather than `undefined` at runtime; read in exactly one place
(`src/config.ts`), never in a component.

<storycode:copy id="f-4ccec8944b4c-ImportMetaEnv">
/// <reference types="vite/client" />

/**
 * The build-time variables this app understands. Declared so a typo is a type
 * error rather than `undefined` at runtime; read in exactly one place
 * (`src/config.ts`), never in a component.
 */
interface ImportMetaEnv {
  readonly VITE_SOURCE_REVISION?: string;
  /** `1` → the mock profile: no network at all. */
  readonly VITE_MOCK?: string;
  /** PostHog project write key (`phc_…`). Absent → no browser capture. */
  readonly VITE_POSTHOG_KEY?: string;
  readonly VITE_POSTHOG_HOST?: string;
  /** Override the install command shown as the primary call to action. */
  readonly VITE_INSTALL_COMMAND?: string;
}

</storycode:copy>
<a id="f-4ccec8944b4c-ImportMeta"></a>
### `ImportMeta`

<storycode:copy id="f-4ccec8944b4c-ImportMeta">
interface ImportMeta {
  readonly env: ImportMetaEnv;
}

</storycode:copy>
<a id="f-4ccec8944b4c-ambient_declaration"></a>
### `ambient_declaration`

The one Node module the test setup reads: the parser's WebAssembly bytes
come off the disk there, because a test has a filesystem and no fetch.
Declared by hand rather than through `@types/node`, which this app does
not take on — nothing that ships to a browser may reach for Node.

<storycode:copy id="f-4ccec8944b4c-ambient_declaration">
/**
 * The one Node module the test setup reads: the parser's WebAssembly bytes
 * come off the disk there, because a test has a filesystem and no fetch.
 * Declared by hand rather than through `@types/node`, which this app does
 * not take on — nothing that ships to a browser may reach for Node.
 */
declare module "node:fs" {
  export function readFileSync(path: string): Uint8Array<ArrayBuffer>;
}
</storycode:copy>

### Assemble `apps/web/src/vite-env.d.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/vite-env.d.ts"><storycode:paste select="#f-4ccec8944b4c-ImportMetaEnv"/><storycode:paste select="#f-4ccec8944b4c-ImportMeta"/><storycode:paste select="#f-4ccec8944b4c-ambient_declaration"/></storycode:file>

</storycode:doc>
