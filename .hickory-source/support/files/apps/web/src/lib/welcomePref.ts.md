<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/welcomePref.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `2bfb77252c03679a114eccb4bed97f4d8b7fbfe7a5cff5dbcf729048c9dbd62a`.

<book:entry path="apps/web/src/lib/welcomePref.ts">
// Whether the welcome pane opens with a folder.
//
// Off by default, and dismissible for good — a welcome screen you cannot
// dismiss is one people learn to close angrily. A per-machine preference, like
// the theme and the zoom: it is about how this person likes to start, not
// about the project.

export const WELCOME_KEY = "hickory.showWelcome";

export function loadShowWelcome(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined"
    ? null
    : localStorage,
): boolean {
  // The editable startup document supplies the introduction.
  return storage?.getItem(WELCOME_KEY) === "1";
}

export function saveShowWelcome(
  show: boolean,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined"
    ? null
    : localStorage,
): void {
  storage?.setItem(WELCOME_KEY, show ? "1" : "0");
}
</book:entry>

</book:doc>
