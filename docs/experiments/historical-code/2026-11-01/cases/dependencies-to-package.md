# dependencies-to-package

Repair ConvertToPackageReferences: replace ProjectReference entries targeting local versioned projects with PackageReference entries using the target filename without extension and its PackageVersion. Skip consumers whose project name without .csproj ends in .Test. Preserve unrelated content. Keep the signature and supplied AbsolutePath API. referencePredicate and backup are unused in this historical contract. Inputs use simple self-closing XML entries.

## Historical solution

Retained from [nloum/PackageProjectDependencySwitcher at 9fa4a1a917f4](https://github.com/nloum/PackageProjectDependencySwitcher/blob/9fa4a1a917f491228f85c3b772ba8856bc5af7c0/src/PackageProjectDependencySwitcher/DependencySwitcherService.cs#L69). This is a method projection with the [fixture limits](../../../../developers/historical-code-benchmark.md); the prompt is retrospective.

```csharp
        public void ConvertToPackageReferences(AbsolutePath solutionFolder, Func<AbsolutePath, string, bool> referencePredicate, bool backup)
        {
            var files = solutionFolder.GetDescendants("*.csproj").ToImmutableList();

            var filesToPackages = new Dictionary<AbsolutePath, Func<string, string>>();
            
            foreach (var file in files)
            {
                var text = file.ReadAllText();
                var packageVersion = _packageVersion.Match(text);

                if (packageVersion.Success)
                {
                    var regex = new Regex($"<ProjectReference Include=\".+{file.Name}\" */>");

                    var packageVersionNumber = packageVersion.Groups["packageVersion"].Value;
                    filesToPackages[file] = str => regex.Replace(str, $"<PackageReference Include=\"{file.WithoutExtension().Name}\" Version=\"{packageVersionNumber}\" />");
                }
            }

            foreach (var file in files)
            {
                var text = file.ReadAllText();

                if (file.WithoutExtension().HasExtension(".Test"))
                {
                    // TODO - make this behavior configurable.
                    continue;
                }
                
                var numChanges = 0;
                foreach (var package in filesToPackages)
                {
                    var prevText = text;
                    text = package.Value(text);
                    if (!text.Equals(prevText))
                    {
                        numChanges++;
                        Console.WriteLine($"Changing {file.WithoutExtension().Name}'s dependency on {package.Key.WithoutExtension().Name} from a project reference to a package reference");
                    }
                }
                
                file.WriteAllText(text);
            }
        }
```

## Public acceptance example

```csharp
Setup(("/s/Lib/Lib.csproj","<Project><PackageVersion>1.2</PackageVersion></Project>"),("/s/App/App.csproj","<Project><ProjectReference Include=\"../Lib/Lib.csproj\" /></Project>")); new Subject().ConvertToPackageReferences(new AbsolutePath("/s"),(p,t)=>true,false); True(AbsolutePath.Files["/s/App/App.csproj"].Contains("<PackageReference Include=\"Lib\" Version=\"1.2\" />"));
```
