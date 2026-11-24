<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Run the Simple workspace in Windows Forms

For maintainers working on the Windows frontend. WinForms integrates with the Windows desktop and hosts the same Simple Browser editor as KDE. Windows workbench behavior uses the existing workspace API and source guards.

Build a Windows executable with the .NET 10 SDK. Use the WinForms component of the pinned WebView2 package and remove its unused WPF reference before assembly resolution.

<ui:copy id="windows-project">
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>WinExe</OutputType>
    <TargetFramework>net10.0-windows10.0.19041.0</TargetFramework>
    <UseWindowsForms>true</UseWindowsForms>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Microsoft.Web.WebView2" Version="1.0.4370.26" />
  </ItemGroup>
  <Target Name="UseWinFormsWebView" BeforeTargets="ResolveAssemblyReferences">
    <ItemGroup>
      <Reference Remove="@(Reference)" Condition="'%(Reference.Filename)' == 'Microsoft.Web.WebView2.Wpf'" />
    </ItemGroup>
  </Target>
</Project>
</ui:copy>

<ui:file path="apps/windows-simple/HickorySimple.csproj"><ui:paste select="#windows-project"/></ui:file>

The launcher accepts a workspace folder or a file. With a file, open its containing folder and encode the workspace-relative path for the editor. With no argument, offer a local folder or a server that serves Simple Browser. Local opens still use the bundled host and the engine still authorizes requested paths. The remote choice is explicit and opens under a temporary WebView profile.

The welcome dialog keeps the address prerequisite on the Connect button. A
disabled WinForms button cannot handle its own mouse movement, so the button
row anchors the explanatory tooltip to the same bounds.

<ui:copy id="windows-workspace-target">
namespace HickorySimple;

internal static class Program
{
    [STAThread]
    private static void Main(string[] args)
    {
        ApplicationConfiguration.Initialize();
        try
        {
            var launch = WorkspaceLaunch.Parse(args);
            if (launch is null) return;
            using var workspace = new WorkspaceApplication(launch);
            Application.Run(workspace);
        }
        catch (Exception error)
        {
            MessageBox.Show(error.Message, "Hickory Simple", MessageBoxButtons.OK, MessageBoxIcon.Error);
        }
    }
}

