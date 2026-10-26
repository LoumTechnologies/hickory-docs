<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Fence Becomes A Cell That Runs

For: Authors editing executable documents and the files those documents produce.

Given a `.md` document whose prose contains a closed fenced code block, when
its rail icon is clicked, then a panel shows the exact `<hick:exec>` that
would replace it — the container it will run in, and a body derived from the
fence's language — and confirming replaces exactly the fence, leaving every
byte around it untouched.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-fence-becomes-a-cell-that-runs.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-a04a2c7645d2"></a>
## `apps/web/src/components/FenceConvert.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Fence Becomes A Cell That Runs](contract.md).

<a id="f-a04a2c7645d2-expression_statement"></a>
### `expression_statement`

@vitest-environment jsdom
Protects docs/guarantees/authoring/a-fence-becomes-a-cell-that-runs.md

<storycode:copy id="f-a04a2c7645d2-expression_statement">
// @vitest-environment jsdom
//
// Protects docs/guarantees/authoring/a-fence-becomes-a-cell-that-runs.md

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";

import { FenceConvert } from "./FenceConvert";

afterEach(cleanup);

</storycode:copy>
<a id="f-a04a2c7645d2-panel"></a>
### `panel`

<storycode:copy id="f-a04a2c7645d2-panel">
function panel(props: Partial<React.ComponentProps<typeof FenceConvert>> = {}) {
  const onConvert = vi.fn();
  const onCancel = vi.fn();
  render(
    <FenceConvert
      info="python"
      body='print("hi")'
      containers={["py"]}
      onConvert={onConvert}
      onCancel={onCancel}
      {...props}
    />,
  );
  return { onConvert, onCancel };
}

