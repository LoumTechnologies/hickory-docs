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
