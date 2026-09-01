// @vitest-environment jsdom
//
// Protects
// docs/guarantees/authoring/a-new-project-reads-the-scaffolder-s-own-options.md
// and docs/guarantees/authoring/a-machine-with-no-sdk-says-so.md

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

vi.mock("../api/client", () => ({
  ApiError: class ApiError extends Error {
    constructor(
      public status: number,
      message: string,
      public body?: unknown,
    ) {
      super(message);
    }
  },
  api: {
    scaffoldTemplates: vi.fn(),
    scaffoldOptions: vi.fn(),
    scaffoldPreview: vi.fn(),
    scaffoldCreate: vi.fn(),
  },
}));

import { NewProjectDialog } from "./NewProjectDialog";
import { ApiError, api } from "../api/client";
import type { ScaffoldCatalog, ScaffoldTemplateDetail } from "../api/types";

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const CATALOG: ScaffoldCatalog = {
  kind: "dotnet",
  sdk_version: "10.0.111",
  image: "mcr.microsoft.com/dotnet/sdk:10.0",
  templates: [
    {
      short_names: ["console"],
      name: "Console App",
      languages: ["C#", "F#"],
      default_language: "C#",
      tags: ["Common", "Console"],
    },
    {
      short_names: ["webapi"],
      name: "ASP.NET Core Web API",
      languages: ["C#"],
      default_language: "C#",
      tags: ["Web", "Web API"],
    },
  ],
};

const DETAIL: ScaffoldTemplateDetail = {
  title: "Console App (C#)",
  author: "Microsoft",
  description: "A project for creating a command-line application.",
  options: [
    {
      names: ["-f", "--framework"],
      flag: "--framework",
      kind: "choice",
      choices: [
        { value: "net10.0", description: "Target net10.0" },
        { value: "net9.0", description: "Target net9.0" },
      ],
      default: "net10.0",
      description: "The target framework for the project.",
      enabled_if: null,
    },
    {
      names: ["--no-restore"],
      flag: "--no-restore",
      kind: "bool",
      choices: [],
      default: "false",
      description: "If specified, skips the automatic restore of the project on create.",
      enabled_if: null,
    },
  ],
  other_languages: ["F#", "VB"],
};

function ready() {
  vi.mocked(api.scaffoldTemplates).mockResolvedValue(CATALOG);
  vi.mocked(api.scaffoldOptions).mockResolvedValue(DETAIL);
  vi.mocked(api.scaffoldPreview).mockResolvedValue({
    path: "greeter.hick",
    command: "dotnet new console -o out -n Greeter --language 'C#' --no-restore",
    source: "# Greeter\n\n<hick:copy id=\"scaffold\">\n",
  });
  const onCreated = vi.fn();
  const onClose = vi.fn();
  render(<NewProjectDialog onCreated={onCreated} onClose={onClose} />);
  return { onCreated, onClose };
}

describe("the machine's own templates", () => {
  it("lists what dotnet has, grouped by its own tags", async () => {
    ready();
    // Twice on purpose: the row in the list, and the heading of the form it
    // opened on.
    expect((await screen.findAllByText("Console App")).length).toBe(2);
    expect(screen.getByText("ASP.NET Core Web API")).toBeTruthy();
    expect(screen.getByText("Common")).toBeTruthy();
    expect(screen.getByText("Web")).toBeTruthy();
    // Which SDK is answering is not something to go looking for.
    expect(screen.getByText("10.0.111")).toBeTruthy();
  });

  it("draws the template's own options as fields", async () => {
    ready();
    await screen.findByText("-f, --framework");
    // A choice the person may decline to make at all.
    expect(screen.getByText("(leave to dotnet)")).toBeTruthy();
    expect(screen.getByText(/Target net9\.0/)).toBeTruthy();
  });

  it("re-reads the options when the language changes", async () => {
    ready();
    await screen.findByText("-f, --framework");
    expect(api.scaffoldOptions).toHaveBeenCalledWith("console", "C#");

    fireEvent.change(screen.getByDisplayValue("C#"), { target: { value: "F#" } });
    await waitFor(() =>
      expect(api.scaffoldOptions).toHaveBeenCalledWith("console", "F#"),
    );
  });
});

describe("the fields the template does not own", () => {
  it("follows the project name until the person takes the wheel", async () => {
    ready();
    await screen.findByDisplayValue("Greeter");

    fireEvent.change(screen.getByDisplayValue("Greeter"), {
      target: { value: "Widgets" },
    });
    expect(screen.getByDisplayValue("widgets.hick")).toBeTruthy();
    expect(screen.getByDisplayValue("widgets")).toBeTruthy();

    // Once touched, the path is theirs: renaming must not overwrite it.
    fireEvent.change(screen.getByDisplayValue("widgets.hick"), {
      target: { value: "notes/mine.hick" },
    });
    fireEvent.change(screen.getByDisplayValue("Widgets"), {
      target: { value: "Gadgets" },
    });
    expect(screen.getByDisplayValue("notes/mine.hick")).toBeTruthy();
  });

  it("creates with what the form says, and closes", async () => {
    const { onCreated, onClose } = ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue({
      id: "abc",
      path: "greeter.hick",
      source: "# Greeter\n",
      ingested: { from: "#scaffold", fingerprint: "ab", files: ["greeter/Program.cs"], skipped: [] },
      executor: "sandbox",
      note: null,
    });
    await screen.findByText("-f, --framework");

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(onCreated).toHaveBeenCalled());
    expect(onClose).toHaveBeenCalled();

    const [path, spec, run] = vi.mocked(api.scaffoldCreate).mock.calls[0];
    expect(path).toBe("greeter.hick");
    expect(run).toBe(true);
    expect(spec.template).toBe("console");
    expect(spec.name).toBe("Greeter");
    expect(spec.output).toBe("greeter");
    expect(spec.image).toBe("mcr.microsoft.com/dotnet/sdk:10.0");
    // Only what was decided: --framework was left at its default and is not
    // written; --no-restore starts on here and is.
    expect(spec.options).toEqual([{ flag: "--no-restore" }]);
  });

  it("shows the failure and stays open rather than losing the form", async () => {
    const { onClose } = ready();
    vi.mocked(api.scaffoldCreate).mockRejectedValue(
      new Error("greeter.hick already exists. Open it, or choose another name."),
    );
    await screen.findByText("-f, --framework");

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await screen.findByText(/already exists/);
    expect(onClose).not.toHaveBeenCalled();
  });
});

describe("a machine with no SDK", () => {
  it("gets its own screen, keyed off the field and not the wording", async () => {
    // The reason the route carries `missing: "dotnet"` at all: a reworded
    // message must not be able to take this screen away.
    vi.mocked(api.scaffoldTemplates).mockRejectedValue(
      new ApiError(422, "some sentence nobody should be matching on", {
        missing: "dotnet",
      }),
    );
    render(<NewProjectDialog onCreated={vi.fn()} onClose={vi.fn()} />);

    await screen.findByText("No .NET SDK on this machine");
    // A sentence and a link, not a button: this product has no catalogue for
    // a platform install and no business fetching one.
    expect(screen.queryByRole("button", { name: /install/i })).toBeNull();
    expect(screen.getByText(/dotnet\.microsoft\.com\/download/)).toBeTruthy();
  });

  it("reports any other failure as itself", async () => {
    vi.mocked(api.scaffoldTemplates).mockRejectedValue(
      new ApiError(500, "the template listing did not finish", {}),
    );
    render(<NewProjectDialog onCreated={vi.fn()} onClose={vi.fn()} />);
    await screen.findByText("Could not read the templates");
  });
});
