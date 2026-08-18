// Turning a fenced code block in a document's prose into a cell that runs.
//
// A markdown fence is a claim about a command that nobody checked. That is
// the exact gap this product exists to close, and the distance between the
// two is small enough that it should be one click: the text is already
// written, it just is not wired to anything.
//
// The conversion is not a rename. `<hick:exec>` runs a SHELL, so a fence
// tagged `bash` transfers verbatim while one tagged `python` has to become a
// command that feeds the program to an interpreter — the `python3 - <<'EOF'`
// idiom the examples already use. Anything we have no interpreter for is
// carried across untouched, with a note saying so: a wrong guess that looks
// right is worse than an honest hand-off.
//
// Pure text in, pure text out. The panel (components/FenceConvert.tsx) shows
// what this produces before anything is written.

/**
 * Fence languages that already ARE shell. Their body is a command list and
 * transfers unchanged. The empty string is here because an untagged fence in
 * a document about running things is, overwhelmingly, a shell transcript.
 */
const SHELL_LANGUAGES = new Set([
  "",
  "sh",
  "bash",
  "shell",
  "shell-session",
  "console",
  "terminal",
  "zsh",
]);

/**
 * Languages whose interpreter reads a program on standard input, and the
 * command that does it. Only interpreters that genuinely accept `-` (or its
 * equivalent) are listed — an entry that silently opened a REPL instead
 * would hang the cell until its timeout.
 */
const STDIN_INTERPRETERS: Record<string, string> = {
  python: "python3 -",
  python3: "python3 -",
  py: "python3 -",
  node: "node -",
  javascript: "node -",
  js: "node -",
  ruby: "ruby -",
  rb: "ruby -",
  perl: "perl -",
  lua: "lua -",
  r: "Rscript -",
};

export interface FenceLike {
  /** The info string after the opening fence. */
  info: string;
  /** The text between the fence lines. */
  body: string;
}

export interface FenceConversion {
  /** The exec cell's body — what goes between the tags. */
  body: string;
  /** The fence's language, lower-cased and trimmed; "" when untagged. */
  language: string;
  /** How the body was arrived at, for the panel to show. */
  how: "verbatim" | "heredoc" | "unknown";
  /** Present when the conversion needs the person to finish it. */
  note: string | null;
}

/** The language of a fence: the first word of its info string, lower-cased,
 * so an info string carrying extra attributes still resolves to python. */
export function fenceLanguage(info: string): string {
  return info.trim().split(/\s+/)[0]?.toLowerCase() ?? "";
}

/**
 * A heredoc delimiter this body cannot contain.
 *
 * A body with a bare `EOF` line would end the heredoc early and hand the
 * shell the rest of the program as commands — a silently truncated cell,
 * which is the worst way for this to fail.
 */
export function heredocDelimiter(body: string): string {
  const lines = new Set(body.split("\n").map((l) => l.trim()));
  if (!lines.has("EOF")) return "EOF";
  for (let n = 2; ; n++) {
    const candidate = `EOF${n}`;
    if (!lines.has(candidate)) return candidate;
  }
}

/**
 * What the exec cell's body should be for this fence.
 *
 * The quoted heredoc delimiter (`<<'EOF'`, not `<<EOF`) matters: unquoted,
 * the shell expands `$name` and backticks inside the program before the
 * interpreter ever sees it, which corrupts every language that uses `$`.
 */
export function convertFence(fence: FenceLike): FenceConversion {
  const language = fenceLanguage(fence.info);
  // A trailing newline inside the fence is the fence's own formatting, not
  // part of the program.
  const body = fence.body.replace(/\n+$/, "");

  if (SHELL_LANGUAGES.has(language)) {
    return { body, language, how: "verbatim", note: null };
  }

  const interpreter = STDIN_INTERPRETERS[language];
  if (interpreter) {
    const delimiter = heredocDelimiter(body);
    return {
      body: `${interpreter} <<'${delimiter}'\n${body}\n${delimiter}`,
      language,
      how: "heredoc",
      note: null,
    };
  }

  return {
    body,
    language,
    how: "unknown",
    note:
      `No interpreter is known here for \`${language}\`, so the code is carried across ` +
      `as-is — which the shell will try to run as commands. Edit the cell into ` +
      `something that runs it (for example a heredoc into the right interpreter), ` +
      `or declare the tool with <hick:needs bin="…" /> on the container.`,
  };
}
