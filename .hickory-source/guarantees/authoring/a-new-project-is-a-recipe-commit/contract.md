<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A New Project Is A Recipe Commit, Made As One Act

For: Authors editing executable documents and the files those documents produce.

Given a `dotnet new` template chosen in File → New Project, a **location**
— any folder on this machine — and a folder name for its files, when the
project is created, then the scaffolder runs in a terminal, in a scratch
directory, and what it wrote is committed the moment it exits zero — in one
act, with nothing a person can do in between — as a commit whose last
paragraph carries the recipe:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)
- [Requirements 3](requirements/03.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-new-project-is-a-recipe-commit.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />
- [Requirement detail](requirements/03.md)
<storycode:include file="requirements/03.md" />

## Implementation

<a id="output-84475d9aff04"></a>
## `apps/web/src/lib/scaffold.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A New Project Is A Recipe Commit, Made As One Act](contract.md).

<a id="f-84475d9aff04-option"></a>
### `option`

Protects docs/guarantees/authoring/a-new-project-is-a-recipe-commit.md

<storycode:copy id="f-84475d9aff04-option">
// Protects docs/guarantees/authoring/a-new-project-is-a-recipe-commit.md

import { describe, expect, it } from "vitest";

import {
  NO_RESTORE,
  chosenOptions,
  filterTemplates,
  grouped,
  initialLanguage,
  initialValues,
  problems,
  slug,
} from "./scaffold";
import type { ScaffoldTemplate, TemplateOption } from "../api/types";

function option(over: Partial<TemplateOption> = {}): TemplateOption {
  return {
    names: ["--framework"],
    flag: "--framework",
    kind: "choice",
    choices: [
      { value: "net10.0", description: "" },
      { value: "net9.0", description: "" },
    ],
    default: "net10.0",
    description: "The target framework for the project.",
    enabled_if: null,
    ...over,
  };
}

</storycode:copy>
<a id="f-84475d9aff04-template"></a>
### `template`

<storycode:copy id="f-84475d9aff04-template">
function template(over: Partial<ScaffoldTemplate> = {}): ScaffoldTemplate {
  return {
    short_names: ["webapi"],
    name: "ASP.NET Core Web API",
    languages: ["C#", "F#"],
    default_language: "C#",
    tags: ["Web", "Web API"],
    ...over,
  };
}

</storycode:copy>
<a id="f-84475d9aff04-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-84475d9aff04-expression_statement">
describe("what ends up on the command line", () => {
  it("writes nothing for a field left at its default", () => {
    const options = [option()];
    expect(chosenOptions(options, initialValues(options))).toEqual([]);
  });

  it("writes the flag once a field says something else", () => {
    const options = [option()];
    expect(chosenOptions(options, { "--framework": "net9.0" })).toEqual([
      { flag: "--framework", value: "net9.0" },
    ]);
  });

  it("leaves a choice to dotnet when it is cleared", () => {
    // "leave it alone" and "pick the value that is the default today" are
    // different intents; an empty field is the first one.
    const options = [option()];
    expect(chosenOptions(options, { "--framework": "" })).toEqual([]);
  });

  it("writes a bool turned on as the bare flag", () => {
    const options = [
      option({ names: ["--aot"], flag: "--aot", kind: "bool", default: "false", choices: [] }),
    ];
    expect(chosenOptions(options, { "--aot": "true" })).toEqual([{ flag: "--aot" }]);
  });

  it("spells out a bool turned off, because the bare flag means true", () => {
    const options = [
      option({
        names: ["--nullable"],
        flag: "--nullable",
        kind: "bool",
        default: "true",
        choices: [],
      }),
    ];
    expect(chosenOptions(options, { "--nullable": "false" })).toEqual([
      { flag: "--nullable", value: "false" },
    ]);
    expect(chosenOptions(options, { "--nullable": "true" })).toEqual([]);
  });

  it("treats dotnet's False and false as the same answer", () => {
    const options = [
      option({ names: ["--sdk"], flag: "--sdk", kind: "bool", default: "False", choices: [] }),
    ];
    expect(chosenOptions(options, { "--sdk": "false" })).toEqual([]);
  });

  it("starts --no-restore on, unlike dotnet's own default", () => {
    // A restore fills obj/ inside the volume the ingest reads. Not generating
    // it is the answer that does not depend on a .gitignore being right.
    const options = [
      option({ names: [NO_RESTORE], flag: NO_RESTORE, kind: "bool", default: "false", choices: [] }),
    ];
    const values = initialValues(options);
    expect(values[NO_RESTORE]).toBe("true");
    expect(chosenOptions(options, values)).toEqual([{ flag: NO_RESTORE }]);
  });
});

</storycode:copy>
<a id="f-84475d9aff04-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-84475d9aff04-expression_statement-2">
describe("the template list", () => {
  it("groups by the first tag and searches name, short name and tag", () => {
    const templates = [
      template(),
      template({ short_names: ["console"], name: "Console App", tags: ["Common", "Console"] }),
    ];
    expect(grouped(templates).map((g) => g.group)).toEqual(["Common", "Web"]);
    expect(filterTemplates(templates, "api").map((t) => t.short_names[0])).toEqual(["webapi"]);
    expect(filterTemplates(templates, "console").map((t) => t.short_names[0])).toEqual([
      "console",
    ]);
    // By tag, for somebody who knows what it is but not what it is called.
    expect(filterTemplates(templates, "common").map((t) => t.short_names[0])).toEqual([
      "console",
    ]);
  });

  it("asks for no language when the template has none", () => {
    // `gitignore` takes no --language at all, and passing one is an error
    // rather than a no-op — so "no language" and "one language" stay apart.
    expect(initialLanguage(template({ languages: [], default_language: null }))).toBe("");
    expect(initialLanguage(template())).toBe("C#");
  });
});

</storycode:copy>
<a id="f-84475d9aff04-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-84475d9aff04-expression_statement-3">
describe("the fields the template does not own", () => {
  it("suggests a path and a folder from the name", () => {
    expect(slug("Greeter")).toBe("greeter");
    expect(slug("Company.WebApplication1")).toBe("company-webapplication1");
    expect(slug("///")).toBe("project");
  });

  it("refuses a document that is not a document", () => {
    expect(problems("", "greeter").name).toBeTruthy();
    expect(problems("Greeter", "").output).toBeTruthy();
    expect(problems("Greeter", ".").output).toBeTruthy();
    expect(problems("Greeter", "../out").output).toBeTruthy();
    expect(problems("Greeter", "apps/greeter")).toEqual({});
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/scaffold.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/scaffold.test.ts"><storycode:paste select="#f-84475d9aff04-option"/><storycode:paste select="#f-84475d9aff04-template"/><storycode:paste select="#f-84475d9aff04-expression_statement"/><storycode:paste select="#f-84475d9aff04-expression_statement-2"/><storycode:paste select="#f-84475d9aff04-expression_statement-3"/></storycode:file>

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/server.rs](../../../mechanisms/apps/desktop/src-tauri/src/server.rs.md#output-821f8d0a36be)
- [apps/web/src/App.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-25cc6f821b46)
- [apps/web/src/api/client.ts](../../../mechanisms/apps/web/src/api/client.ts.md#output-1e909407d4cd)
- [apps/web/src/components/NewProjectDialog.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-65e48889b856)
- [crates/hick-plugins/src/scaffold.rs](../../../mechanisms/crates/hick-plugins/src/scaffold.rs.md#output-6b3ddf926d91)
- [crates/hickory-cli/src/scaffold.rs](../../../mechanisms/crates/hickory-cli/src/scaffold.rs.md#output-a3d176caef4d)
- [crates/hickory-cli/src/scaffold_commit.rs](../../../mechanisms/crates/hickory-cli/src/scaffold_commit.rs.md#output-b1965dc85e6b)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)
- [crates/hickory-cli/src/serve/scaffold.rs](../../../mechanisms/crates/hickory-cli/src/serve/scaffold.rs.md#output-3fd865a33211)

### Tests and fixtures

- [apps/web/src/components/NewProjectDialog.test.tsx](../../../mechanisms/apps/web/src/components/NewProjectDialog.tsx.md#output-065ce188ea68)
- [apps/web/src/lib/scaffold.test.ts](contract.md#output-84475d9aff04)
- [crates/hickory-cli/tests/scaffold_templates.rs](../../../mechanisms/crates/hickory-cli/tests/scaffold_templates.rs.md#output-65595a9b8013)
- [crates/hickory-cli/tests/serve_scaffold.rs](../../../mechanisms/crates/hickory-cli/tests/serve_scaffold.rs.md#output-5a76df8fd4d1)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