internal sealed record WorkspaceLaunch(string Root, string? File, ushort Port, Uri? Server = null)
{
    public static WorkspaceLaunch? Parse(string[] args)
    {
        string? root = null;
        string? file = null;
        ushort port = 38174;
        var hasPort = false;
        for (var index = 0; index < args.Length; index++)
        {
            switch (args[index])
            {
                case "--blank-window" when args.Length == 1:
                    break;
                case "--port" when !hasPort && index + 1 < args.Length &&
                    ushort.TryParse(args[index + 1], out port):
                    hasPort = true;
                    index++;
                    break;
                case "--file" when file is null && index + 1 < args.Length:
                    file = args[++index];
                    break;
                case var path when root is null && !path.StartsWith("--", StringComparison.Ordinal):
                    root = path;
                    break;
                default:
                    throw new ArgumentException("Use HickorySimple.exe [folder or file] [--file relative/path] [--port 0–65535].");
            }
        }

        if (root is null)
        {
            var choice = ShowWelcome(allowServer: file is null);
            if (choice.Server is not null) return new WorkspaceLaunch("", null, 0, choice.Server);
            root = choice.Folder;
            if (root is null) return null;
        }

        root = Path.GetFullPath(root);
        if (System.IO.File.Exists(root) && file is null)
        {
            file = Path.GetFileName(root);
            root = Path.GetDirectoryName(root)!;
        }
        if (!Directory.Exists(root))
            throw new ArgumentException("Choose an existing workspace folder.");
        if (file is not null)
        {
            if (Path.IsPathRooted(file))
                throw new ArgumentException("Pass a workspace-relative file after --file.");
            var target = Path.GetFullPath(file, root);
            file = Path.GetRelativePath(root, target);
            if (file == ".." || file.StartsWith(@"..\", StringComparison.Ordinal) ||
                Path.IsPathRooted(file) || !System.IO.File.Exists(target))
                throw new ArgumentException("Choose an existing file inside the workspace.");
            file = file.Replace('\\', '/');
        }
        return new WorkspaceLaunch(root, file, port);
    }

    private static (string? Folder, Uri? Server) ShowWelcome(bool allowServer)
    {
        var folder = new TaskDialogButton("Open Folder…");
        var server = new TaskDialogButton("Connect to a Workspace Server…");
        var page = new TaskDialogPage
        {
            Caption = "Hickory Docs",
            Heading = "Welcome to Hickory Docs",
            DefaultButton = folder,
        };
        page.Buttons.Add(folder);
        if (allowServer) page.Buttons.Add(server);
        page.Buttons.Add(TaskDialogButton.Cancel);

        while (true)
        {
            var choice = TaskDialog.ShowDialog(page);
            if (choice == folder)
            {
                using var picker = new FolderBrowserDialog { Description = "Open Folder" };
                if (picker.ShowDialog() == DialogResult.OK) return (picker.SelectedPath, null);
            }
            else if (choice == server)
            {
                var address = PromptForServer();
                if (address is not null) return (null, address);
            }
            else return (null, null);
        }
    }

    private static Uri? PromptForServer()
    {
        using var prompt = new Form
        {
            Text = "Connect to a Workspace Server",
            AutoSize = true,
            AutoSizeMode = AutoSizeMode.GrowAndShrink,
            StartPosition = FormStartPosition.CenterScreen,
            FormBorderStyle = FormBorderStyle.FixedDialog,
            MaximizeBox = false,
            MinimizeBox = false,
        };
        var content = new FlowLayoutPanel
        {
            FlowDirection = FlowDirection.TopDown,
            AutoSize = true,
            WrapContents = false,
            Padding = new Padding(12),
        };
        var label = new Label { Text = "Workspace server address", AutoSize = true };
        var address = new TextBox { Width = 320, AccessibleName = label.Text, PlaceholderText = "https://server:port" };
        var buttons = new FlowLayoutPanel { AutoSize = true };
        const string missingAddress = "Enter the workspace server address.";
        using var connectionTip = new ToolTip();
        var connect = new Button { Text = "Connect", AutoSize = true, Enabled = false, AccessibleDescription = missingAddress };
        var showingConnectionTip = false;
        void HideConnectionTip()
        {
            if (!showingConnectionTip) return;
            connectionTip.Hide(buttons);
            showingConnectionTip = false;
        }

        // Disabled child windows leave mouse movement with the containing panel.
        buttons.MouseMove += (_, eventArgs) =>
        {
            if (connect.Enabled || !connect.Bounds.Contains(eventArgs.Location))
            {
                HideConnectionTip();
                return;
            }
            if (showingConnectionTip) return;
            connectionTip.Show(missingAddress, buttons, connect.Left, connect.Bottom + 2);
            showingConnectionTip = true;
        };
        buttons.MouseLeave += (_, _) => HideConnectionTip();
        var cancel = new Button { Text = "Cancel", AutoSize = true, DialogResult = DialogResult.Cancel };
        Uri? result = null;
        address.TextChanged += (_, _) =>
        {
            connect.Enabled = !string.IsNullOrWhiteSpace(address.Text);
            connect.AccessibleDescription = connect.Enabled ? null : missingAddress;
            if (connect.Enabled) HideConnectionTip();
        };
        connect.Click += (_, _) =>
        {
            if (!Uri.TryCreate(address.Text.Trim(), UriKind.Absolute, out var url) ||
                url.Scheme is not ("http" or "https") || url.Host.Length == 0 ||
                url.UserInfo.Length != 0 || url.GetLeftPart(UriPartial.Authority).Contains('@') ||
                url.AbsolutePath != "/" || url.Query.Length != 0 || url.Fragment.Length != 0 || url.Port == 0)
            {
                MessageBox.Show(prompt, "Enter the server’s HTTP or HTTPS address, without a path or credentials.",
                    "Invalid workspace address", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                address.Focus();
                return;
            }
            result = new Uri(url, "/simple.html");
            prompt.DialogResult = DialogResult.OK;
        };
        buttons.Controls.AddRange([connect, cancel]);
        content.Controls.AddRange([label, address, buttons]);
        prompt.Controls.Add(content);
        prompt.AcceptButton = connect;
        prompt.CancelButton = cancel;
        return prompt.ShowDialog() == DialogResult.OK ? result : null;
    }

    public Uri EditorUri(int boundPort) => new UriBuilder("http", "127.0.0.1", boundPort, "/simple.html")
    {
        Query = File is null ? "" : $"path={Uri.EscapeDataString(File)}"
    }.Uri;
}
</ui:copy>

<ui:file path="apps/windows-simple/Program.cs"><ui:paste select="#windows-workspace-target"/></ui:file>

Start the same local host as KDE, with the browser assets beside it. A readiness response must identify the actual port before WebView2 navigates. Keep the process input open until the last window closes, then use EOF so the engine can detach its client.

<ui:copy id="windows-workspace-process">
using System.Diagnostics;

namespace HickorySimple;

internal sealed class WorkspaceHost : IDisposable
{
    private const string ReadyPrefix = "HICKORY_SIMPLE_PORT ";
    private readonly Process process;
    private readonly TaskCompletionSource<ushort> ready = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private string? lastError;

    public WorkspaceHost(WorkspaceLaunch workspace)
    {
        var info = new ProcessStartInfo(Path.Combine(AppContext.BaseDirectory, "hickory-simple-host.exe"))
        {
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
        };
        foreach (var argument in new[] {
            "--workspace-host", workspace.Root, Path.Combine(AppContext.BaseDirectory, "web"),
            workspace.Port.ToString(System.Globalization.CultureInfo.InvariantCulture)
        })
            info.ArgumentList.Add(argument);
        process = new Process { StartInfo = info };
        process.OutputDataReceived += (_, eventArgs) =>
        {
            var line = eventArgs.Data;
            if (line is null)
                ready.TrySetException(new IOException("The workspace host stopped before opening the editor."));
            else if (line.StartsWith(ReadyPrefix, StringComparison.Ordinal) &&
                ushort.TryParse(line.AsSpan(ReadyPrefix.Length), out var port) &&
                port > 0 && (workspace.Port == 0 || workspace.Port == port))
                ready.TrySetResult(port);
        };
        process.ErrorDataReceived += (_, eventArgs) =>
        {
            if (!string.IsNullOrWhiteSpace(eventArgs.Data)) lastError = eventArgs.Data;
        };
        try
        {
            if (!process.Start()) throw new IOException("Could not start the workspace host.");
        }
        catch
        {
            process.Dispose();
            throw;
        }
        process.BeginErrorReadLine();
        process.BeginOutputReadLine();
    }

    public async Task<ushort> ReadyAsync()
    {
        try
        {
            return await ready.Task.WaitAsync(TimeSpan.FromSeconds(60));
        }
        catch (Exception error) when (error is IOException or TimeoutException)
        {
            if (process.HasExited) process.WaitForExit();
            throw new IOException(lastError ?? error.Message, error);
        }
    }

    public void Dispose()
    {
        using (process)
        {
            try
            {
                process.StandardInput.Close();
            }
            catch (IOException) { }
            if (!process.WaitForExit(6000))
            {
                try { process.Kill(); }
                catch (InvalidOperationException) when (process.HasExited) { }
                process.WaitForExit(2000);
            }
        }
    }
}
</ui:copy>

<ui:file path="apps/windows-simple/WorkspaceHost.cs"><ui:paste select="#windows-workspace-process"/></ui:file>

The application owns the workspace process and its windows. Windows for the same workspace share an environment, and close independently. A fixed port and per-workspace profile allow editor sessions to be recovered when reopening the workspace. The port-zero development option gets a temporary profile.

<ui:copy id="windows-window-lifetime">
using System.Security.Cryptography;
using System.Text;
using Microsoft.Web.WebView2.Core;
using Microsoft.Web.WebView2.WinForms;

namespace HickorySimple;

internal sealed class WorkspaceApplication : ApplicationContext
{
    private readonly WorkspaceLaunch launch;
    private readonly HashSet<WorkspaceWindow> windows = [];
    private readonly string profile;
    private WorkspaceHost? host;
    private Task<CoreWebView2Environment>? environment;

    public WorkspaceApplication(WorkspaceLaunch launch)
    {
        this.launch = launch;
        var rootKey = Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(launch.Root)));
        profile = launch.Port == 0
            ? Path.Combine(Path.GetTempPath(), "HickorySimple", Guid.NewGuid().ToString("N"))
            : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "HickorySimple", "Workspaces", rootKey, launch.Port.ToString());

