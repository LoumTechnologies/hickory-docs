<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/welcomePref.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5ec9262b669b"></a>
## `apps/web/src/lib/welcomePref.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-5ec9262b669b-WELCOME_KEY"></a>
### `WELCOME_KEY`

Whether the welcome pane opens with a folder.
Off by default, and dismissible for good — a welcome screen you cannot
dismiss is one people learn to close angrily. A per-machine preference, like
the theme and the zoom: it is about how this person likes to start, not
about the project.

<storycode:copy id="f-5ec9262b669b-WELCOME_KEY">
// Whether the welcome pane opens with a folder.
//
// Off by default, and dismissible for good — a welcome screen you cannot
// dismiss is one people learn to close angrily. A per-machine preference, like
// the theme and the zoom: it is about how this person likes to start, not
// about the project.

export const WELCOME_KEY = "hickory.showWelcome";

</storycode:copy>
<a id="f-5ec9262b669b-loadShowWelcome"></a>
### `loadShowWelcome`

<storycode:copy id="f-5ec9262b669b-loadShowWelcome">
export function loadShowWelcome(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined"
    ? null
    : localStorage,
): boolean {
  // The editable startup document supplies the introduction.
  return storage?.getItem(WELCOME_KEY) === "1";
}

</storycode:copy>
<a id="f-5ec9262b669b-saveShowWelcome"></a>
### `saveShowWelcome`

<storycode:copy id="f-5ec9262b669b-saveShowWelcome">
export function saveShowWelcome(
  show: boolean,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined"
    ? null
    : localStorage,
): void {
  storage?.setItem(WELCOME_KEY, show ? "1" : "0");
}
</storycode:copy>

### Assemble `apps/web/src/lib/welcomePref.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/welcomePref.ts"><storycode:paste select="#f-5ec9262b669b-WELCOME_KEY"/><storycode:paste select="#f-5ec9262b669b-loadShowWelcome"/><storycode:paste select="#f-5ec9262b669b-saveShowWelcome"/></storycode:file>

</storycode:doc>
