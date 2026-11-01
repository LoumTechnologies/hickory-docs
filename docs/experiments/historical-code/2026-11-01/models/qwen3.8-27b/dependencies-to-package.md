# qwen3.8:27b: dependencies-to-package

Public and held-out checks passed in this reduced fixture, after 1 attempt(s).

The goal is to replace `ProjectReference` entries in consumer projects with `PackageReference` entries, using the target project's name (without extension) and its `PackageVersion`. We must skip consumers whose name ends in `.Test`.

### Explanation
1.  **Identify Targets**: Iterate through all `.csproj` files in the solution folder. For each file, check if it contains a `<PackageVersion>` tag. If it does, it is a "target" project that can be referenced as a package. We store the target's path, its package name (filename without extension), and the extracted version.
2.  **Identify Consumers**: Iterate through the same list of files. For each file, check if it is a consumer (i.e., it contains `ProjectReference` entries). We must skip any consumer whose filename (without `.csproj`) ends with `.Test`.
3.  **Transformation**: For each valid consumer, we look for `ProjectReference` entries that point to any of the identified targets.
    *   We extract the `Include` path from the `ProjectReference`.
    *   We resolve this path relative to the consumer's directory to get the absolute path of the referenced project.
    *   We check if this absolute path matches one of our identified targets.
    *   If it matches, we replace the `ProjectReference` entry with a `PackageReference` entry using the target's package name and version.
4.  **Preservation**: We use a regex replacement that only modifies the specific `ProjectReference` lines that match our targets, leaving all other content intact.

### Implementation

```csharp
        public void ConvertToPackageReferences(AbsolutePath solutionFolder, Func<AbsolutePath, string, bool> referencePredicate, bool backup)
        {
            var files = solutionFolder.GetDescendants("*.csproj").ToImmutableList();

            // Step 1: Identify target projects that have a PackageVersion
            var targets = new Dictionary<AbsolutePath, (string Name, string Version)>();
            foreach (var file in files)
            {
                var text = file.ReadAllText();
                var packageVersionMatch = _packageVersion.Match(text);
                if (packageVersionMatch.Success)
                {
                    var name = Path.GetFileNameWithoutExtension(file.ToString());
                    var version = packageVersionMatch.Groups["packageVersion"].Value;
                    targets[file] = (name, version);
                }
            }

            // Step 2: Process consumer projects
            foreach (var file in files)
            {
                // Skip consumers whose project name without .csproj ends in .Test
                var consumerName = Path.GetFileNameWithoutExtension(file.ToString());
                if (consumerName.EndsWith(".Test"))
                {
                    continue;
                }

                var text = file.ReadAllText();
                var originalText = text;

                // Find all ProjectReference entries in this consumer
                var projectReferenceRegex = new Regex(@"<ProjectReference\s+Include=""([^""]+)""[^/]*?/>");
                var matches = projectReferenceRegex.Matches(text);

                foreach (Match match in matches)
                {
                    var includePath = match.Groups[1].Value;
                    
                    // Resolve the include path relative to the consumer's directory
                    // The include path is relative to the consumer project file's location
                    var consumerDir = file.Parent();
                    AbsolutePath targetPath;
                    try
                    {
                        // Combine the consumer directory with the relative include path
                        // Note: AbsolutePath doesn't have a Combine method, so we use Path.Combine on the strings
                        var combinedPath = Path.Combine(consumerDir.ToString(), includePath);
                        // Normalize the path to handle ".." etc.
                        combinedPath = Path.GetFullPath(combinedPath);
                        targetPath = new AbsolutePath(combinedPath);
                    }
                    catch
                    {
                        continue;
                    }

                    // Check if this target is in our targets dictionary
                    if (targets.TryGetValue(targetPath, out var targetInfo))
                    {
                        // Replace the ProjectReference with a PackageReference
                        var newReference = $"<PackageReference Include=\"{targetInfo.Name}\" Version=\"{targetInfo.Version}\" />";
                        text = text.Replace(match.Value, newReference);
                    }
                }

                // Only write if there were changes
                if (text != originalText)
                {
                    file.WriteAllText(text);
                }
            }
        }
```