        var window = AddWindow();
        window.Shown += async (_, _) =>
        {
            try
            {
                var address = launch.Server;
                if (address is null)
                {
                    host = new WorkspaceHost(launch);
                    address = launch.EditorUri(await host.ReadyAsync());
                }
                if (window.IsDisposed) return;
                await window.MountAsync(address, GetEnvironment(), OpenWindowAsync);
                if (!window.IsDisposed) window.Editor.Source = address;
            }
            catch (Exception error)
            {
                if (!window.IsDisposed) window.Fail(error.Message);
            }
        };
        window.Show();
    }

    private Task<CoreWebView2Environment> GetEnvironment() =>
        environment ??= CoreWebView2Environment.CreateAsync(userDataFolder: profile);

    private WorkspaceWindow AddWindow()
    {
        var window = new WorkspaceWindow();
        windows.Add(window);
        window.FormClosed += (_, _) =>
        {
            windows.Remove(window);
            if (windows.Count == 0) ExitThread();
        };
        return window;
    }

    private async Task OpenWindowAsync(Uri address, CoreWebView2NewWindowRequestedEventArgs request)
    {
        var window = AddWindow();
        try
        {
            window.Show();
            await window.MountAsync(address, GetEnvironment(), OpenWindowAsync);
            if (!window.IsDisposed) request.NewWindow = window.Editor.CoreWebView2;
        }
        catch (Exception error)
        {
            if (!window.IsDisposed) window.Fail(error.Message);
        }
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing)
        {
            host?.Dispose();
            if (launch.Port == 0)
            {
                try { Directory.Delete(profile, recursive: true); }
                catch (IOException) { }
                catch (UnauthorizedAccessException) { }
            }
        }
        base.Dispose(disposing);
    }
}
</ui:copy>

