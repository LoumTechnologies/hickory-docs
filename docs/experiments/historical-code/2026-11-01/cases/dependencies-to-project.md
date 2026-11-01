# dependencies-to-project

Repair ConvertToProjectReferences so it converts PackageReference entries matching local versioned projects into ProjectReference entries relative to each consumer directory. Do not modify a project to reference itself; preserve unrelated entries and do not write unchanged files. Keep the public signature; referencePredicate and backup are unused in this historical contract. Inputs use simple self-closing XML entries and local projects have PackageVersion tags. Use supplied AbsolutePath collaborator.

## Historical solution

Retained from [nloum/PackageProjectDependencySwitcher at 9fa4a1a917f4](https://github.com/nloum/PackageProjectDependencySwitcher/blob/9fa4a1a917f491228f85c3b772ba8856bc5af7c0/src/PackageProjectDependencySwitcher/DependencySwitcherService.cs#L15). This is a method projection with the [fixture limits](../../../../developers/historical-code-benchmark.md); the prompt is retrospective.

```csharp
        public void ConvertToProjectReferences(AbsolutePath solutionFolder, Func<AbsolutePath, string, bool> referencePredicate, bool backup)
        {
            var files = solutionFolder.GetDescendants("*.csproj").ToImmutableList();

            var filesToPackages = new Dictionary<AbsolutePath, Func<string, AbsolutePath, string>>();
            
            foreach (var file in files)
            {
                var text = file.ReadAllText();
                var packageVersion = _packageVersion.Match(text);

                if (packageVersion.Success)
                {
                    var regex = new Regex($"<PackageReference Include=\"{Path.GetFileNameWithoutExtension(file.Name)}\".+/>");

                    filesToPackages[file] = (csprojFileText, relativeTo) =>
                    {
                        var path = file.RelativeTo(relativeTo);

                        var result = regex.Replace(csprojFileText,
                                $"<ProjectReference Include=\"{path}\" />");
                        return result;
                    };
                }
            }
            
            foreach (var file in files)
            {
                var text = file.ReadAllText();

                var numChanges = 0;
                foreach (var package in filesToPackages)
                {
                    if (package.Key.Equals(file))
                    {
                        continue;
                    }
                    
                    var prevText = text;
                    text = package.Value(text, file.Parent());
                    if (!text.Equals(prevText))
                    {
                        numChanges++;
                        Console.WriteLine($"Changing {Path.GetFileNameWithoutExtension(file.ToString())}'s dependency on {Path.GetFileNameWithoutExtension(package.Key.ToString())} from a package reference to a project reference");
                    }
                }

                if (numChanges > 0)
                {
                    file.WriteAllText(text);
                }
            }
        }
```

## Public acceptance example

```csharp
Setup(("/s/Lib/Lib.csproj","<Project><PackageVersion>1.2</PackageVersion></Project>"),("/s/App/App.csproj","<Project><PackageReference Include=\"Lib\" Version=\"0.1\" /></Project>")); new Subject().ConvertToProjectReferences(new AbsolutePath("/s"),(p,t)=>true,false); True(AbsolutePath.Files["/s/App/App.csproj"].Contains("<ProjectReference Include=\"../Lib/Lib.csproj\" />")); Equal(AbsolutePath.Writes.Count,1);
```
