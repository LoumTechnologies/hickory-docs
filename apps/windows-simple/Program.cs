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

internal sealed record WorkspaceLaunch(string Root, string? File, ushort Port)
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
            using var picker = new FolderBrowserDialog { Description = "Open a Hickory workspace" };
            if (picker.ShowDialog() != DialogResult.OK) return null;
            root = picker.SelectedPath;
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

    public Uri EditorUri(int boundPort) => new UriBuilder("http", "127.0.0.1", boundPort, "/simple.html")
    {
        Query = File is null ? "" : $"path={Uri.EscapeDataString(File)}"
    }.Uri;
}
