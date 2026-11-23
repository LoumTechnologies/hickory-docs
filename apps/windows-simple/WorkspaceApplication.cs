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
