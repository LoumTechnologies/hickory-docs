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
  location: "/home/nate/notes",
  separator: "/",
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
    output: "greeter",
    folder: "/home/nate/notes/greeter",
    repository: "/home/nate/notes",
    command: "dotnet new console -o greeter -n Greeter --language 'C#' --no-restore",
    message:
      "Scaffold Greeter with `dotnet new console`\n\nHick-Recipe: dotnet new console -o greeter -n Greeter --language 'C#' --no-restore\n",
  });
  const onStarted = vi.fn();
  const onClose = vi.fn();
  render(<NewProjectDialog onStarted={onStarted} onClose={onClose} />);
  return { onStarted, onClose };
}

/** What `POST /api/scaffold` answers: a terminal, not a commit. */
const STARTED = {
  session: {
    id: "term-3",
    title: "New project: Greeter",
    cwd: "/tmp/scratch",
    monitor: false,
    state: "working",
    since_ms: 0,
    branch: null,
    dirty: false,
    preview: "",
    prompt: null,
    exit_code: null,
  },
  output: "greeter",
  folder: "/home/nate/notes/greeter",
  repository: "/home/nate/notes",
} as never;

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
    expect(screen.getByDisplayValue("widgets")).toBeTruthy();

    // Once touched, the folder is theirs: renaming must not overwrite it.
    fireEvent.change(screen.getByDisplayValue("widgets"), {
      target: { value: "apps/mine" },
    });
    fireEvent.change(screen.getByDisplayValue("Widgets"), {
      target: { value: "Gadgets" },
    });
    expect(screen.getByDisplayValue("apps/mine")).toBeTruthy();
  });

  it("hands over the terminal it started, and closes", async () => {
    // The button does not wait for a commit: it starts the scaffolder in a
    // terminal and gets out of the way.
    // docs/guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal.md
    const { onStarted, onClose } = ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue(STARTED);
    await screen.findByText("-f, --framework");
    // The preview is the commit, trailers and all.
    await screen.findByText(/Hick-Recipe: dotnet new console -o greeter/);

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(onStarted).toHaveBeenCalledWith(STARTED));
    expect(onClose).toHaveBeenCalled();

    const [spec] = vi.mocked(api.scaffoldCreate).mock.calls[0];
    expect(spec.template).toBe("console");
    expect(spec.name).toBe("Greeter");
    expect(spec.output).toBe("greeter");
    // The location is a field, and it is sent: a project is made where the
    // person said, not inside the folder the app happens to have open.
    expect(spec.location).toBe("/home/nate/notes");
    expect(spec.image).toBe("mcr.microsoft.com/dotnet/sdk:10.0");
    // Only what was decided: --framework was left at its default and is not
    // written; --no-restore starts on here and is.
    expect(spec.options).toEqual([{ flag: "--no-restore" }]);
  });

  it("makes a project anywhere on the machine", async () => {
    const { onStarted } = ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue(STARTED);
    await screen.findByText("-f, --framework");

    fireEvent.change(screen.getByLabelText("Location"), {
      target: { value: "/home/nate/src" },
    });
    // The dialog shows where the two fields land together, so the person can
    // read the answer rather than assemble it.
    await screen.findByText("/home/nate/src/greeter");

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(onStarted).toHaveBeenCalled());
    const [spec] = vi.mocked(api.scaffoldCreate).mock.calls[0];
    expect(spec.location).toBe("/home/nate/src");
    expect(spec.output).toBe("greeter");
  });

  it("names the repository that will record it when it is not the open one", async () => {
    ready();
    vi.mocked(api.scaffoldPreview).mockResolvedValue({
      output: "apps/greeter",
      folder: "/home/nate/src/apps/greeter",
      repository: "/home/nate/src",
      command: "dotnet new console -o apps/greeter -n Greeter",
      message: "Scaffold Greeter with `dotnet new console`\n",
    });
    fireEvent.change(await screen.findByLabelText("Location"), {
      target: { value: "/home/nate/src/apps" },
    });
    await screen.findByText(/not the folder you have open/);
  });

  it("shows the failure and stays open rather than losing the form", async () => {
    const { onClose } = ready();
    vi.mocked(api.scaffoldCreate).mockRejectedValue(
      new Error("`greeter/` already exists and is not empty."),
    );
    await screen.findByText("-f, --framework");

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await screen.findByText(/already exists/);
    expect(onClose).not.toHaveBeenCalled();
  });

  it("ticks 'create a git repository' by itself when the location has none", async () => {
    // The repository is a checkbox on the form, not a screen after a refusal:
    // by the time you have typed a location and a name you have said what you
    // want. It is ticked FOR you and stays yours to untick.
    ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue(STARTED);
    await screen.findByText("-f, --framework");
    // Inside a repository already: nothing to make, and it says which one.
    // Waited for, because it is the preview that knows — the checkbox has no
    // opinion of its own until the server has resolved the location.
    await screen.findByText(/Already in one:/);
    const box = screen.getByRole("checkbox", { name: /Create a git repository/ });
    expect((box as HTMLInputElement).checked).toBe(false);
    expect((box as HTMLInputElement).disabled).toBe(true);

    vi.mocked(api.scaffoldPreview).mockResolvedValue({
      output: "greeter",
      folder: "/home/nate/src/fresh/greeter",
      repository: null,
      needs_repository: "/home/nate/src/fresh",
      problem: "not inside a git repository",
      command: "dotnet new console -o greeter -n Greeter",
      message: "Scaffold Greeter with `dotnet new console`\n",
    });
    fireEvent.change(screen.getByLabelText("Location"), {
      target: { value: "/home/nate/src/fresh" },
    });
    await waitFor(() =>
      expect(
        (screen.getByRole("checkbox", {
          name: /Create a git repository/,
        }) as HTMLInputElement).checked,
      ).toBe(true),
    );

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(api.scaffoldCreate).toHaveBeenCalled());
    const [spec] = vi.mocked(api.scaffoldCreate).mock.calls[0];
    expect(spec.init_repository).toBe(true);
  });

  it("leaves the repository alone once the person unticks it", async () => {
    ready();
    vi.mocked(api.scaffoldCreate).mockRejectedValue(
      new ApiError(422, "that location is not inside a git repository", {
        missing: "repository",
        path: "/home/nate/src/fresh",
      }),
    );
    vi.mocked(api.scaffoldPreview).mockResolvedValue({
      output: "greeter",
      folder: "/home/nate/src/fresh/greeter",
      repository: null,
      needs_repository: "/home/nate/src/fresh",
      command: "dotnet new console -o greeter -n Greeter",
      message: "Scaffold Greeter with `dotnet new console`\n",
    });
    fireEvent.change(await screen.findByLabelText("Location"), {
      target: { value: "/home/nate/src/fresh" },
    });
    const box = await screen.findByRole("checkbox", {
      name: /Create a git repository/,
    });
    await waitFor(() => expect((box as HTMLInputElement).checked).toBe(true));
    fireEvent.click(box);
    // Untouched by the next keystroke: a derived answer stops deriving the
    // moment somebody decides for themselves.
    fireEvent.change(screen.getByDisplayValue("Greeter"), {
      target: { value: "Widgets" },
    });
    await waitFor(() => expect((box as HTMLInputElement).checked).toBe(false));

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    // The server still refuses by type, and its sentence is what is shown.
    await screen.findByText(/not inside a git repository/);
    const [spec] = vi.mocked(api.scaffoldCreate).mock.calls[0];
    expect(spec.init_repository).toBe(false);
  });

  it("opens a project made elsewhere, and leaves one made here alone", async () => {
    ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue(STARTED);
    await screen.findByText("-f, --framework");
    // Inside the folder this window already shows: the tree will show it, so
    // there is nothing to open.
    const open = screen.getByRole("checkbox", {
      name: /Open the project when it is made/,
    });
    expect((open as HTMLInputElement).checked).toBe(false);

    vi.mocked(api.scaffoldPreview).mockResolvedValue({
      output: "greeter",
      folder: "/home/nate/src/greeter",
      repository: "/home/nate/src",
      command: "dotnet new console -o greeter -n Greeter",
      message: "Scaffold Greeter with `dotnet new console`\n",
    });
    fireEvent.change(screen.getByLabelText("Location"), {
      target: { value: "/home/nate/src" },
    });
    await waitFor(() => expect((open as HTMLInputElement).checked).toBe(true));

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(api.scaffoldCreate).toHaveBeenCalled());
    expect(vi.mocked(api.scaffoldCreate).mock.calls[0][0].open).toBe("new-window");
  });

  it("takes over this window when the new-window box is cleared", async () => {
    ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue(STARTED);
    await screen.findByText("-f, --framework");
    fireEvent.click(
      screen.getByRole("checkbox", { name: /Open the project when it is made/ }),
    );
    const inNew = screen.getByRole("checkbox", { name: /in a new window/ });
    expect((inNew as HTMLInputElement).checked).toBe(true);
    fireEvent.click(inNew);
    // Said plainly, because it costs you the terminal you were just reading.
    await screen.findByText(/every tab in this window goes/);

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(api.scaffoldCreate).toHaveBeenCalled());
    expect(vi.mocked(api.scaffoldCreate).mock.calls[0][0].open).toBe("this-window");
  });

  it("asks for no window at all when the project is not being opened", async () => {
    const { onStarted } = ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue(STARTED);
    await screen.findByText("-f, --framework");
    expect(
      (screen.getByRole("checkbox", { name: /in a new window/ }) as HTMLInputElement)
        .disabled,
    ).toBe(true);
    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(onStarted).toHaveBeenCalled());
    expect(vi.mocked(api.scaffoldCreate).mock.calls[0][0].open).toBe("none");
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
    render(<NewProjectDialog onStarted={vi.fn()} onClose={vi.fn()} />);

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
    render(<NewProjectDialog onStarted={vi.fn()} onClose={vi.fn()} />);
    await screen.findByText("Could not read the templates");
  });
});