</storycode:copy>
<a id="f-a04a2c7645d2-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-a04a2c7645d2-expression_statement-2">
describe("converting a fence", () => {
  it("shows the exec cell it would write before writing it", () => {
    panel();
    const preview = document.querySelector(".insert-menu__preview-text")!.textContent!;
    expect(preview).toContain('<hick:exec container="py">');
    expect(preview).toContain("python3 - <<'EOF'");
    expect(preview).toContain('print("hi")');
    expect(preview).toContain("</hick:exec>");
  });

  it("hands back exactly the previewed text", () => {
    const { onConvert } = panel();
    const preview = document.querySelector(".insert-menu__preview-text")!.textContent!;
    fireEvent.click(screen.getByRole("button", { name: "Convert" }));
    expect(onConvert).toHaveBeenCalledWith(preview);
  });

  it("offers the containers the document declares, and follows the choice", () => {
    panel({ containers: ["py", "node"] });
    fireEvent.change(screen.getByLabelText(/Container/), { target: { value: "node" } });
    expect(document.querySelector(".insert-menu__preview-text")!.textContent).toContain(
      '<hick:exec container="node">',
    );
  });

  it("asks for a container name when the document declares none", () => {
    const { onConvert } = panel({ containers: [] });
    const input = screen.getByLabelText(/Container/) as HTMLInputElement;
    expect(input.tagName).toBe("INPUT");
    // An exec with nowhere to run is not a conversion worth doing.
    expect((screen.getByRole("button", { name: "Convert" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    fireEvent.change(input, { target: { value: "py" } });
    fireEvent.click(screen.getByRole("button", { name: "Convert" }));
    expect(onConvert).toHaveBeenCalled();
  });

  it("says how a shell fence was treated", () => {
    panel({ info: "bash", body: "make build" });
    expect(screen.getByText(/already shell/)).toBeTruthy();
    expect(document.querySelector(".insert-menu__preview-text")!.textContent).toContain(
      "make build",
    );
  });

  it("warns rather than guesses for a language it has no interpreter for", () => {
    panel({ info: "sql", body: "SELECT 1;" });
    expect(screen.getByRole("note").textContent).toContain("sql");
  });

  it("cancels without converting", () => {
    const { onConvert, onCancel } = panel();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancel).toHaveBeenCalled();
    expect(onConvert).not.toHaveBeenCalled();
  });
});

</storycode:copy>
<a id="f-a04a2c7645d2-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-a04a2c7645d2-expression_statement-3">
it("shares the conversion draft and preview with agents and retires replaced-source handles", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const convert = vi.fn(() => true), cancel = vi.fn();
  const panel = (body: string) => <UiActionContext.Provider value={registry}><FenceConvert info="python" body={body} containers={["py", "other"]} onConvert={convert} onCancel={cancel} /></UiActionContext.Provider>;
  const view = render(panel("print(1)"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  const read = await run("Read fence conversion preview bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(Uint8Array.from(read.outcome.value.bytes))).toContain("print(1)");
  await run("Set fence conversion container", { container: "other" });
  expect((screen.getByLabelText(/Container/) as HTMLSelectElement).value).toBe("other");
  const old = actions().find(action => action.label === "Convert fence to executable cell")!.handle;
  view.rerender(panel("print(2)"));
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: old, arguments: {}, request_id: "stale" })).not.toMatchObject({ status: "result" });
  expect(convert).not.toHaveBeenCalled();
  const preview = document.querySelector(".insert-menu__preview-text")!.textContent!;
  expect(await run("Convert fence to executable cell")).toMatchObject({ status: "result", outcome: { value: { source_changed: true, saved: false, executed: false } } });
  expect(convert).toHaveBeenCalledExactlyOnceWith(preview);
  await run("Cancel fence conversion");
  expect(cancel).toHaveBeenCalledTimes(1);
});
</storycode:copy>
<a id="f-a04a2c7645d2-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-a04a2c7645d2-expression_statement-4">
it("preserves the conversion draft when the owner refuses changed source", () => {
  panel({ onConvert: () => false });
  fireEvent.click(screen.getByRole("button", { name: "Convert" }));
  expect(screen.getByRole("alert").textContent).toContain("source or editor authority changed");
  expect((screen.getByLabelText(/Container/) as HTMLSelectElement).value).toBe("py");
});
</storycode:copy>

### Assemble `apps/web/src/components/FenceConvert.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/FenceConvert.test.tsx"><storycode:paste select="#f-a04a2c7645d2-expression_statement"/><storycode:paste select="#f-a04a2c7645d2-panel"/><storycode:paste select="#f-a04a2c7645d2-expression_statement-2"/><storycode:paste select="#f-a04a2c7645d2-expression_statement-3"/><storycode:paste select="#f-a04a2c7645d2-expression_statement-4"/></storycode:file>

<a id="output-53218b160f5a"></a>
## `apps/web/src/components/FenceConvert.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Fence Becomes A Cell That Runs](contract.md).

<a id="f-53218b160f5a-FenceConvertProps"></a>
### `FenceConvertProps`

"Make this fence a cell" — the panel a fence's rail icon opens.
It shows the exec cell it would write before writing it, for the same
reason the Insert menu does: this is a text format a person owns, and a
conversion you cannot see is one you cannot check. The container is the
only choice worth asking about, because it is the only thing the fence
cannot tell us — everything else is derived from the language tag.

<storycode:copy id="f-53218b160f5a-FenceConvertProps">
// "Make this fence a cell" — the panel a fence's rail icon opens.
//
// It shows the exec cell it would write before writing it, for the same
// reason the Insert menu does: this is a text format a person owns, and a
// conversion you cannot see is one you cannot check. The container is the
// only choice worth asking about, because it is the only thing the fence
// cannot tell us — everything else is derived from the language tag.

import { useId, useMemo, useState } from "react";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";
import { convertFence } from "../lib/fenceToExec";
import { elementById, initialValues, renderElement } from "../lib/insertCatalog";

export interface FenceConvertProps {
  /** The fence's info string and body. */
  info: string;
  body: string;
  /** Containers this document already declares, in document order. */
  containers: string[];
  /** Replace the fence with this text. */
  onConvert: (text: string) => boolean | void;
  onCancel: () => void;
}

</storycode:copy>
<a id="f-53218b160f5a-FenceConvert"></a>
### `FenceConvert`

<storycode:copy id="f-53218b160f5a-FenceConvert">
export function FenceConvert({
  info,
  body,
  containers,
  onConvert,
  onCancel,
}: FenceConvertProps) {
  // The first declared container is nearly always the right answer, and a
  // document with none gets a name to type rather than a silent omission —
  // an exec with no container and no image has nowhere to run.
  const [container, setContainer] = useState(containers[0] ?? "");
  const conversion = useMemo(() => convertFence({ info, body }), [info, body]);

  const exec = elementById("exec");
  const preview = useMemo(() => {
    if (!exec) return "";
    return renderElement(exec, { ...initialValues(exec), container }, conversion.body);
  }, [exec, container, conversion.body]);

  const parent = `fence-convert:${useId()}`;
  const [error, setError] = useState<string | null>(null);
  const callbackRevision = useMemo(() => crypto.randomUUID(), [onConvert, onCancel]);
  const revision = useUiRevision(() => [info, body, containers, container, preview, callbackRevision]);
  const changeContainer = (text: string): UiOutcome => {
    if (text.length > 256 || (containers.length > 0 && !containers.includes(text))) return { status: "unavailable", reason: "Choose a currently declared container" };
    setContainer(text); setError(null);
    return { status: "completed", value: { draft_updated: true } };
  };
  const convert = (): UiOutcome => {
    if (!container.trim() || (containers.length > 0 && !containers.includes(container))) return { status: "unavailable", reason: "Choose a current container before converting" };
    try {
      const changed = onConvert(preview);
      if (changed === false) {
        const reason = "The fence source or editor authority changed. Reopen this fence and inspect the current preview.";
        setError(reason); return { status: "requires_input", request: `${parent}:source`, reason };
      }
      return { status: "completed", value: { edit_requested: true, source_changed: changed === true, saved: false, executed: false } };
    } catch (cause) { setError(String(cause)); return { status: "unavailable", reason: String(cause) }; }
  };
  const cancel = (): UiOutcome => { onCancel(); return { status: "completed", value: { dismissal_requested: true, converted: false } }; };
  useUiCommands([
    { key: `${parent}:container`, command: { parent, subject: parent, label: "Set fence conversion container", parameters: { container: { type: "string", maxLength: 256, ...(containers.length ? { choices: containers } : {}) } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true }), execute: async args => changeContainer(args.container as string) } },
    { key: `${parent}:convert`, command: { parent, subject: parent, label: "Convert fence to executable cell", parameters: {}, prerequisites: ["Preview is a source edit, not execution or saving"],
      read: () => ({ revision: revision(), enabled: !!container.trim() && (!containers.length || containers.includes(container)) }), execute: async () => convert() } },
    { key: `${parent}:cancel`, command: { parent, subject: parent, label: "Cancel fence conversion", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true }), execute: async () => cancel() } },
  ]);
  useUiReading("fence conversion preview", { info, body, container, containers, conversion, preview, error, scope: "Rendered preview; conversion neither saves nor executes the cell" });

  return (
    <div className="fence-convert">
      <header className="fence-convert__header">
        <h3 className="fence-convert__title">Make this a cell</h3>
        <span className="fence-convert__lang mono">
          {conversion.language || "no language tag"}
        </span>
      </header>

      <p className="fence-convert__how">
        {conversion.how === "heredoc"
          ? "The program is fed to its interpreter on standard input, so the cell runs exactly this code."
          : conversion.how === "verbatim"
            ? "The fence is already shell, so its commands transfer unchanged."
            : "Carried across unchanged — see below."}
      </p>

      <label className="fence-convert__label" htmlFor="fence-convert-container">
        Container
        <span className="mono insert-menu__attr">container</span>
      </label>
      {containers.length > 0 ? (
        <select
          id="fence-convert-container"
          className="insert-menu__input"
          value={container}
          onChange={(event) => changeContainer(event.target.value)}
        >
          {containers.map((name) => (
            <option key={name} value={name}>
              {name}
            </option>
          ))}
        </select>
      ) : (
        <input
          id="fence-convert-container"
          className="insert-menu__input"
          type="text"
          value={container}
          spellCheck={false}
          placeholder="py"
          maxLength={256}
          onChange={(event) => changeContainer(event.target.value)}
        />
      )}
      {containers.length === 0 && (
        <p className="insert-menu__hint">
          This document declares no container yet. Name one here, then add a{" "}
          <span className="mono">hick:container</span> for it — the Insert menu has one.
        </p>
      )}

      {conversion.note && (
        <p className="fence-convert__note" role="note">
          {conversion.note}
        </p>
      )}

      <div className="insert-menu__preview">
        <h4 className="insert-menu__preview-title">What replaces the fence</h4>
        <pre className="insert-menu__preview-text mono">{preview}</pre>
      </div>

      {error && <p role="alert">{error}</p>}
      <div className="insert-menu__actions">
        <button type="button" className="btn" onClick={() => cancel()}>
          Cancel
        </button>
        <button
          type="button"
          className="btn btn-primary"
          disabled={!container.trim() || (containers.length > 0 && !containers.includes(container))}
          data-tip={container.trim() ? undefined : "Name the container it runs in"}
          onClick={() => convert()}
        >
          Convert
        </button>
      </div>
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/FenceConvert.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/FenceConvert.tsx"><storycode:paste select="#f-53218b160f5a-FenceConvertProps"/><storycode:paste select="#f-53218b160f5a-FenceConvert"/></storycode:file>

<a id="output-9a528e9ffdee"></a>
## `apps/web/src/lib/fenceToExec.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Fence Becomes A Cell That Runs](contract.md).

<a id="f-9a528e9ffdee-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9a528e9ffdee-expression_statement">
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

</storycode:copy>
<a id="f-9a528e9ffdee-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-9a528e9ffdee-expression_statement-2">
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

</storycode:copy>
<a id="f-9a528e9ffdee-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-9a528e9ffdee-expression_statement-3">
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

</storycode:copy>
<a id="f-9a528e9ffdee-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-9a528e9ffdee-expression_statement-4">
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

</storycode:copy>
<a id="f-9a528e9ffdee-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-9a528e9ffdee-expression_statement-5">
describe("a language we have no interpreter for", () => {
  it("is carried across unchanged and says so, rather than guessing", () => {
    const out = convertFence({ info: "sql", body: "SELECT 1;" });
    expect(out.body).toBe("SELECT 1;");
    expect(out.how).toBe("unknown");
    expect(out.note).toContain("sql");
    expect(out.note).toContain("hick:needs");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/fenceToExec.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/fenceToExec.test.ts"><storycode:paste select="#f-9a528e9ffdee-expression_statement"/><storycode:paste select="#f-9a528e9ffdee-expression_statement-2"/><storycode:paste select="#f-9a528e9ffdee-expression_statement-3"/><storycode:paste select="#f-9a528e9ffdee-expression_statement-4"/><storycode:paste select="#f-9a528e9ffdee-expression_statement-5"/></storycode:file>

<a id="output-4dadfd5775b8"></a>
## `apps/web/src/lib/fenceToExec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Fence Becomes A Cell That Runs](contract.md).

<a id="f-4dadfd5775b8-SHELL_LANGUAGES"></a>
### `SHELL_LANGUAGES`

Turning a fenced code block in a document's prose into a cell that runs.
A markdown fence is a claim about a command that nobody checked. That is
the exact gap this product exists to close, and the distance between the
two is small enough that it should be one click: the text is already
written, it just is not wired to anything.
The conversion is not a rename. `<hick:exec>` runs a SHELL, so a fence
tagged `bash` transfers verbatim while one tagged `python` has to become a
command that feeds the program to an interpreter — the `python3 - <<'EOF'`
idiom the examples already use. Anything we have no interpreter for is
carried across untouched, with a note saying so: a wrong guess that looks
right is worse than an honest hand-off.
Pure text in, pure text out. The panel (components/FenceConvert.tsx) shows
what this produces before anything is written.
Fence languages that already ARE shell. Their body is a command list and
transfers unchanged. The empty string is here because an untagged fence in
a document about running things is, overwhelmingly, a shell transcript.

<storycode:copy id="f-4dadfd5775b8-SHELL_LANGUAGES">
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

</storycode:copy>
<a id="f-4dadfd5775b8-STDIN_INTERPRETERS"></a>
### `STDIN_INTERPRETERS`

Languages whose interpreter reads a program on standard input, and the
command that does it. Only interpreters that genuinely accept `-` (or its
equivalent) are listed — an entry that silently opened a REPL instead
would hang the cell until its timeout.

<storycode:copy id="f-4dadfd5775b8-STDIN_INTERPRETERS">
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

</storycode:copy>
<a id="f-4dadfd5775b8-FenceLike"></a>
### `FenceLike`

<storycode:copy id="f-4dadfd5775b8-FenceLike">
export interface FenceLike {
  /** The info string after the opening fence. */
  info: string;
  /** The text between the fence lines. */
  body: string;
}

</storycode:copy>
<a id="f-4dadfd5775b8-FenceConversion"></a>
### `FenceConversion`

<storycode:copy id="f-4dadfd5775b8-FenceConversion">
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

</storycode:copy>
<a id="f-4dadfd5775b8-fenceLanguage"></a>
### `fenceLanguage`

The language of a fence: the first word of its info string, lower-cased,
so an info string carrying extra attributes still resolves to python.

<storycode:copy id="f-4dadfd5775b8-fenceLanguage">
/** The language of a fence: the first word of its info string, lower-cased,
 * so an info string carrying extra attributes still resolves to python. */
export function fenceLanguage(info: string): string {
  return info.trim().split(/\s+/)[0]?.toLowerCase() ?? "";
}

</storycode:copy>
<a id="f-4dadfd5775b8-heredocDelimiter"></a>
### `heredocDelimiter`

A heredoc delimiter this body cannot contain.
A body with a bare `EOF` line would end the heredoc early and hand the
shell the rest of the program as commands — a silently truncated cell,
which is the worst way for this to fail.

<storycode:copy id="f-4dadfd5775b8-heredocDelimiter">
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

</storycode:copy>
<a id="f-4dadfd5775b8-convertFence"></a>
### `convertFence`

What the exec cell's body should be for this fence.
The quoted heredoc delimiter (`<<'EOF'`, not `<<EOF`) matters: unquoted,
the shell expands `$name` and backticks inside the program before the
interpreter ever sees it, which corrupts every language that uses `$`.

<storycode:copy id="f-4dadfd5775b8-convertFence">
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
</storycode:copy>

### Assemble `apps/web/src/lib/fenceToExec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/fenceToExec.ts"><storycode:paste select="#f-4dadfd5775b8-SHELL_LANGUAGES"/><storycode:paste select="#f-4dadfd5775b8-STDIN_INTERPRETERS"/><storycode:paste select="#f-4dadfd5775b8-FenceLike"/><storycode:paste select="#f-4dadfd5775b8-FenceConversion"/><storycode:paste select="#f-4dadfd5775b8-fenceLanguage"/><storycode:paste select="#f-4dadfd5775b8-heredocDelimiter"/><storycode:paste select="#f-4dadfd5775b8-convertFence"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/FenceConvert.tsx](contract.md#output-53218b160f5a)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/hickDoc.ts](../../../mechanisms/apps/web/src/editor/hickDoc.ts.md#output-c2f7bc821e36)
- [apps/web/src/lib/fenceToExec.ts](contract.md#output-4dadfd5775b8)

### Tests and fixtures

- [apps/web/src/components/FenceConvert.test.tsx](contract.md#output-a04a2c7645d2)
- [apps/web/src/editor/DocumentEditor.test.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.test.tsx.md#output-71bbcfe029bb)
- [apps/web/src/editor/cards.test.ts](../../../mechanisms/apps/web/src/editor/cards.ts.md#output-66013e5cdf76)
- [apps/web/src/lib/fenceToExec.test.ts](contract.md#output-9a528e9ffdee)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
