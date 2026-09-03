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
