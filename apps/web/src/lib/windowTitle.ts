// What the window calls itself, as one pure precedence rule:
//
//   1. the custom override from Settings (GET/PUT /api/settings/ui) — used
//      verbatim, because a person who typed a title meant that exact title;
//   2. else the open project folder's name (GET /api/files' `root`);
//   3. else the focused file's name;
//   4. else just the app.
//
// This covers the in-window half (document.title). The native window title
// the desktop shell sets at launch follows the same rule — see
// apps/desktop/src-tauri/src/lib.rs — but does not update live: there is no
// IPC from this page to the shell, deliberately.

export interface TitleParts {
  /** The custom override, or null/absent when none is set. */
  custom?: string | null;
  /** The open folder, as the server names it (may be a path). */
  folder?: string | null;
  /** The focused file's display name ("Untitled" counts). */
  file?: string | null;
}

const APP = "Hickory Docs";

/** A folder value may arrive as a path; the title wants its last segment. */
function folderName(folder: string): string {
  const name = folder.replace(/[/\\]+$/, "").split(/[/\\]/).pop() ?? folder;
  return name === "" ? folder : name;
}

export function windowTitle({ custom, folder, file }: TitleParts): string {
  const override = custom?.trim();
  if (override) return override;
  const project = folder?.trim();
  if (project) return `${folderName(project)} — ${APP}`;
  const name = file?.trim();
  if (name) return `${name} — ${APP}`;
  return APP;
}
