// What the Insert menu can write, and exactly what bytes it writes.
//
// A hick document is XML you edit by hand, which means the vocabulary is
// learned by finding it — and until now the only place to find it was
// docs/docs/hick-guide.md in another window. This module is that guide,
// turned into something the editor can insert: one entry per element a
// PERSON authors, each naming its attributes, which are required, and what a
// sensible value looks like.
//
// It is deliberately pure text: no CodeMirror, no React, no DOM. The panel
// (components/InsertMenu.tsx) collects values and the workspace dispatches
// the result, but the decision about what a `<hick:copy id="…">` looks like —
// where its newlines go, how a selection becomes its body, where the caret
// lands afterwards — is made here, where it can be tested as data in and
// text out.
//
// The no-escaping invariant reaches this file too: hick has no entity
// escaping, so a value carrying BOTH quote characters cannot be written as an
// attribute at all. That is refused with a message rather than smuggled
// through as `&quot;` — see `validate`.
//
// Machine-written tags are NOT here. `<hick:user>`, `<hick:assistant>`,
// `<hick:action>`, `<hick:observation>`, `<hick:tool-result>` and friends are
// the agent transcript format; a person writing one by hand is describing a
// conversation that never happened.

/** Where an element belongs relative to the caret. */
export type Placement =
  /** Its own paragraph, blank line above and below (exec, file, container). */
  | "block"
  /** Inside a parent's body, on its own line, indented (allow, needs). */
  | "child"
  /** In the flow of the line being typed (paste, val). */
  | "inline";

/** What sits between the open and close tags. */
export type BodyShape =
  /** Self-closing: `<hick:confirm … />`. */
  | "none"
  /** On the same line: `<hick:var name="x">2.0.0</hick:var>`. */
  | "inline"
  /** On its own lines, the shape every multi-line element uses. */
  | "block";

export interface InsertField {
  /** The attribute name, written verbatim. */
  name: string;
  label: string;
  /** What it means, shown under the input. One sentence. */
  hint: string;
  required?: boolean;
  placeholder?: string;
  /** A closed set renders as a select; the first entry is the default. */
  choices?: readonly string[];
  /** Prefilled when the form opens. */
  value?: string;
}

export interface InsertElement {
  /** Menu id. Usually the tag name; distinct where several menu entries
   * write the same tag (the capability rules all write `allow`). */
  id: string;
  /** The hick tag name, without the prefix. */
  tag: string;
  /** What the menu calls it. */
  title: string;
  group: string;
  /** One line, shown in the list and above the form. */
  summary: string;
  placement: Placement;
  body: BodyShape;
  fields: readonly InsertField[];
  /** Label for the body input. Absent when `body` is "none". */
  bodyLabel?: string;
  /** Written as the body when the caret carries no selection. */
  bodyPlaceholder?: string;
  /** A starter body that depends on one field's chosen value — the diagram's
   * body is mermaid text or scene JSON depending on `renderer`. Falls back
   * to `bodyPlaceholder` for values not listed. */
  bodyPlaceholderBy?: {
    field: string;
    bodies: Readonly<Record<string, string>>;
  };
  /** The element this one belongs inside, named in the form so a rule is
   * never inserted somewhere it means nothing. */
  belongsIn?: string;
}

const SHOW_MODES = ["", "command", "output", "none"] as const;
const FREEZE = ["", "true", "false"] as const;

/**
 * Every element the menu offers, in menu order.
 *
 * Attribute wording follows docs/docs/hick-guide.md; where the guide and the
 * implementation disagree the implementation wins and the guide is the bug.
 */