<ui:file path="apps/windows-simple/WorkspaceApplication.cs"><ui:paste select="#windows-window-lifetime"/></ui:file>

The form fills its client area with the shared editor. Keep workspace navigation inside its loopback page, send explicitly opened external HTTP links to the default browser, and admit clipboard reads through a native prompt. Before closing, ask the editor to evaluate its usual unsaved-changes guard, then use a WinForms confirmation when required.

<ui:copy id="windows-editor-surface">
using System.Diagnostics;
using Microsoft.Web.WebView2.Core;
using Microsoft.Web.WebView2.WinForms;

namespace HickorySimple;

internal sealed class WorkspaceWindow : Form
{
    private readonly Label status = new() { Dock = DockStyle.Fill, TextAlign = ContentAlignment.MiddleCenter,
        Text = "Opening workspace…" };
    public WebView2 Editor { get; } = new() { Dock = DockStyle.Fill, Visible = false };
    private bool editorLoaded;
    private bool closing;
    private bool closeApproved;

    public WorkspaceWindow()
    {
        Text = "Hickory Simple";
        ClientSize = new Size(1120, 760);
        Controls.Add(status);
        Controls.Add(Editor);
    }

    public async Task MountAsync(Uri workspace, Task<CoreWebView2Environment> environment,
        Func<Uri, CoreWebView2NewWindowRequestedEventArgs, Task> openWindow)
    {
        var selected = await environment;
        if (IsDisposed) return;
        await Editor.EnsureCoreWebView2Async(selected);
        if (IsDisposed) return;
        var core = Editor.CoreWebView2;
        core.Settings.AreHostObjectsAllowed = false;
        core.Settings.IsWebMessageEnabled = false;
        core.Settings.AreDevToolsEnabled = false;
        core.Settings.AreDefaultScriptDialogsEnabled = true;
        core.NavigationStarting += (_, request) =>
        {
            if (IsWorkspacePage(request.Uri, workspace)) return;
            request.Cancel = true;
            if (request.IsUserInitiated && !request.IsRedirected) OpenExternal(request.Uri, workspace);
        };
        core.NewWindowRequested += async (_, request) =>
        {
            request.Handled = true;
            if (!request.IsUserInitiated) return;
            if (!IsWorkspacePage(request.Uri, workspace))
            {
                OpenExternal(request.Uri, workspace);
                return;
            }

            using var deferral = request.GetDeferral();
            await Task.Yield();
            await openWindow(new Uri(request.Uri), request);
        };
        core.PermissionRequested += async (_, request) =>
        {
            request.SavesInProfile = false;
            request.State = CoreWebView2PermissionState.Deny;
            if (request.PermissionKind != CoreWebView2PermissionKind.ClipboardRead ||
                !request.IsUserInitiated || !SameOrigin(request.Uri, workspace)) return;

            using var deferral = request.GetDeferral();
            await Task.Yield();
            if (!IsDisposed && MessageBox.Show(this,
                "Allow this workspace to read the clipboard for this editor request?",
                "Clipboard access", MessageBoxButtons.YesNo, MessageBoxIcon.Question,
                MessageBoxDefaultButton.Button2) == DialogResult.Yes)
                request.State = CoreWebView2PermissionState.Allow;
        };
        core.WindowCloseRequested += (_, _) => BeginInvoke(Close);
        core.NavigationCompleted += (_, result) =>
        {
            if (result.IsSuccess && IsWorkspacePage(core.Source, workspace))
            {
                editorLoaded = true;
                status.Visible = false;
                Editor.Visible = true;
            }
            else if (!editorLoaded && result.WebErrorStatus != CoreWebView2WebErrorStatus.OperationCanceled)
                Fail("The workspace could not load. Reopen Hickory and check that the workspace host is running.");
        };
        core.ProcessFailed += (_, failure) =>
        {
            if (failure.ProcessFailedKind is CoreWebView2ProcessFailedKind.BrowserProcessExited or
                CoreWebView2ProcessFailedKind.RenderProcessExited)
                Fail("The editor renderer stopped. Reopen Hickory and use Editor sessions to recover a draft.");
        };
    }

