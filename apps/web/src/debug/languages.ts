// Which languages this app can debug, in one place.
//
// The editor needs it while typing — to decide whether a line can hold a
// breakpoint at all — and the file chip needs it to decide whether to offer
// a Debug button. Asking the server would make both flicker: the answer
// arrives a round trip after the text does.
//
// Kept in step with `hick_dap::discovery::candidates` by hand. Being wrong
// here costs a refusal that names the language; being absent costs the
// feature.

export const DEBUGGABLE_LANGUAGES = new Set([
  "python",
  "typescript",
  "javascript",
  "go",
  "rust",
  // C#, via netcoredbg. `hick_dap::discovery` has answered for csharp since
  // the Build step landed; this list did not, so a `.cs` file block offered no
  // ghost dot and the breakpoint gutter rendered no cells at all — which is
  // also why a click in the gutter did nothing rather than refusing in words.
  "csharp",
]);

export function isDebuggable(language: string | null | undefined): boolean {
  return !!language && DEBUGGABLE_LANGUAGES.has(language);
}
