import { describe, expect, it } from "vitest";

// Protects docs/guarantees/authoring/a-fence-becomes-a-cell-that-runs.md
import { convertFence, fenceLanguage, heredocDelimiter } from "./fenceToExec";

describe("the fence's language", () => {
  it("is the first word of the info string, case-folded", () => {
    expect(fenceLanguage("Python")).toBe("python");
    expect(fenceLanguage("  bash  ")).toBe("bash");
    expect(fenceLanguage("python showLineNumbers")).toBe("python");
    expect(fenceLanguage("")).toBe("");
  });
});

describe("shell fences", () => {
  it("transfer verbatim, because a shell fence already is the command", () => {
    const out = convertFence({ info: "bash", body: "make build\nmake test" });
    expect(out.body).toBe("make build\nmake test");
    expect(out.how).toBe("verbatim");
    expect(out.note).toBeNull();
  });

  it("treat an untagged fence as shell", () => {
    expect(convertFence({ info: "", body: "ls -la" }).how).toBe("verbatim");
  });

  it("drop the fence's own trailing blank line, not the program's content", () => {
    expect(convertFence({ info: "sh", body: "echo hi\n\n" }).body).toBe("echo hi");
    expect(convertFence({ info: "sh", body: "echo a\n\necho b" }).body).toBe("echo a\n\necho b");
  });
});

describe("interpreted fences", () => {
  it("feed the program to its interpreter on stdin", () => {
    const out = convertFence({ info: "python", body: 'print("hi")' });
    expect(out.body).toBe("python3 - <<'EOF'\nprint(\"hi\")\nEOF");
    expect(out.how).toBe("heredoc");
    expect(out.note).toBeNull();
  });

  it("quotes the delimiter so the shell cannot expand the program first", () => {
    // Unquoted, `$name` and backticks would be substituted before the
    // interpreter ever saw them — every language that uses $ would break.
    const out = convertFence({ info: "ruby", body: 'puts "#{x} `y` $z"' });
    expect(out.body).toContain("<<'EOF'");
    expect(out.body).toContain('puts "#{x} `y` $z"');
  });

  it("knows the common aliases", () => {
    expect(convertFence({ info: "py", body: "x" }).body).toContain("python3 -");
    expect(convertFence({ info: "js", body: "x" }).body).toContain("node -");
    expect(convertFence({ info: "JavaScript", body: "x" }).body).toContain("node -");
  });
});

describe("the heredoc delimiter", () => {
  it("is EOF when the program does not contain one", () => {
    expect(heredocDelimiter("print(1)")).toBe("EOF");
  });

  it("steps aside for a program with a bare EOF line", () => {
    // Otherwise the heredoc ends early and the shell runs the rest of the
    // program as commands — a silently truncated cell.
    expect(heredocDelimiter("a\nEOF\nb")).toBe("EOF2");
    expect(heredocDelimiter("a\nEOF\nEOF2\nb")).toBe("EOF3");
    expect(heredocDelimiter("a\n  EOF  \nb")).toBe("EOF2");
  });

  it("is used by the conversion, not just available to it", () => {
    const out = convertFence({ info: "python", body: "s = '''\nEOF\n'''" });
    expect(out.body.startsWith("python3 - <<'EOF2'")).toBe(true);
    expect(out.body.endsWith("\nEOF2")).toBe(true);
  });
});

describe("a language we have no interpreter for", () => {
  it("is carried across unchanged and says so, rather than guessing", () => {
    const out = convertFence({ info: "sql", body: "SELECT 1;" });
    expect(out.body).toBe("SELECT 1;");
    expect(out.how).toBe("unknown");
    expect(out.note).toContain("sql");
    expect(out.note).toContain("hick:needs");
  });
});