    private static bool SameOrigin(string candidate, Uri workspace) =>
        Uri.TryCreate(candidate, UriKind.Absolute, out var uri) &&
        uri.UserInfo.Length == 0 && uri.Scheme == workspace.Scheme &&
        uri.Host == workspace.Host && uri.Port == workspace.Port;

    private static bool IsWorkspacePage(string candidate, Uri workspace) =>
        SameOrigin(candidate, workspace) && new Uri(candidate).AbsolutePath == "/simple.html";

    private void OpenExternal(string candidate, Uri workspace)
    {
        if (SameOrigin(candidate, workspace) || !Uri.TryCreate(candidate, UriKind.Absolute, out var uri) ||
            uri.Scheme is not ("http" or "https") || uri.Host.Length == 0 || uri.UserInfo.Length != 0)
            return;
        try { Process.Start(new ProcessStartInfo(uri.AbsoluteUri) { UseShellExecute = true }); }
        catch (System.ComponentModel.Win32Exception error)
        {
            BeginInvoke(() => MessageBox.Show(this, error.Message, "Could not open link"));
        }
    }

    public void Fail(string message)
    {
        Editor.Visible = false;
        status.Text = message;
        status.Visible = true;
    }

    protected override void OnFormClosing(FormClosingEventArgs args)
    {
        base.OnFormClosing(args);
        if (args.Cancel || !editorLoaded || closeApproved) return;
        args.Cancel = true;
        if (!closing) _ = ConfirmCloseAsync();
    }