export const INSERT_ELEMENTS: readonly InsertElement[] = [
  // ---- Execution --------------------------------------------------------
  {
    id: "exec",
    tag: "exec",
    title: "Exec cell",
    group: "Execution",
    summary: "Run commands in a container and capture the transcript.",
    placement: "block",
    body: "block",
    bodyLabel: "Commands",
    bodyPlaceholder: "echo hello",
    fields: [
      {
        name: "container",
        label: "Container",
        hint: "Name of the container to run in. Execs sharing a container run in order.",
        placeholder: "reporter",
      },
      {
        name: "image",
        label: "Image",
        hint: "Only on the first exec of a container you did not declare — it creates one.",
        placeholder: "python:3.12",
      },
      {
        name: "mount",
        label: "Mount",
        hint: "volume:/path — mounting a volume is what creates a data dependency.",
        placeholder: "shared-output:/output",
      },
      {
        name: "show",
        label: "Show",
        hint: "Which part of the transcript a surrounding file gets. Blank means commands and output.",
        choices: SHOW_MODES,
      },
      {
        name: "freeze",
        label: "Freeze",
        hint: "true runs this cell once, ever, and replays the recording after. false always runs it.",
        choices: FREEZE,
      },
      {
        name: "timeout",
        label: "Timeout",
        hint: "Whole seconds. 0 means unbounded, explicitly. Blank inherits the 120s default.",
        placeholder: "600",
      },
      {
        name: "when",
        label: "When",
        hint: "Include this cell only when the condition holds — name, !name, name=value.",
        placeholder: "lint",
      },
    ],
  },
  {
    id: "expect",
    tag: "expect",
    title: "Expectation",
    group: "Execution",
    summary: "What the cell above must print for the document to pass.",
    placement: "child",
    body: "block",
    belongsIn: "exec",
    bodyLabel: "Expected output",
    bodyPlaceholder: "hello",
    fields: [
      {
        name: "match",
        label: "Match",
        hint: "exact compares byte for byte; regex-lines makes every line a full-line regex.",
        choices: ["", "exact", "regex-lines"],
      },
    ],
  },
  {
    id: "container",
    tag: "container",
    title: "Container",
    group: "Execution",
    summary: "An isolated environment, with the capabilities it is granted.",
    placement: "block",
    body: "block",
    bodyLabel: "Capability rules",
    bodyPlaceholder: '  <hick:deny network="*" />',
    fields: [
      {
        name: "name",
        label: "Name",
        hint: "How execs address this container.",
        required: true,
        placeholder: "reporter",
      },
      {
        name: "image",
        label: "Image",
        hint: "Recorded always, used by the Docker executor. Local runs use the machine's own programs.",
        placeholder: "python:3.12",
      },
    ],
  },
  {
    id: "needs",
    tag: "needs",
    title: "Needs a program",
    group: "Execution",
    summary: "A program this container's cells call — checked before anything runs.",
    placement: "child",
    body: "none",
    belongsIn: "container",
    fields: [
      {
        name: "bin",
        label: "Program",
        hint: "Looked for on the executor that will run the cell, not on this machine.",
        required: true,
        placeholder: "duckdb",
      },
      {
        name: "for",
        label: "Used for",
        hint: "Optional and worth writing: it tells a reader whether they want to install it.",
        placeholder: "the queries in section 3",
      },
    ],
  },
  {
    id: "secret",
    tag: "secret",
    title: "Secret",
    group: "Execution",
    summary: "An environment variable filled from the secrets provider, never from the document.",
    placement: "child",
    body: "none",
    belongsIn: "container",
    fields: [
      {
        name: "name",
        label: "Variable",
        hint: "The environment variable the cell reads.",
        required: true,
        placeholder: "PULUMI_TOKEN",
      },
      {
        name: "from",
        label: "Secret name",
        hint: "The entry in the secrets provider. The value never enters this file.",
        required: true,
        placeholder: "pulumi-token",
      },
    ],
  },
  {
    id: "volume",
    tag: "volume",
    title: "Volume",
    group: "Execution",
    summary: "Storage containers share, and optionally a host directory it maps.",
    placement: "block",
    body: "block",
    bodyLabel: "Access rules",
    bodyPlaceholder: "",
    fields: [
      {
        name: "name",
        label: "Name",
        hint: "What a mount= refers to.",
        required: true,
        placeholder: "shared-output",
      },
      {
        name: "input",
        label: "Input",
        hint: "Host directory snapshotted into the volume before the run.",
        placeholder: ".",
      },
      {
        name: "output",
        label: "Output",
        hint: "Contents become pipeline file outputs when the run finishes.",
        placeholder: "dist/",
      },
    ],
  },
  {
    id: "agent",
    tag: "agent",
    title: "Agent cell",
    group: "Execution",
    summary: "A reasoning step in the document, scheduled like an exec.",
    placement: "block",
    body: "block",
    bodyLabel: "Prompt",
    bodyPlaceholder: "  <hick:prompt>Describe what the agent should do.</hick:prompt>",
    fields: [
      {
        name: "id",
        label: "Id",
        hint: "Identifies the cell across the agent's own edits, which move every line number.",
        required: true,
        placeholder: "impl-tokenizer",
      },
      {
        name: "model",
        label: "Model",
        hint: "Optional. Naming it is what lets the cell be verified from its recording without credentials.",
        placeholder: "claude-sonnet-5",
      },
      {
        name: "max-turns",
        label: "Max turns",
        hint: "A graph invariant, not a budget: an exhausted allowance fails the cell.",
        placeholder: "20",
      },
      {
        name: "freeze",
        label: "Freeze",
        hint: "Exactly as on an exec cell.",
        choices: FREEZE,
      },
    ],
  },
  {
    id: "confirm",
    tag: "confirm",
    title: "Confirmation gate",
    group: "Execution",
    summary: "Stop and ask before going further.",
    placement: "block",
    body: "none",
    fields: [
      {
        name: "message",
        label: "Question",
        hint: "What the person is agreeing to. Say the consequence, not just the step.",
        required: true,
        placeholder: "Deploy to production?",
      },
    ],
  },
  {
    id: "verify",
    tag: "verify",
    title: "Verify command",
    group: "Execution",
    summary: "A command that checks the generated files, run outside the document's cells.",
    placement: "block",
    body: "none",
    fields: [
      {
        name: "command",
        label: "Command",
        hint: "Run against what this document produced.",
        required: true,
        placeholder: "cargo check",
      },
      {
        name: "description",
        label: "Description",
        hint: "What a failure would mean.",
        placeholder: "Run backend tests",
      },
    ],
  },
  {
    id: "capture",
    tag: "capture",
    title: "Capture",
    group: "Execution",
    summary: "Record named expressions at a line of a generated program while it runs.",
    placement: "block",
    body: "none",
    fields: [
      {
        name: "at",
        label: "At",
        hint: "file:line inside a file this document generates.",
        required: true,
        placeholder: "pricing.py:14",
      },
      {
        name: "of",
        label: "Of",
        hint: "Comma-separated expressions, evaluated in that frame.",
        required: true,
        placeholder: "subtotal, len(lines)",
      },
      {
        name: "condition",
        label: "Condition",
        hint: "Capture only when this is true — the way to catch the one bad iteration.",
        placeholder: "subtotal < 0",
      },
      {
        name: "max",
        label: "Max",
        hint: "Stop after this many hits.",
        placeholder: "20",
      },
    ],
  },
  {
    id: "feature",
    tag: "feature",
    title: "Feature",
    group: "Execution",
    summary: "An optional section of the document, off unless asked for.",
    placement: "block",
    body: "none",
    fields: [
      {
        name: "name",
        label: "Name",
        hint: "Enabled with hick --features <name>.",
        required: true,
        placeholder: "with-r",
      },
      {
        name: "description",
        label: "Description",
        hint: "What it adds, and what it needs installed.",
        placeholder: "The R section (needs Rscript with ggplot2)",
      },
    ],
  },

  // ---- Capabilities -----------------------------------------------------
  {
    id: "allow-network",
    tag: "allow",
    title: "Allow network",
    group: "Capabilities",
    summary: "Give a container a network. Without one of these it has none.",
    placement: "child",
    body: "none",
    belongsIn: "container",
    fields: [
      {
        name: "network",
        label: "Host and port",
        hint: "host:port, either side wildcardable. The sandbox grants a network; it does not police the host.",
        required: true,
        value: "example.com:443",
      },
    ],
  },
  {
    id: "deny-network",
    tag: "deny",
    title: "Deny network",
    group: "Capabilities",
    summary: "Say out loud that everything not allowed above is refused.",
    placement: "child",
    body: "none",
    belongsIn: "container",
    fields: [
      {
        name: "network",
        label: "Pattern",
        hint: "Alone this means no network, which is already the default — it documents intent.",
        required: true,
        value: "*",
      },
    ],
  },
  {
    id: "allow-file-read",
    tag: "allow",
    title: "Allow file read",
    group: "Capabilities",
    summary: "Let a container read under a path.",
    placement: "child",
    body: "none",
    belongsIn: "container",
    fields: [
      {
        name: "file-read",
        label: "Path",
        hint: "A glob. Reading is not implied by anything else.",
        required: true,
        placeholder: "/data/*",
      },
    ],
  },
  {
    id: "allow-file-write",
    tag: "allow",
    title: "Allow file write",
    group: "Capabilities",
    summary: "Let a container write under a path. Writing implies reading.",
    placement: "child",
    body: "none",
    belongsIn: "container",
    fields: [
      {
        name: "file-write",
        label: "Path",
        hint: "A glob. Write access carries read access with it.",
        required: true,
        placeholder: "/output/*",
      },
    ],
  },
  {
    id: "volume-access",
    tag: "allow",
    title: "Volume access rule",
    group: "Capabilities",
    summary: "Which part of a volume one container may read and write.",
    placement: "child",
    body: "none",
    belongsIn: "volume",
    fields: [
      {
        name: "container",
        label: "Container",
        hint: "Naming anyone binds everyone: a container these rules never mention cannot mount the volume at all.",
        required: true,
        placeholder: "linter",
      },
      {
        name: "read",
        label: "Read",
        hint: "A glob within the volume.",
        placeholder: "**",
      },
      {
        name: "write",
        label: "Write",
        hint: "A glob within the volume. Only granted paths merge back out.",
        placeholder: "Controllers/**",
      },
    ],
  },
  {
    id: "fork",
    tag: "fork",
    title: "Fork a container",
    group: "Capabilities",
    summary: "Clone a container's state into a new one, narrower than the original.",
    placement: "block",
    body: "block",
    bodyLabel: "Added restrictions",
    bodyPlaceholder: '  <hick:deny network="*" />',
    fields: [
      {
        name: "from",
        label: "From",
        hint: "The container to clone. The fork depends on its last exec.",
        required: true,
        placeholder: "base",
      },
      {
        name: "to",
        label: "To",
        hint: "The new container's name.",
        required: true,
        placeholder: "analyzer",
      },
    ],
  },
  {
    id: "attenuate",
    tag: "attenuate",
    title: "Attenuate a container",
    group: "Capabilities",
    summary: "Narrow a container in place. Capabilities only ever shrink.",
    placement: "block",
    body: "block",
    bodyLabel: "Added restrictions",
    bodyPlaceholder: '  <hick:deny network="*" />',
    fields: [
      {
        name: "container",
        label: "Container",
        hint: "The container to restrict from here on.",
        required: true,
        placeholder: "sandbox",
      },
    ],
  },

  // ---- Reuse ------------------------------------------------------------
  {
    id: "copy",
    tag: "copy",
    title: "Copy",
    group: "Reuse",
    summary: "Name a piece of content so it can be pasted, and keep it here too.",
    placement: "block",
    body: "block",
    bodyLabel: "Content",
    bodyPlaceholder: "The text to reuse.",
    fields: [
      {
        name: "id",
        label: "Id",
        hint: "What a paste selects with #id. Ids reach across documents in one run.",
        required: true,
        placeholder: "version",
      },
    ],
  },
  {
    id: "cut",
    tag: "cut",
    title: "Cut",
    group: "Reuse",
    summary: "Same as copy, except this content is removed from the output.",
    placement: "block",
    body: "block",
    bodyLabel: "Content",
    bodyPlaceholder: "Available to paste, absent from the output.",
    fields: [
      {
        name: "id",
        label: "Id",
        hint: "What a paste selects with #id.",
        required: true,
        placeholder: "internal-notes",
      },
    ],
  },
  {
    id: "paste",
    tag: "paste",
    title: "Paste",
    group: "Reuse",
    summary: "Insert a copied or cut fragment here, by id.",
    placement: "inline",
    body: "none",
    fields: [
      {
        name: "select",
        label: "Select",
        hint: "The id of a copy or cut, with a leading # (added for you if you leave it off).",
        required: true,
        placeholder: "#version",
      },
    ],
  },

  // ---- Documents and prose ----------------------------------------------
  {
    id: "file",
    tag: "file",
    title: "Generated file",
    group: "Document",
    summary: "A file this document writes, whose content is prose, pastes, and transcripts.",
    placement: "block",
    body: "block",
    bodyLabel: "File content",
    bodyPlaceholder: "",
    fields: [
      {
        name: "path",
        label: "Path",
        hint: "Relative to the output directory.",
        required: true,
        placeholder: "report.md",
      },
      {
        name: "when",
        label: "When",
        hint: "Write this file only when the condition holds.",
        placeholder: "debug",
      },
    ],
  },
  {
    id: "var",
    tag: "var",
    title: "Variable",
    group: "Document",
    summary: "A value used through the document, overridable with --param.",
    placement: "block",
    body: "inline",
    bodyLabel: "Value",
    bodyPlaceholder: "2.0.0",
    fields: [
      {
        name: "name",
        label: "Name",
        hint: "Referenced by a val tag, a when test, or --param name=value.",
        required: true,
        placeholder: "version",
      },
    ],
  },
  {
    id: "val",
    tag: "val",
    title: "Variable value",
    group: "Document",
    summary: "Write a variable's value here.",
    placement: "inline",
    body: "none",
    fields: [
      {
        name: "name",
        label: "Name",
        hint: "A variable declared above, or supplied with --param.",
        required: true,
        placeholder: "version",
      },
    ],
  },
  {
    id: "when",
    tag: "when",
    title: "Conditional block",
    group: "Document",
    summary: "Content that is kept or dropped whole, by a condition.",
    placement: "block",
    body: "block",
    bodyLabel: "Content",
    bodyPlaceholder: "",
    fields: [
      {
        name: "test",
        label: "Test",
        hint: "name (set and non-empty), !name, name=value, or name!=value.",
        required: true,
        placeholder: "mode=prod",
      },
    ],
  },
  {
    id: "diagram",
    tag: "diagram",
    title: "Diagram",
    group: "Document",
    summary: "A drawing that can name the cell which proves it still tells the truth.",
    placement: "block",
    body: "block",
    bodyLabel: "Diagram source",
    bodyPlaceholder: "graph TD\n  A --> B",
    // The starter body is the renderer's: mermaid is text, a graph scene is
    // JSON the canvas can open. Inserting mermaid text under
    // renderer="graph" would greet the author with a parse error.
    bodyPlaceholderBy: {
      field: "renderer",
      bodies: {
        graph:
          '{\n' +
          '  "nodes": [\n' +
          '    {"id": "a", "label": "A"},\n' +
          '    {"id": "b", "label": "B"}\n' +
          '  ],\n' +
          '  "edges": [\n' +
          '    {"from": "a", "to": "b"}\n' +
          '  ],\n' +
          '  "layout": {}\n' +
          '}',
      },
    },
    fields: [
      {
        name: "renderer",
        label: "Renderer",
        hint: "mermaid is text you type; graph is a JSON scene you drag on a canvas. Any other renderer still weaves as a tagged fence.",
        choices: ["mermaid", "graph"],
      },
      {
        name: "asserts",
        label: "Asserts",
        hint: "The #id of a cell that fails when this diagram stops being true. Optional, and its absence is visible.",
        placeholder: "#no-back-edges",
      },
    ],
  },
  {
    id: "table",
    tag: "table",
    title: "Table",
    group: "Document",
    summary:
      "A dataset that is also prose: CSV in the document, a markdown table in the weave, and the file itself when it names one.",
    placement: "block",
    body: "block",
    bodyLabel: "CSV",
    bodyPlaceholder: "region,units\nnorth,120",
    fields: [
      {
        name: "path",
        label: "Path",
        hint: "Where the CSV is written. Leave empty for a table that writes no file.",
        placeholder: "data/sales.csv",
      },
      {
        name: "delimiter",
        label: "Delimiter",
        hint: "Comma unless said otherwise. `tab` is spelled out — a tab cannot be typed into an attribute.",
        choices: [",", "tab", ";", "|"],
      },
      {
        name: "header",
        label: "Header row",
        hint: "The first row names the columns. Set false when it is data.",
        choices: ["true", "false"],
      },
      {
        name: "language",
        label: "Formula language",
        hint: "Cells beginning with `=` are expressions in this language. Leave empty and `=` is just text.",
        choices: ["python", "javascript"],
      },
    ],
  },
  {
    id: "math",
    tag: "math",
    title: "Equation",
    group: "Document",
    summary: "Display maths, typeset in the document and woven as `$$…$$`.",
    placement: "block",
    body: "block",
    bodyLabel: "LaTeX",
    bodyPlaceholder: "e = mc^2",
    fields: [],
  },
  {
    id: "transform",
    tag: "transform",
    title: "Transform",
    group: "Document",
    summary: "A passage written by a model from named fragments, pinned to their fingerprint.",
    placement: "block",
    body: "block",
    bodyLabel: "Passage",
    bodyPlaceholder: "",
    fields: [
      {
        name: "select",
        label: "Select",
        hint: "The fragments the model may read — .class or #id.",
        required: true,
        placeholder: ".incident",
      },
      {
        name: "instruct",
        label: "Instruction",
        hint: "What to do with them. The instruction is part of what the fingerprint covers.",
        required: true,
        placeholder: "Summarize this for a busy engineer in at most three sentences.",
      },
      {
        name: "from",
        label: "Fingerprint",
        hint: "Leave empty — hick refresh writes it, and hick test then checks the passage against it.",
        placeholder: "",
      },
    ],
  },
  {
    id: "include",
    tag: "include",
    title: "Include a file",
    group: "Document",
    summary: "Splice another document's nodes in here.",
    placement: "block",
    body: "none",
    fields: [
      {
        name: "file",
        label: "File",
        hint: "Relative to this document.",
        required: true,
        placeholder: "shared/preamble.md",
      },
    ],
  },
  {
    id: "upstream",
    tag: "upstream",
    title: "Upstream document",
    group: "Document",
    summary: "A pipeline edge: everything upstream becomes selectable, and none of it is rendered.",
    placement: "block",
    body: "none",
    fields: [
      {
        name: "file",
        label: "File",
        hint: "Relative to this document. This is what lets you paste a decision three hops back.",
        required: true,
        placeholder: "../decisions/api.md",
      },
    ],
  },
];

