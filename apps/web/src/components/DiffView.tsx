// A diff, as git prints it, with the lines told apart.
//
// Unified diff and nothing cleverer: side-by-side needs a wide pane and a
// merge view needs three versions, and neither is what "what did I change in
// this file" asks for. The hunk headers, the additions and the removals are
// coloured; the file headers are dimmed rather than hidden, because `---`
// and `+++` are how a rename or a mode change announces itself.

export type DiffLineKind = "meta" | "hunk" | "add" | "del" | "ctx";

/** Which kind of line this is, by its first character — the way every diff
 * reader since 1990 has decided it. */
export function diffLineKind(line: string): DiffLineKind {
  if (line.startsWith("@@")) return "hunk";
  if (
    line.startsWith("diff ") ||
    line.startsWith("index ") ||
    line.startsWith("--- ") ||
    line.startsWith("+++ ") ||
    line.startsWith("new file") ||
    line.startsWith("deleted file") ||
    line.startsWith("similarity") ||
    line.startsWith("rename ") ||
    line.startsWith("old mode") ||
    line.startsWith("new mode") ||
    line.startsWith("Binary files")
  ) {
    return "meta";
  }
  if (line.startsWith("+")) return "add";
  if (line.startsWith("-")) return "del";
  return "ctx";
}

export function DiffView({
  path,
  diff,
  binary,
  staged,
}: {
  path: string;
  diff: string;
  binary: boolean;
  staged: boolean;
}) {
  const lines = diff.replace(/\n$/, "").split("\n");
  return (
    <section className="diff-view" aria-label={`Diff of ${path}`}>
      <header className="diff-view__head">
        <span className="mono diff-view__path">{path}</span>
        <span className="muted">{staged ? "staged" : "unstaged"}</span>
      </header>
      {binary ? (
        <p className="muted diff-view__binary">A binary file; there are no lines to show.</p>
      ) : diff.trim() === "" ? (
        <p className="muted diff-view__binary">No difference.</p>
      ) : (
        <pre className="diff-view__body">
          {lines.map((line, index) => (
            <span key={index} className={`diff-line diff-line--${diffLineKind(line)}`}>
              {line}
              {"\n"}
            </span>
          ))}
        </pre>
      )}
    </section>
  );
}