    private async Task ConfirmCloseAsync()
    {
        closing = true;
        var message = "The editor has pending changes. Close this window anyway?";
        var needsConfirmation = true;
        try
        {
            // Ask the same synchronous guard used when the browser window is closed.
            var state = await Editor.ExecuteScriptAsync(
                "window.dispatchEvent(new Event('beforeunload', { cancelable: true }))")
                .WaitAsync(TimeSpan.FromSeconds(3));
            needsConfirmation = state != "true";
        }
        catch (Exception error) when (error is TimeoutException or InvalidOperationException or
            System.Runtime.InteropServices.COMException)
        {
            message = "Hickory could not check for unsaved changes. Close this window anyway?";
        }

        closeApproved = !needsConfirmation || MessageBox.Show(this, message, "Close workspace",
            MessageBoxButtons.YesNo, MessageBoxIcon.Warning, MessageBoxDefaultButton.Button2) == DialogResult.Yes;
        closing = false;
        if (closeApproved && !IsDisposed) BeginInvoke(Close);
    }
}
</ui:copy>

<ui:file path="apps/windows-simple/WorkspaceWindow.cs"><ui:paste select="#windows-editor-surface"/></ui:file>

Build the Simple Browser entry, the Rust workspace host, and the WinForms app together. The app folder includes the .NET runtime and resolves its workspace host and browser assets relative to its executable. Windows supplies the Evergreen WebView2 Runtime.

<ui:copy id="windows-bundle-layout">
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

case "$(rustc -vV | sed -n 's/^host: //p')" in
  x86_64-pc-windows-msvc) runtime=win-x64 ;;
  aarch64-pc-windows-msvc) runtime=win-arm64 ;;
  *) echo 'Run just simple-build windows on Windows with the MSVC Rust toolchain and .NET 10 SDK.' >&2; exit 2 ;;
esac

build=target/simple-windows
if [ ! -f apps/web/node_modules/vite/bin/vite.js ] || [ apps/web/package-lock.json -nt apps/web/node_modules ]; then
  npm --prefix apps/web ci
fi
(cd apps/web && node node_modules/vite/bin/vite.js build --config vite.simple.config.ts \
  --outDir ../../target/simple-windows/web --emptyOutDir)
cargo build --locked --target-dir target -p hickory-cli --example simple_dev
dotnet publish apps/windows-simple/HickorySimple.csproj --configuration Release \
  --runtime "$runtime" --self-contained --artifacts-path "$build/dotnet" --output "$build/app"
cp target/debug/examples/simple_dev.exe "$build/app/hickory-simple-host.exe"
rm -rf "$build/app/web"
cp -R "$build/web" "$build/app/web"
</ui:copy>

<ui:file path="scripts/build-simple-windows.sh" executable="true"><ui:paste select="#windows-bundle-layout"/></ui:file>

The Windows smoke test opens an exact filename containing spaces, punctuation and Unicode in a real WinForms window. It checks that Files and source appear together, a cross-origin API request is refused, Save writes the guarded edit, a second window shares the workspace, generated source stays read-only, and closing the last window detaches the host.

<ui:copy id="windows-window-smoke">
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { once } from "node:events";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { chromium, expect } from "../apps/web/node_modules/@playwright/test/index.mjs";

async function eventually(probe, label, timeout = 45000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const result = await probe();
    if (result) return result;
    await delay(100);
  }
  throw new Error(`Timed out waiting for ${label}.`);
}

const temp = await mkdtemp(join(tmpdir(), "hickory-windows-"));
const workspace = join(temp, "workspace");
await mkdir(workspace);
const name = "note + #é.txt";
const source = join(workspace, name);
await writeFile(source, "first line\n");
await writeFile(join(workspace, "sibling.txt"), "same workspace\n");
await writeFile(join(workspace, "author.md"), [
  '<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">',
  '<hick:file path="generated.txt">Generated source',
  "</hick:file>",
  "</hick:doc>",
  "",
].join("\n"));
await writeFile(join(workspace, "generated.txt"), "Generated source\n");