/** Menu groups, in the order the panel lists them. */
export const INSERT_GROUPS: readonly string[] = ["Execution", "Capabilities", "Reuse", "Document"];

export function elementById(id: string): InsertElement | undefined {
  return INSERT_ELEMENTS.find((element) => element.id === id);
}

/** The catalogue entry that describes an existing tag. Several capability
 * entries write `hick:allow`; their attribute names distinguish them. */
export function elementForExisting(
  tag: string,
  attrs: Readonly<Record<string, string>>,
): InsertElement | undefined {
  const candidates = INSERT_ELEMENTS.filter((element) => element.tag === tag);
  return candidates.sort((a, b) => {
    const score = (element: InsertElement) =>
      element.fields.reduce((total, field) => total + (field.name in attrs ? 1 : 0), 0);
    return score(b) - score(a);
  })[0];
}

/**
 * Rank elements against a typed query.
 *
 * Title first, then tag, then the summary — a person typing "net" wants
 * "Allow network" before "Container", whose summary happens to mention one.
 * An empty query keeps catalogue order.
 */
export function filterElements(query: string): InsertElement[] {
  const q = query.trim().toLowerCase();
  if (!q) return [...INSERT_ELEMENTS];
  const scored: { element: InsertElement; score: number }[] = [];
  for (const element of INSERT_ELEMENTS) {
    const title = element.title.toLowerCase();
    const score = title.startsWith(q)
      ? 0
      : title.includes(q)
        ? 1
        : element.tag.includes(q)
          ? 2
          : element.summary.toLowerCase().includes(q)
            ? 3
            : -1;
    if (score >= 0) scored.push({ element, score });
  }
  scored.sort((a, b) => a.score - b.score);
  return scored.map((s) => s.element);
}

export {
  initialValues, normalize, validate, renderElement, renderExistingElement,
  defaultBody, buildInsertion,
} from "./insertRendering";
export type { FieldValues, EditContext, Insertion } from "./insertRendering";
