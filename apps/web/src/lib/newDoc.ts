import startupIntroduction from "../content/startup.md?raw";

// New notes remain ordinary Markdown documents until explicitly saved.

/** Edit content/startup.md in any Markdown editor; bundled as the startup buffer. */
export const STARTUP_INTRODUCTION = startupIntroduction;
export const UNTITLED_RECOVERY_KEY = "untitled:recovery";

/**
 * The path the untitled document is created at: `untitled.md`, then
 * `untitled-2.md` and so on past whatever the folder already holds.
 *
 * Taken-ness is judged on the final path segment so a nested
 * `notes/untitled.md` still pushes the name along — two files a person can
 * only tell apart by directory is a worse outcome than skipping a number.
 */
export function untitledPath(existing: string[]): string {
  const taken = new Set(
    existing.map((p) => p.replace(/\\/g, "/").split("/").pop() ?? p),
  );
  for (let n = 1; ; n++) {
    const candidate = n === 1 ? "untitled.md" : `untitled-${n}.md`;
    if (!taken.has(candidate)) return candidate;
  }
}

/** A native Save dialog's suggested name: the first Markdown heading, when
 * there is one, otherwise the familiar untitled sequence. */
export function untitledSaveName(source: string, existing: string[]): string {
  const heading = source.match(/^#{1,6}[ \t]+(.+?)[ \t]*#*[ \t]*$/m)?.[1]?.trim();
  if (!heading) return untitledPath(existing);
  const stem = heading
    .replace(/[<>:"/\\|?*\u0000-\u001F]/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/[. ]+$/g, "");
  return stem ? `${stem}.md` : untitledPath(existing);
}

/**
 * The draft-store key for an untitled buffer.
 *
 * It is not a path in the project. The workspace store uses `path` as a
 * stable, opaque key, and putting this under the project would turn an
 * unsaved document into a file git can discover.
 */
export function untitledDraftKey(tabId: string): string {
  return `untitled:${tabId}`;
}

/**
 * The created document: the typed prose, and nothing else.
 *
 * A new note is **bare** (`docs/specs/freeform/bare-documents.md`): the root
 * element is optional, the prefix defaults to `hick`, and `weave=` defaults to
 * the document's own name — so the envelope this used to add bought nothing
 * and cost the thing the spec exists for. Its motivating case *is* this one:
 * "the file is a thing you open on a phone in a meeting, and the first
 * impression of the format is the first line of the file." Typing `# Standup`
 * and getting three lines of XML around it is the impression that spec was
 * adopted to remove.
 *
 * A document that needs the wrapper still opts into it by typing it — that is
 * rule 1 working in the only direction that matters. Prose about hick itself,
 * which rebinds the prefix to `h:`, is the one kind that must.
 */
export function wrapUntitled(prose: string): string {
  return prose.endsWith("\n") ? prose : `${prose}\n`;
}