const listener = createServer().listen(0, "127.0.0.1");
await once(listener, "listening");
const debug = listener.address().port;
await new Promise((done, fail) => listener.close(error => error ? fail(error) : done()));
const app = spawn(resolve("target/simple-windows/app/HickorySimple.exe"), [source, "--port", "0"], {
  env: {
    ...process.env,
    HICKORY_STATE_DIR: join(temp, "engine"),
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${debug}`,
  },
  stdio: "ignore",
});
const stopped = once(app, "exit");
stopped.catch(() => {});
let browser;

try {
  const debuggerOrigin = `http://127.0.0.1:${debug}`;
  await eventually(() => fetch(`${debuggerOrigin}/json/version`, { signal: AbortSignal.timeout(1000) })
    .then(response => response.ok).catch(() => false), "WebView2");
  browser = await chromium.connectOverCDP(debuggerOrigin, { noDefaults: true });
  const page = await eventually(() => browser.contexts().flatMap(context => context.pages())
    .find(tab => new URL(tab.url()).pathname === "/simple.html"), "the WinForms workspace");
  const editor = page.locator(".cm-content").and(page.locator(`[aria-label=${JSON.stringify(name)}]`));
  const tree = page.locator('[aria-label="Files"]');
  await expect(editor).toHaveText("first line");
  await expect(editor).toHaveAttribute("contenteditable", "true");
  await expect(tree.locator('[title="sibling.txt"]')).toBeVisible();

  const endpoint = new URL("/api/files", page.url());
  const refused = await fetch(endpoint, { headers: { Origin: "https://other.example" } });
  assert.equal(refused.status, 403);

  await editor.press("End");
  await page.keyboard.insertText(" from WinForms");
  assert.equal(await readFile(source, "utf8"), "first line\n");
  await page.locator("nav").getByText("File", { exact: true }).click();
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await eventually(async () => await readFile(source, "utf8") === "first line from WinForms\n", "the guarded save");

  await page.locator("nav").getByText("File", { exact: true }).click();
  await page.getByRole("button", { name: "New Window", exact: true }).click();
  const other = await eventually(() => page.context().pages().find(tab =>
    tab !== page && new URL(tab.url()).pathname === "/simple.html"), "a second WinForms workspace");
  await expect(other.locator(".cm-content").and(
    other.locator(`[aria-label=${JSON.stringify(name)}]`))).toHaveText("first line from WinForms");

  await other.bringToFront();
  await other.locator('[aria-label="Files"] [title="generated.txt"]').click();
  await expect(other.locator('.cm-content[aria-label="generated.txt"]')).toHaveAttribute("contenteditable", "false");
  assert.equal(await readFile(join(workspace, "generated.txt"), "utf8"), "Generated source\n");
  const popupClosed = once(other, "close");
  await other.evaluate(() => { setTimeout(() => window.close(), 50); });
  await Promise.race([popupClosed, delay(15000, undefined, { ref: false }).then(() => { throw new Error("Second window did not close."); })]);

  execFileSync("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command",
    `if (-not (Get-Process -Id ${app.pid}).CloseMainWindow()) { exit 1 }`]);
  const exit = await Promise.race([stopped, delay(15000, null, { ref: false })]);
  assert.equal(exit?.[0], 0, "Closing the last window should stop the app.");
  await eventually(() => fetch(endpoint, { signal: AbortSignal.timeout(1000) })
    .then(() => false).catch(() => true), "the workspace host to detach", 10000);
  console.log("WinForms workspace smoke passed.");
} finally {
  try {
    if (browser) await browser.close();
  } finally {
    if (app.pid && app.exitCode === null) {
      try { execFileSync("taskkill.exe", ["/PID", String(app.pid), "/T", "/F"], { stdio: "ignore" }); }
      catch { if (app.exitCode === null) app.kill(); }
      await Promise.race([stopped, delay(5000, undefined, { ref: false })]);
    }
    await rm(temp, { recursive: true, force: true, maxRetries: 4 });
  }
}
</ui:copy>

<ui:file path="scripts/test-simple-windows.mjs"><ui:paste select="#windows-window-smoke"/></ui:file>

</ui:doc>
