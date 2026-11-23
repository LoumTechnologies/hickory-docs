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
        var connect = new Button { Text = "Connect", AutoSize = true, Enabled = false };
        var cancel = new Button { Text = "Cancel", AutoSize = true, DialogResult = DialogResult.Cancel };
        Uri? result = null;
        address.TextChanged += (_, _) => connect.Enabled = !string.IsNullOrWhiteSpace(address.Text);
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
