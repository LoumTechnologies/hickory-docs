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

            var address = new Uri(request.Uri);
            if (address.Query == "?blank-window")
            {
                try
                {
                    Process.Start(new ProcessStartInfo(Application.ExecutablePath, "--blank-window"));
                }
                catch (Exception error)
                {
                    MessageBox.Show(this, error.Message, "Could not open another window",
                        MessageBoxButtons.OK, MessageBoxIcon.Error);
                }
                return;
            }

            using var deferral = request.GetDeferral();
            await Task.Yield();
            await openWindow(address, request);
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
